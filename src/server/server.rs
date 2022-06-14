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
use tokio::time::{sleep, Duration};

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
    root: Module,
    workers: usize,
}

impl Router for Server {
    type Module = Module;

    fn module<S: Into<String>>(&mut self, name: S) -> &mut Self::Module {
        self.root.module(name)
    }

    fn handle<S: Into<String>>(&mut self, name: S, handler: Handler) -> &mut Self {
        self.root.handle(name, handler);
        self
    }
}

impl Server {
    pub fn new(pool: Pool<RedisConnectionManager>, workers: usize) -> Self {
        Self {
            pool,
            root: Module::new(),
            workers,
        }
    }

    pub fn lookup<S: AsRef<str>>(&self, path: S) -> Option<&Handler> {
        self.root.lookup(path)
    }

    pub async fn run(self) -> Result<()> {
        let pool = self.pool;
        let keys = self.root.functions();
        let runner = WorkRunner::new(pool.clone(), self.root);
        let mut workers = WorkerPool::new(Arc::new(runner), self.workers);
        loop {
            let worker_handler = workers.get().await;
            let mut conn = match pool.get().await {
                Ok(conn) => conn,
                Err(err) => {
                    log::error!("failed to get redis connection: {}", err);
                    sleep(Duration::from_secs(2)).await;
                    continue;
                }
            };

            let (command, message): (String, Message) = match conn.brpop(&keys, 0).await {
                Ok(resp) => resp,
                Err(err) => {
                    log::error!("failed to get next command: {}", err);
                    sleep(Duration::from_secs(2)).await;
                    continue;
                }
            };

            if let Err(err) = worker_handler.send((command, message)) {
                log::debug!("can not send job to worker because of '{}'", err);
            }
        }
    }
}
