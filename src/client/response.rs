use crate::msg::Message;
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
    deadline: usize,
}

impl Response {
    pub fn new(
        pool: Pool<RedisConnectionManager>,
        ret_queue: String,
        response_num: usize,
        deadline: usize,
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
        let msg: Message = {
            let mut conn = self.get_connection().await.unwrap();
            conn.brpop(&self.ret_queue, timeout)
                .await
                .context("failed to get a response message")?
        };
        self.response_num -= 1;

        let payload = if let Some(err) = msg.error {
            Err(ResponseErr::Remote(err))
        } else {
            let data =
                base64::decode(msg.data).with_context(|| "can not decode the received response");

            if let Err(err) = data {
                Err(ResponseErr::Protocol(err.to_string()))
            } else {
                Ok(Some(data.unwrap()))
            }
        };

        Ok(Some(ResponseBody {
            payload,
            schema: msg.schema,
        }))
    }
}

type Payload = Result<Option<Vec<u8>>, ResponseErr>;
pub enum ResponseErr {
    Protocol(String),
    Remote(String),
}
pub struct ResponseBody {
    pub payload: Payload,
    pub schema: String,
}
