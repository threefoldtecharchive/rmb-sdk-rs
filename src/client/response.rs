use crate::protocol::Message;
use crate::util;

use anyhow::{Context, Result};
use bb8_redis::{
    bb8::{Pool, PooledConnection},
    redis::AsyncCommands,
    RedisConnectionManager,
};

pub struct Response {
    pool: Pool<RedisConnectionManager>,
    ret_queue: String,
    response_num: usize,
    deadline: u64,
}

impl Response {
    pub fn new(
        pool: Pool<RedisConnectionManager>,
        ret_queue: String,
        response_num: usize,
        deadline: u64,
    ) -> Self {
        Self {
            pool,
            ret_queue,
            response_num,
            deadline,
        }
    }

    async fn get_connection(&self) -> Result<PooledConnection<'_, RedisConnectionManager>> {
        let conn = self
            .pool
            .get()
            .await
            .context("unable to retrieve a redis connection from the pool")?;

        Ok(conn)
    }

    pub async fn get(&mut self) -> Result<Option<ResponseBody>> {
        let timeout = util::timestamp() - self.deadline;

        if timeout == 0 || self.response_num == 0 {
            return Ok(None);
        }

        let msg: Option<Message> = {
            let mut conn = self.get_connection().await?;

            conn.brpop(&self.ret_queue, timeout as usize)
                .await
                .context("failed to get a response message")?
        };

        self.response_num -= 1;
        Ok(msg.map(|m| m.into()))
    }
}

type Payload = Result<Vec<u8>, ResponseErr>;

#[derive(Debug)]
pub enum ResponseErr {
    Protocol(String),
    Remote(String),
}

#[derive(Debug)]
pub struct ResponseBody {
    pub payload: Payload,
    pub schema: String,
}

impl From<Message> for ResponseBody {
    fn from(msg: Message) -> Self {
        let payload = match msg.error {
            Some(err) => Err(ResponseErr::Remote(err)),
            None => match base64::decode(msg.data) {
                Ok(data) => Ok(data),
                Err(err) => Err(ResponseErr::Protocol(err.to_string())),
            },
        };

        ResponseBody {
            payload,
            schema: msg.schema,
        }
    }
}
