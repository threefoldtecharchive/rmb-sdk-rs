
use anyhow::{Context, Result};
use bb8_redis::{
    bb8::{Pool, PooledConnection},
    redis::{AsyncCommands, RedisResult},
    RedisConnectionManager,
};
use std::{collections::HashMap, time::Duration};

use crate::{msg::Message, util};


type Cmd = String;

pub struct CmdArgs {
    pub data: String,
    pub schema: String,
}

pub struct Server<F>
where
    F: Fn(CmdArgs),
{
    pool: Pool<RedisConnectionManager>,
    channels: HashMap<Cmd, F>,
}

impl<F> Server<F>
where
    F: Fn(CmdArgs) + Clone,
{
    pub fn new(pool: Pool<RedisConnectionManager>) -> Self {
        let channels = HashMap::new();
        Self { pool, channels }
    }

    async fn get_connection(&self) -> Result<PooledConnection<'_, RedisConnectionManager>> {
        let conn = self
            .pool
            .get()
            .await
            .context("unable to retrieve a redis connection from the pool")?;

        Ok(conn)
    }

    pub fn add_channel(mut self, cmd: Cmd, handler: F) -> Self {
        let key = format!("msgbus.{}", cmd);
        self.channels.insert(key, handler);
        self
    }

    pub async fn exec(self) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let keys: Vec<String> = self.channels.clone().into_keys().by_ref().collect();
        let timeout = util::timestamp() + Duration::from_secs(100).as_secs() as usize;
        loop {
                let msg_res: RedisResult<(String, Message)> = conn.brpop(&keys, timeout).await;

                if let Ok((list, msg)) = msg_res {
                    // decode an encoded data by me which will not panic
                    let data = base64::decode(msg.data).unwrap();
                    let data = String::from_utf8(data).unwrap();
                    let handler = self.channels.get(&list).unwrap();
                    let cmd_args = CmdArgs { data, schema: msg.schema };
                    let reply = handler(cmd_args);
                }
        }
    }

    /// Send a reply to a previously command received
    async fn reply<R: AsRef<str>>(&self, reply: R, msg: &Message) -> Result<()> {
        let mut conn = self.get_connection().await?;

        conn.rpush(reply.as_ref(), msg)
            .await
            .context("unable to send your reply")?;

        Ok(())
    }

    // pub fn reply() {}
}
