use anyhow::Result;
use workers::WorkerPool;

use super::{work_runner::WorkRunner, Handler, Router};
use crate::protocol::IncomingRequest;
use bb8_redis::{bb8::Pool, redis::AsyncCommands, RedisConnectionManager};
use std::iter::Iterator;
use std::{collections::HashMap, sync::Arc};
use tokio::time::{sleep, Duration};

pub struct Module<D> {
    modules: HashMap<String, Module<D>>,
    handlers: HashMap<String, Box<dyn Handler<D>>>,
}

impl<D> Module<D> {
    fn new() -> Self {
        Self {
            modules: HashMap::default(),
            handlers: HashMap::default(),
        }
    }

    pub fn lookup<S: AsRef<str>>(&self, path: S) -> Option<&Box<dyn Handler<D>>> {
        let parts: Vec<&str> = path.as_ref().split(".").collect();

        self.lookup_parts(&parts)
    }

    fn lookup_parts(&self, parts: &[&str]) -> Option<&Box<dyn Handler<D>>> {
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

impl<D> Router<D> for Module<D>
where
    D: 'static,
{
    type Module = Module<D>;

    fn module<S: Into<String>>(&mut self, name: S) -> &mut Module<D> {
        let name = name.into();
        assert!(!name.contains("."), "module name cannot contain a dot");
        self.modules
            .entry(name.clone())
            .or_insert_with(|| Module::new())
    }

    fn handle<S: Into<String>>(&mut self, name: S, handler: impl Handler<D>) -> &mut Self {
        let name = name.into();
        assert!(!name.contains("."), "module name cannot contain a dot");
        if self.handlers.contains_key(&name) {
            panic!("double registration of same function: {}", name);
        }

        self.handlers.insert(name, Box::new(handler));
        self
    }
}

pub struct Server<D> {
    pool: Pool<RedisConnectionManager>,
    root: Module<D>,
    workers: usize,
    data: D,
}

impl<D> Router<D> for Server<D>
where
    D: 'static,
{
    type Module = Module<D>;

    fn module<S: Into<String>>(&mut self, name: S) -> &mut Self::Module {
        self.root.module(name)
    }

    fn handle<S: Into<String>>(&mut self, name: S, handler: impl Handler<D>) -> &mut Self {
        self.root.handle(name, handler);
        self
    }
}

impl<D> Server<D>
where
    D: Clone + Send + Sync + 'static,
{
    pub fn new(data: D, pool: Pool<RedisConnectionManager>, workers: usize) -> Self {
        Self {
            pool,
            root: Module::new(),
            data,
            workers,
        }
    }

    pub fn lookup<S: AsRef<str>>(&self, path: S) -> Option<&Box<dyn Handler<D>>> {
        self.root.lookup(path)
    }

    /// start this server instance
    pub async fn run(self) -> Result<()> {
        let pool = self.pool;
        let keys: Vec<String> = self
            .root
            .functions()
            .into_iter()
            .map(|k| format!("msgbus.{}", k))
            .collect();

        let runner = WorkRunner::new(pool.clone(), self.data, self.root);
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

            let (command, request): (String, IncomingRequest) = match conn.brpop(&keys, 0).await {
                Ok(resp) => resp,
                Err(err) => {
                    log::error!("failed to get next command: {}", err);
                    sleep(Duration::from_secs(2)).await;
                    continue;
                }
            };

            let command: String = command.strip_prefix("msgbus.").unwrap_or("").into();
            if let Err(err) = worker_handler.send((command, request)) {
                log::debug!("can not send job to worker because of '{}'", err);
            }
        }
    }
}
