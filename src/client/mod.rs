pub mod builder;
pub mod response;

use crate::protocol::{Message, Queue};
use crate::util::timestamp;
use anyhow::{Context, Result};
use bb8_redis::{
    bb8::{Pool, PooledConnection},
    redis::AsyncCommands,
    RedisConnectionManager,
};

use response::Response;

pub type Request = builder::Request;

pub struct Client {
    pool: Pool<RedisConnectionManager>,
}

impl Client {
    /// Client creates a new client
    pub fn new(pool: Pool<RedisConnectionManager>) -> Self {
        Self { pool }
    }

    /// create a client from URL
    pub async fn from<U: AsRef<str>>(u: U) -> Result<Self> {
        let mgr = RedisConnectionManager::new(u.as_ref())?;
        let pool = Pool::builder().max_size(20).build(mgr).await?;

        Ok(Self { pool })
    }

    async fn get_connection(&self) -> Result<PooledConnection<'_, RedisConnectionManager>> {
        let conn = self
            .pool
            .get()
            .await
            .context("unable to retrieve a redis connection from the pool")?;

        Ok(conn)
    }

    /// send a request and get a response object
    pub async fn send(&self, req: Request) -> Result<Response> {
        let mut msg: Message = req.into();

        // we set and calculate deadline based on the sending time
        // not on the message creation time.
        msg.now = timestamp();
        let deadline = msg.now + msg.expiration;
        let response = Response::new(
            self.pool.clone(),
            msg.command.clone(),
            msg.destination.len(),
            deadline,
        );

        let mut conn = self.get_connection().await?;
        conn.rpush(Queue::Local.as_ref(), msg)
            .await
            .context("unable to send your message")?;

        Ok(response)
    }
}
