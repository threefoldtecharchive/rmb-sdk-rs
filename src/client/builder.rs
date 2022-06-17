use crate::protocol::Message;
use crate::util;
use serde::ser::Serialize;
use std::time::Duration;

/// Request object
#[derive(Debug, Clone)]
pub struct Request {
    msg: Message,
}

impl Request {
    /// Create a new request to given command
    pub fn new<C: Into<String>>(cmd: C) -> Self {
        let mut msg = Message::default();
        msg.reply = util::unique_id().to_string();
        msg.command = cmd.into();

        Self { msg }
    }

    /// add a new destination to the message
    pub fn destination(mut self, destination: u32) -> Self {
        let mut destination = vec![destination];
        self.msg.destination.append(&mut destination);

        self
    }

    /// add all destinations at once
    pub fn destinations<T: Iterator<Item = u32>>(mut self, destinations: T) -> Self {
        self.msg.destination.extend(destinations);

        self
    }

    /// set request expiration time
    pub fn expiration(mut self, exp: Duration) -> Self {
        self.msg.expiration = exp.as_secs();
        self
    }

    /// set command arguments to given object (request body)
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

impl From<Message> for Request {
    fn from(msg: Message) -> Self {
        Self { msg }
    }
}
