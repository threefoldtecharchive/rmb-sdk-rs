use anyhow::{Context, Result};
use async_trait::async_trait;
use bb8_redis::{
    bb8::{Pool, PooledConnection},
    redis::AsyncCommands,
    RedisConnectionManager,
};
use workers::Work;

use crate::protocol::{Error, IncomingRequest, OutgoingResponse, Queue};

use super::{HandlerInput, Module};

pub struct WorkRunner<D> {
    pool: Pool<RedisConnectionManager>,
    root: Module<D>,
    data: D,
}

impl<D> WorkRunner<D> {
    pub fn new(pool: Pool<RedisConnectionManager>, data: D, root: Module<D>) -> Self {
        WorkRunner {
            pool,
            data,
            root: root,
        }
    }

    #[inline]
    async fn get_connection(&self) -> Result<PooledConnection<'_, RedisConnectionManager>> {
        let conn = self
            .pool
            .get()
            .await
            .context("unable to retrieve a redis connection from the pool")?;

        Ok(conn)
    }

    async fn send(&self, msg: OutgoingResponse) -> Result<()> {
        let mut conn = self.get_connection().await?;
        conn.rpush(Queue::Reply.as_ref(), msg)
            .await
            .context("unable to send your reply message")?;

        Ok(())
    }
}

#[async_trait]
impl<D> Work for WorkRunner<D>
where
    D: Clone + Send + Sync + 'static,
{
    type Input = (String, IncomingRequest);
    type Output = ();
    async fn run(&self, input: Self::Input) -> Self::Output {
        let (command, msg) = input;
        let data = base64::decode(&msg.data).unwrap(); // <- not safe
        let handler = self
            .root
            .lookup(command)
            .context("handler not found this should never happen")
            .unwrap();

        let state = self.data.clone();
        let out = handler
            .call(
                state,
                HandlerInput {
                    source: &msg.source,
                    data: data,
                    schema: msg.schema.as_deref(),
                },
            )
            .await;

        let response = OutgoingResponse {
            version: msg.version,
            schema: if let Ok(ref out) = out {
                Some(out.schema.clone())
            } else {
                None
            },
            reference: msg.reference,
            destination: msg.source,
            timestamp: 0,
            data: if let Ok(ref out) = out {
                base64::encode(&out.data)
            } else {
                String::default()
            },
            error: if let Err(err) = out {
                Some(Error {
                    code: 0,
                    message: err.to_string(),
                })
            } else {
                None
            },
        };

        if let Err(err) = self.send(response).await {
            log::debug!("{}", err);
        }
    }
}
