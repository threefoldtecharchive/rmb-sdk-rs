use crate::msg::Message;
use anyhow::{Context, Result};
use async_stream::stream;
use bb8_redis::{
    bb8::{Pool, PooledConnection},
    redis::AsyncCommands,
    RedisConnectionManager,
};
use futures::Stream;

pub struct Response {
    pool: Pool<RedisConnectionManager>,
}

impl Response {
    pub fn new(pool: Pool<RedisConnectionManager>) -> Self {
        Self { pool }
    }

    async fn get_connection(&self) -> Result<PooledConnection<'_, RedisConnectionManager>> {
        let conn = self
            .pool
            .get()
            .await
            .context("unable to retrieve a redis connection from the pool")?;

        Ok(conn)
    }

    pub fn response<C: AsRef<str>>(self, ret: C) -> impl Stream<Item = Result<Message>> {
        stream! {
            let mut conn = self.get_connection().await.unwrap();
            let msg: Result<Message> = conn
            .lpop(ret.as_ref(), None)
            .await
            .context("failed to get a response message");
            yield msg;
        }
    }
}
