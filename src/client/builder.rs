use crate::protocol::Message;
use crate::util;
use serde::ser::Serialize;
use std::time::Duration;

#[derive(Debug)]
pub struct Request {
    msg: Message,
}

impl Request {
    pub fn new<C: Into<String>>(cmd: C) -> Self {
        let mut msg = Message::default();
        msg.reply = util::unique_id().to_string();
        msg.command = cmd.into();

        Self { msg }
    }

    pub fn destination(mut self, destination: u32) -> Self {
        let mut destination = vec![destination];
        self.msg.destination.append(&mut destination);

        self
    }

    pub fn destinations(mut self, destinations: Vec<u32>) -> Self {
        let mut destinations = destinations;
        self.msg.destination.append(&mut destinations);

        self
    }

    pub fn expiration(mut self, exp: Duration) -> Self {
        self.msg.expiration = exp.as_secs();
        self
    }

    pub fn destinations_len(&self) -> usize {
        self.msg.destination.len()
    }

    pub fn args<A: Serialize>(mut self, args: A) -> Self {
        let body = serde_json::to_vec(&args).unwrap();
        let data = base64::encode(&body);
        self.msg.data = data;
        self
    }
}

impl From<Request> for Message {
    fn from(req: Request) -> Self {
        req.msg
    }
}
