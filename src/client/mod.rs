pub mod request;
pub mod response;

use anyhow::{Context, Result};
use bb8_redis::{
    bb8::{Pool, PooledConnection},
    redis::AsyncCommands,
    RedisConnectionManager,
};
use futures::Stream;
use request::Request; 
use response::Response;

use crate::{msg::Message};

enum Queue {
    Local,
    Reply,
}

impl AsRef<str> for Queue {
    fn as_ref(&self) -> &str {
        match self {
            Queue::Local => "msgbus.system.local",
            Queue::Reply => "msgbus.system.reply",
        }
    }
}

impl std::fmt::Display for Queue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

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

    pub async fn request(&self, req: Request) -> Result<impl Stream<Item = Result<Message>>> {
        let mut conn = self.get_connection().await?;
        conn.rpush(Queue::Local.as_ref(), req.body())
            .await
            .context("unable to send your message")?;
        let response = Response::new(self.pool.clone()).response(Queue::Reply.as_ref());

        Ok(response)
    }
}
