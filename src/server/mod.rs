mod server;
mod work_runner;
use anyhow::{Context, Result};
pub use handler::handler;
use serde::{Deserialize, Serialize};
pub use server::{Module, Server};

/// HandlerInput holds request body.
#[derive(Debug)]
pub struct HandlerInput<'a> {
    pub source: &'a str,
    pub data: Vec<u8>,
    pub schema: Option<&'a str>,
}

/// HandlerOutput holds response body
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

impl<'a> HandlerInput<'a> {
    pub fn inputs<'s, T: Deserialize<'s>>(&'s self) -> Result<T> {
        let obj = match self.schema {
            Some(schema) if schema == "application/json" => {
                serde_json::from_slice(&self.data).context("failed to decode object")?
            }
            _ => anyhow::bail!("not supported schema"),
        };

        Ok(obj)
    }
}

impl HandlerOutput {
    pub fn from<T: Serialize>(o: T) -> Result<Self> {
        Ok(HandlerOutput {
            schema: "application/json".into(),
            data: serde_json::to_vec(&o).context("failed to encode object")?,
        })
    }
}
