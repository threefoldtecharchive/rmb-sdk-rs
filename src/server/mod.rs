use anyhow::{Context, Result};
use bb8_redis::{
    bb8::{Pool, PooledConnection},
    redis::{AsyncCommands, RedisResult},
    RedisConnectionManager,
};
use std::{collections::HashMap, hash::Hash};

use crate::msg::Message;

pub struct Server<C, F>
where
    C: ToString,
    F: Fn(String),
{
    pool: Pool<RedisConnectionManager>,
    channels: HashMap<C, F>,
}

impl<C, F> Server<C, F>
where
    C: ToString + Eq + Hash,
    F: Fn(String),
{
    pub fn new(pool: Pool<RedisConnectionManager>) -> Self {
        let channels = HashMap::new();
        Self { pool, channels }
    }

    async fn get_connection(&self) -> Result<PooledConnection<'_, RedisConnectionManager>> {
        let conn = self
            .pool
            .get()
            .await
            .context("unable to retrieve a redis connection from the pool")?;

        Ok(conn)
    }

    pub fn add_channel(mut self, cmd: C, handler: F) -> Self {
        self.channels.insert(cmd, handler);
        self
    }

    pub async fn exec(&self) -> Result<()> {
        let mut conn = self.get_connection().await?;
        loop {
            for (cmd, handler) in self.channels.iter() {
                let msg_res: RedisResult<Message> =
                    conn.lpop(format!("msgbus.{}", cmd.to_string()), None).await;

                if let Ok(msg) = msg_res {
                    // decode an encoded data by me which will not panic
                    let args = base64::decode(msg.data).unwrap();
                    let args = String::from_utf8(args).unwrap();
                    let reply = handler(args);
                }
            }
        }
    }

    /// Send a reply to a previously command received
    async fn reply<R: AsRef<str>>(&self, reply: R, msg: &Message) -> Result<()> {
        let mut conn = self.get_connection().await?;

        conn.rpush(reply.as_ref(), msg)
            .await
            .context("unable to send your reply")?;

        Ok(())
    }

    // pub fn reply() {}
}
