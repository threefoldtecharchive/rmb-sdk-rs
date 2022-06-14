use anyhow::{Context, Result};
use workers::WorkerPool;

use super::{work_runner::WorkRunner, Handler, Router};
use crate::msg::Message;
use bb8_redis::{
    bb8::{Pool, PooledConnection},
    redis::{AsyncCommands, RedisResult},
    RedisConnectionManager,
};
use std::iter::Iterator;
use std::{collections::HashMap, sync::Arc};

pub struct Module {
    modules: HashMap<String, Module>,
    handlers: HashMap<String, Handler>,
}

impl Module {
    fn new() -> Self {
        Self {
            modules: HashMap::default(),
            handlers: HashMap::default(),
        }
    }

    pub fn lookup<S: AsRef<str>>(&self, path: S) -> Option<&Handler> {
        let parts: Vec<&str> = path.as_ref().split(".").collect();

        self.lookup_parts(&parts)
    }

    fn lookup_parts(&self, parts: &[&str]) -> Option<&Handler> {
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
    pub fn functions(&self) -> Vec<String> {
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

    fn handle<S: Into<String>>(&mut self, name: S, handler: super::Handler) -> &mut Self {
        let name = name.into();
        assert!(!name.contains("."), "module name cannot contain a dot");
        if self.handlers.contains_key(&name) {
            panic!("double registration of same function: {}", name);
        }

        self.handlers.insert(name, handler);
        self
    }
}

pub struct Server {
    pool: Pool<RedisConnectionManager>,
    runner: WorkRunner,
    workers: usize,
}

impl Router for Server {
    type Module = Module;

    fn module<S: Into<String>>(&mut self, name: S) -> &mut Self::Module {
        self.runner.module(name)
    }

    fn handle<S: Into<String>>(&mut self, name: S, handler: Handler) -> &mut Self {
        self.runner.handle(name, handler);
        self
    }
}

impl Server {
    pub fn new(pool: Pool<RedisConnectionManager>, workers: usize) -> Self {
        let runner = WorkRunner::new(pool.clone(), Module::new());
        Self {
            pool,
            runner,
            workers,
        }
    }

    pub fn functions(&self) -> Vec<String> {
        self.runner.functions()
    }

    pub fn lookup<S: AsRef<str>>(&self, cmd: S) -> Option<&Handler> {
        self.runner.lookup(cmd)
    }

    async fn get_connection(
        pool: &Pool<RedisConnectionManager>,
    ) -> Result<PooledConnection<'_, RedisConnectionManager>> {
        let conn = pool
            .get()
            .await
            .context("unable to retrieve a redis connection from the pool")?;

        Ok(conn)
    }

    pub async fn run(self) -> Result<()> {
        let keys = self.functions();
        let mut workers = WorkerPool::new(Arc::new(self.runner), self.workers);
        loop {
            let worker_handler = workers.get().await;
            let mut conn = Self::get_connection(&self.pool).await?;

            let msg_res: RedisResult<(String, Message)> = conn.brpop(&keys, 0).await;

            let (command, msg) = match msg_res {
                Ok(msg) => msg,
                Err(err) => {
                    log::debug!("redis error happened because of '{}'", err);
                    continue;
                }
            };

            if let Err(err) = worker_handler.send((command, msg)) {
                log::debug!("can not send job to worker because of '{}'", err);
            }

            // todo:
            // - handler is not async. hence
            //  - you can only process single command at a time
            //  - you cannot control number of workers.
            //  - handler code can't do async work
        }
    }
}
