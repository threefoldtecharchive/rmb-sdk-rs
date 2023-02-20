use crate::{protocol::IncomingResponse, util};
use serde::Deserialize;

use anyhow::{Context, Result};
use bb8_redis::{
    bb8::{Pool, PooledConnection},
    redis::AsyncCommands,
    RedisConnectionManager,
};

/// Response object
pub struct Call {
    pool: Pool<RedisConnectionManager>,
    ret_queue: String,
    response_num: usize,
    deadline: u64,
}

impl Call {
    pub(crate) fn new(
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

    /// wait for next response for this request. Usually the caller
    /// need to wait in a loop. None is returned if all expected responses
    /// has been received or expiration time of message has been exceeded.
    pub async fn get(&mut self) -> Result<Option<Return>> {
        let timeout = match self.deadline.checked_sub(util::timestamp()) {
            Some(timeout) => timeout,
            None => return Ok(None),
        };

        if self.response_num == 0 {
            return Ok(None);
        }

        let msg: Option<IncomingResponse> = {
            let mut conn = self.get_connection().await?;

            let res: Option<(String, IncomingResponse)> = conn
                .brpop(&self.ret_queue, timeout as usize)
                .await
                .context("failed to get a response message")?;

            res.map(|(_, msg)| msg)
        };

        self.response_num -= 1;
        Ok(msg.map(|m| m.into()))
    }
}

type Payload = Result<Vec<u8>, ResponseErr>;

#[derive(thiserror::Error, Debug, Clone)]
pub enum ResponseErr {
    #[error("protocol error: {0}")]
    Protocol(String),
    #[error("remote error: {0}")]
    Remote(String),
}

#[derive(Debug)]
pub struct Return {
    pub source: String,
    pub payload: Payload,
    pub schema: Option<String>,
}

impl From<IncomingResponse> for Return {
    fn from(msg: IncomingResponse) -> Self {
        let payload = match msg.error {
            Some(err) => Err(ResponseErr::Remote(err.message)),
            None => match base64::decode(msg.data) {
                Ok(data) => Ok(data),
                Err(err) => Err(ResponseErr::Protocol(err.to_string())),
            },
        };

        Return {
            source: msg.source,
            schema: msg.schema,
            payload,
        }
    }
}

impl Return {
    /// auto decode the returned response to concrete types.
    pub fn outputs<'a, T>(&'a self) -> Result<T, ResponseErr>
    where
        T: Deserialize<'a>,
    {
        match &self.payload {
            Ok(data) => {
                let obj = match self.schema {
                    Some(ref schema) if schema == "application/json" => {
                        serde_json::from_slice(data)
                            .map_err(|e| ResponseErr::Remote(format!("schema error {}", e)))?
                    }
                    _ => return Err(ResponseErr::Remote("not supported encoding type".into())),
                };

                Ok(obj)
            }
            Err(err) => Err(err.clone()),
        }
    }
}
