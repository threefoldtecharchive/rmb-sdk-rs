use crate::msg::Message;
use serde::ser::Serialize;

#[derive(Debug)]
pub struct Request {
    msg: Message,
}

impl Request {
    pub fn new<C: Into<String>>(cmd: C) -> Self {
        let mut msg = Message::default();
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

    pub fn expiration(mut self, exp: usize) -> Self {
        self.msg.expiration = exp;
        self
    }

    pub fn args<A: AsRef<[u8]>>(mut self, args: A) -> Self {
        let data = base64::encode(args.as_ref());
        self.msg.data = data;
        self
    }

    pub fn body(self) -> Message {
        self.msg
    }
}

impl Serialize for Request {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.msg.serialize(serializer)
    }
}
