use std::time::SystemTime;

use crate::msg::Message;
use crate::util;

use anyhow::{Context, Ok, Result};
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

    pub async fn get(&mut self) -> Option<Result<ResponseBody>> {
        let timeout = util::timestamp() - self.deadline;

        if timeout == 0 || self.response_num == 0 {
            return None;
        }
        let msg: Result<Message> = {
            let mut conn = self.get_connection().await.unwrap();
            conn.brpop(&self.ret_queue, timeout)
                .await
                .context("failed to get a response message")
                .map_err(|err| anyhow::anyhow!("{}", err))
        };
        self.response_num -= 1;

        if msg.is_err() {
            return Some(Err(msg.err().unwrap()));
        }

        let data = base64::decode(msg.unwrap().data);

        let data = if let Err(err) = data {
            return Some(Err(anyhow::Error::from(err)));
        } else {
            data.unwrap()
        };

        Some(Ok(ResponseBody(String::from_utf8(data).unwrap())))
    }
}

pub struct ResponseBody(String);

impl ResponseBody {
    pub fn body(self) -> impl ToString {
        self.0
    }
}
