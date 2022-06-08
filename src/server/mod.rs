mod module;

use anyhow::{Context, Result};
use bb8_redis::{
    bb8::{Pool, PooledConnection},
    redis::{AsyncCommands, RedisResult},
    RedisConnectionManager,
};
use module::*;
use std::time::Duration;

use crate::{msg::Message, util};

// type Cmd = String;
type Handler<P, R> = fn(P) -> R;

pub struct CmdArgs {
    pub data: String,
    pub schema: String,
}

pub struct Server<P, R> {
    pool: Pool<RedisConnectionManager>,
    // channels: HashMap<Cmd, Handler<P, R>>,
    name: String,
    module: ServiceModule<P, R>,
}

impl<P, R> Router<P, R> for Server<P, R> {
    fn handle<S: Into<String>>(&mut self, submod: S, handler: Option<Handler<P, R>>) -> &Self {
        self.module.handle(submod.into(), handler);
        self
    }
}

impl<P, R> Server<P, R> {
    pub fn new<M: Into<String>>(pool: Pool<RedisConnectionManager>, modname: M) -> Self {
        Self {
            pool,
            name: modname.into(),
            module: ServiceModule::new(None),
        }
    }

    pub fn submodule<S: Into<String>>(&mut self, submod: S) -> &impl Router<P, R> {
        self.module.handle(submod.into(), None)
    }

    async fn get_connection(&self) -> Result<PooledConnection<'_, RedisConnectionManager>> {
        let conn = self
            .pool
            .get()
            .await
            .context("unable to retrieve a redis connection from the pool")?;

        Ok(conn)
    }

    pub async fn exec(self) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let keys = self.module.form_keys(&self.name).unwrap();
        let timeout = util::timestamp() + Duration::from_secs(100).as_secs() as usize;
        loop {
            let msg_res: RedisResult<(String, Message)> = conn.brpop(&keys, timeout).await;

            if let Ok((list, msg)) = msg_res {
                // decode an encoded data by me which will not panic
                let data = base64::decode(msg.data).unwrap();
                let data = String::from_utf8(data).unwrap();
                let handler = self.module.get_handler(&self.name, &list);
                if let Some(_handler) = handler {
                    let _cmd_args = CmdArgs {
                        data,
                        schema: msg.schema,
                    };
                    // TODO
                    // - call handler
                    // - get result
                    // let reply = handler(cmd_args);
                }
            }
        }
    }

    // Send a reply to a previously command received
    // async fn reply<R: AsRef<str>>(&self, reply: R, msg: &Message) -> Result<()> {
    //     let mut conn = self.get_connection().await?;

    //     conn.rpush(reply.as_ref(), msg)
    //         .await
    //         .context("unable to send your reply")?;

    //     Ok(())
    // }

    // pub fn reply() {}
}
