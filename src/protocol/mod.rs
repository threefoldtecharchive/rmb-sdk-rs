use bb8_redis::redis;
use serde::{Deserialize, Serialize};

pub enum Queue {
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IncomingRequest {
    #[serde(rename = "ver")]
    pub version: usize,
    #[serde(rename = "ref")]
    pub reference: Option<String>,
    #[serde(rename = "src")]
    pub source: String,
    #[serde(rename = "cmd")]
    pub command: String,
    #[serde(rename = "exp")]
    pub expiration: u64,
    #[serde(rename = "dat")]
    pub data: String,
    #[serde(rename = "tag")]
    pub tags: Option<String>,
    #[serde(rename = "ret")]
    pub reply_to: String,
    #[serde(rename = "shm")]
    pub schema: Option<String>,
    #[serde(rename = "now")]
    pub timestamp: u64,
}

impl redis::ToRedisArgs for IncomingRequest {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        let bytes = serde_json::to_vec(self).expect("failed to json encode message");
        out.write_arg(&bytes);
    }
}

impl redis::FromRedisValue for IncomingRequest {
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Error {
    pub code: u32,
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OutgoingResponse {
    #[serde(rename = "ver")]
    pub version: usize,
    #[serde(rename = "ref")]
    pub reference: Option<String>,
    #[serde(rename = "dat")]
    pub data: String,
    #[serde(rename = "dst")]
    pub destination: String,
    #[serde(rename = "shm")]
    pub schema: Option<String>,
    #[serde(rename = "now")]
    pub timestamp: u64,
    #[serde(rename = "err")]
    pub error: Option<Error>,
}

impl redis::ToRedisArgs for OutgoingResponse {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        let bytes = serde_json::to_vec(self).expect("failed to json encode message");
        out.write_arg(&bytes);
    }
}

impl redis::FromRedisValue for OutgoingResponse {
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IncomingResponse {
    #[serde(rename = "ver")]
    pub version: usize,
    #[serde(rename = "ref")]
    pub reference: Option<String>,
    #[serde(rename = "dat")]
    pub data: String,
    #[serde(rename = "src")]
    pub source: String,
    #[serde(rename = "shm")]
    pub schema: Option<String>,
    #[serde(rename = "now")]
    pub timestamp: u64,
    #[serde(rename = "err")]
    pub error: Option<Error>,
}

impl redis::ToRedisArgs for IncomingResponse {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        let bytes = serde_json::to_vec(self).expect("failed to json encode message");
        out.write_arg(&bytes);
    }
}

impl redis::FromRedisValue for IncomingResponse {
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
