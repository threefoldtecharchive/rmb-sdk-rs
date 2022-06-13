use anyhow::{Context, Ok, Result};
use async_trait::async_trait;
use bb8_redis::{
    bb8::{Pool, PooledConnection},
    redis::AsyncCommands,
    RedisConnectionManager,
};
use workers::Work;

use crate::{
    msg::{Message, MessageBuilder},
    util::Queue,
};

use super::{HandlerInput, HandlerOutput, Module};

type Reply = MessageBuilder;

#[derive(Clone)]
pub struct WorkRunner {
    pool: Pool<RedisConnectionManager>,
}

impl WorkRunner {
    pub fn new(pool: Pool<RedisConnectionManager>) -> Self {
        WorkRunner { pool }
    }

    async fn get_connection(&self) -> Result<PooledConnection<'_, RedisConnectionManager>> {
        let conn = self
            .pool
            .get()
            .await
            .context("unable to retrieve a redis connection from the pool")?;

        Ok(conn)
    }

    async fn prepare(msg: &mut Message, result: HandlerOutput) {
        msg.data = base64::encode(result.data);

        let src = msg.source;
        msg.source = msg.destination[0];
        msg.destination = vec![src];
    }

    async fn send(&self, msg: Message) -> Result<()> {
        let mut conn = self.get_connection().await?;
        conn.rpush(Queue::Reply.as_ref(), msg)
            .await
            .context("unable to send your reply message")?;

        Ok(())
    }
}

#[async_trait]
impl Work for WorkRunner {
    type Input = (String, Message, &'static Module);
    type Output = ();
    async fn run(&self, input: Self::Input) -> Self::Output {
        let (command, mut msg, root) = input;
        let data = base64::decode(&msg.data).unwrap(); // <- not safe
        let handler = root
            .lookup(command)
            .context("handler not found this should never happen")
            .unwrap();

        let out = handler(HandlerInput {
            data: data,
            schema: msg.schema.clone(),
        });

        Self::prepare(&mut msg, out).await;

        if let Err(err) = self.send(msg).await {
            log::debug!("{}", err);
        }
    }
}
