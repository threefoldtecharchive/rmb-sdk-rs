use crate::msg::Message;
use crate::util;
use serde::ser::Serialize;

#[derive(Debug)]
pub struct Request {
    msg: Message,
}

impl Request {
    pub fn new<C: Into<String>>(cmd: C) -> Self {
        let mut msg = Message::default();
        msg.reply = util::unique_id().to_string();
        msg.now = util::timestamp() as u64;
        msg.command = cmd.into();

        Self { msg }
    }

    pub fn update_timestamp(mut self) -> Self {
        self.msg.now = util::timestamp() as u64;
        self
    }

    pub fn get_ret(&self) -> &String {
        &self.msg.reply
    }

    pub fn calc_deadline(&self) -> usize {
        self.msg.now as usize + self.msg.expiration
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

    pub fn destinations_len(&self) -> usize {
        self.msg.destination.len()
    }

    pub fn args<A: AsRef<[u8]>>(mut self, args: A) -> Self {
        let data = base64::encode(args.as_ref());
        self.msg.data = data;
        self
    }

    pub fn body(&self) -> Message {
        self.msg.clone()
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
