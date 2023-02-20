use crate::util;
use bb8_redis::redis;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Request {
    #[serde(rename = "ver")]
    version: usize,
    #[serde(rename = "ref")]
    reference: Option<String>,
    #[serde(rename = "cmd")]
    command: String,
    #[serde(rename = "exp")]
    expiration: u64,
    #[serde(rename = "dat")]
    data: String,
    #[serde(rename = "tag")]
    tags: Option<String>,
    #[serde(rename = "dst")]
    destinations: Vec<u32>,
    #[serde(rename = "ret")]
    reply_to: String,
    #[serde(rename = "shm")]
    schema: Option<String>,
    #[serde(rename = "now")]
    timestamp: u64,
}

impl Request {
    pub fn builder<C: Into<String>>(cmd: C) -> RequestBuilder {
        RequestBuilder::new(cmd.into())
    }

    pub(crate) fn deadline(&self) -> u64 {
        self.timestamp + self.expiration
    }

    pub(crate) fn reply(&self) -> &str {
        &self.reply_to
    }

    pub(crate) fn destinations(&self) -> &[u32] {
        &self.destinations
    }
}

impl redis::ToRedisArgs for Request {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        let bytes = serde_json::to_vec(self).expect("failed to json encode message");
        out.write_arg(&bytes);
    }
}

impl redis::FromRedisValue for Request {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        if let redis::Value::Data(data) = v {
            serde_json::from_slice(data).map_err(|e| {
                redis::RedisError::from((
                    redis::ErrorKind::TypeError,
                    "cannot decode a message from json {}",
                    e.to_string(),
                ))
            })
        } else {
            Err(redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "expected a data type from redis",
            )))
        }
    }
}

pub struct RequestBuilder {
    cmd: String,
    destination: Vec<u32>,
    expiration: u64,
    data: String,
}

impl RequestBuilder {
    fn new(cmd: String) -> RequestBuilder {
        Self {
            cmd: cmd,
            destination: Vec::default(),
            expiration: 60,
            data: String::default(),
        }
    }

    /// add a new destination to the message
    pub fn destination(mut self, destination: u32) -> Self {
        let mut destination = vec![destination];
        self.destination.append(&mut destination);

        self
    }

    /// add all destinations at once
    pub fn destinations<T: Iterator<Item = u32>>(mut self, destinations: T) -> Self {
        self.destination.extend(destinations);

        self
    }

    /// set request expiration time
    pub fn expiration(mut self, exp: Duration) -> Self {
        self.expiration = exp.as_secs();
        self
    }

    /// set command arguments to given object (request body)
    pub fn args<A: Serialize>(mut self, args: A) -> Self {
        let body = serde_json::to_vec(&args).unwrap();
        let data = base64::encode(&body);
        self.data = data;
        self
    }
}

impl From<RequestBuilder> for Request {
    fn from(value: RequestBuilder) -> Self {
        let id = util::unique_id().to_string();
        Self {
            version: 1,
            command: value.cmd,
            data: value.data,
            destinations: value.destination,
            expiration: value.expiration,
            reference: Some(id.clone()),
            reply_to: id,
            schema: Some("application/json".into()),
            tags: None,
            timestamp: util::timestamp(),
        }
    }
}
