mod server;
mod work_runner;
use anyhow::Result;

pub use handler::handler;
pub use server::{Module, Server};

#[derive(Debug)]
pub struct HandlerInput {
    pub data: Vec<u8>,
    pub schema: String,
}

#[derive(Debug)]
pub struct HandlerOutput {
    pub data: Vec<u8>,
    pub schema: String,
}

#[async_trait::async_trait]
pub trait Handler<D>: Send + Sync + 'static
where
    D: 'static,
{
    async fn call(&self, data: D, input: HandlerInput) -> Result<HandlerOutput>;
}

pub trait Router<D>
where
    D: 'static,
{
    type Module: Router<D>;

    fn module<S: Into<String>>(&mut self, name: S) -> &mut Self::Module;
    fn handle<S: Into<String>>(&mut self, name: S, handler: impl Handler<D>) -> &mut Self;
}
