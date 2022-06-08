pub mod request;
pub mod response;

use crate::util::Queue;
use anyhow::{Context, Result};
use bb8_redis::{
    bb8::{Pool, PooledConnection},
    redis::AsyncCommands,
    RedisConnectionManager,
};
use request::Request;
use response::Response;

pub struct Client {
    pool: Pool<RedisConnectionManager>,
}

impl Client {
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

    pub async fn send(&self, req: Request) -> Result<Response> {
        let mut conn = self.get_connection().await?;
        conn.rpush(Queue::Local.as_ref(), req.body())
            .await
            .context("unable to send your message")?;

        let response = Response::new(
            self.pool.clone(),
            req.get_ret().to_owned(),
            req.destinations_len(),
            req.calc_deadline(),
        );

        Ok(response)
    }
}
