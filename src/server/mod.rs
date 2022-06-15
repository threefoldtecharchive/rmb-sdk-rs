mod server;
mod work_runner;
use anyhow::{Context, Result};
pub use handler::handler;
use serde::{Deserialize, Serialize};
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

pub fn inputs<'a, T: Deserialize<'a>>(input: &'a HandlerInput) -> Result<T> {
    let obj = match input.schema.as_str() {
        "" | "application/json" => {
            serde_json::from_slice(&input.data).context("failed to decode object")?
        }
        _ => anyhow::bail!("not supported encoding type"),
    };

    Ok(obj)
}

pub fn output<T: Serialize>(output: T) -> Result<HandlerOutput> {
    Ok(HandlerOutput {
        schema: "application/json".into(),
        data: serde_json::to_vec(&output).context("failed to encode object")?,
    })
}
