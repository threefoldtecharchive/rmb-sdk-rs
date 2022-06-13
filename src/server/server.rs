use anyhow::{Context, Result};

use super::{Handler, HandlerInput, Router};
use crate::msg::Message;
use bb8_redis::{
    bb8::{Pool, PooledConnection},
    redis::{AsyncCommands, RedisResult},
    RedisConnectionManager,
};
use std::collections::HashMap;
use std::iter::Iterator;

pub struct Module {
    modules: HashMap<String, Module>,
    handlers: HashMap<String, Box<dyn Handler>>,
}

impl Module {
    fn new() -> Self {
        Self {
            modules: HashMap::default(),
            handlers: HashMap::default(),
        }
    }

    pub fn lookup<S: AsRef<str>>(&self, path: S) -> Option<&Box<dyn Handler>> {
        let parts: Vec<&str> = path.as_ref().split(".").collect();

        self.lookup_parts(&parts)
    }

    fn lookup_parts(&self, parts: &[&str]) -> Option<&Box<dyn Handler>> {
        match parts.len() {
            0 => None,
            1 => self.handlers.get(parts[0]),
            _ => match self.modules.get(parts[0]) {
                None => None,
                Some(sub) => sub.lookup_parts(&parts[1..]),
            },
        }
    }

    /// return all registered keys
    fn functions(&self) -> Vec<String> {
        // todo!: implement with iterators instead

        let mut fns: Vec<String> = self.handlers.keys().map(|k| k.to_owned()).collect();
        for (name, module) in self.modules.iter() {
            fns.extend(module.functions().iter().map(|k| format!("{}.{}", name, k)))
        }

        fns
    }
}

impl Router for Module {
    type Module = Module;

    fn module<S: Into<String>>(&mut self, name: S) -> &mut Module {
        let name = name.into();
        assert!(!name.contains("."), "module name cannot contain a dot");
        self.modules
            .entry(name.clone())
            .or_insert_with(|| Module::new())
    }

    fn handle<S: Into<String>>(&mut self, name: S, handler: impl Handler) -> &mut Self {
        let name = name.into();
        assert!(!name.contains("."), "module name cannot contain a dot");
        if self.handlers.contains_key(&name) {
            panic!("double registration of same function: {}", name);
        }

        self.handlers.insert(name, Box::new(handler));
        self
    }
}

pub struct Server {
    root: Module,
    pool: Pool<RedisConnectionManager>,
}

impl Router for Server {
    type Module = Module;

    fn module<S: Into<String>>(&mut self, name: S) -> &mut Self::Module {
        self.root.module(name)
    }

    fn handle<S: Into<String>>(&mut self, name: S, handler: impl Handler) -> &mut Self {
        self.root.handle(name, handler);
        self
    }
}

impl Server {
    pub fn new(pool: Pool<RedisConnectionManager>) -> Self {
        Self {
            root: Module::new(),
            pool: pool,
        }
    }

    pub fn functions(&self) -> Vec<String> {
        self.root.functions()
    }

    pub fn lookup<S: AsRef<str>>(&self, cmd: S) -> Option<&Box<dyn Handler>> {
        self.root.lookup(cmd)
    }

    async fn get_connection(&self) -> Result<PooledConnection<'_, RedisConnectionManager>> {
        let conn = self
            .pool
            .get()
            .await
            .context("unable to retrieve a redis connection from the pool")?;

        Ok(conn)
    }

    pub async fn run(self) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let keys = self.root.functions();
        loop {
            let msg_res: RedisResult<(String, Message)> = conn.brpop(&keys, 0).await;

            if let Ok((command, msg)) = msg_res {
                // decode an encoded data by me which will not panic
                let data = base64::decode(msg.data).unwrap(); // <- not safe
                let handler = self
                    .root
                    .lookup(command)
                    .context("handler not found this should never happen")
                    .unwrap();

                let _out = handler
                    .call(HandlerInput {
                        data: data,
                        schema: msg.schema,
                    })
                    .await;

                // todo:
                // - handler is not async. hence
                //  - you can only process single command at a time
                //  - you cannot control number of workers.
                //  - handler code can't do async work
            }
        }
    }
}
