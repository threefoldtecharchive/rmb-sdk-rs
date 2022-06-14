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
pub trait Handler: Send + Sync + 'static {
    async fn call(&self, input: HandlerInput) -> Result<HandlerOutput>;
}

pub trait Router {
    type Module: Router;

    fn module<S: Into<String>>(&mut self, name: S) -> &mut Self::Module;
    fn handle<S: Into<String>>(&mut self, name: S, handler: impl Handler) -> &mut Self;
}
