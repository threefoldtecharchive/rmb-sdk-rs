mod server;
mod work_runner;

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

type Handler = fn(HandlerInput) -> HandlerOutput;

pub trait Router {
    type Module: Router;

    fn module<S: Into<String>>(&mut self, name: S) -> &mut Self::Module;
    fn handle<S: Into<String>>(&mut self, name: S, handler: Handler) -> &mut Self;
}
