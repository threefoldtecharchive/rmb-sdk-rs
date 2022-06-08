use std::time::SystemTime;

pub fn timestamp() -> usize {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
}

pub fn unique_id() -> impl ToString {
    uuid::Uuid::new_v4().to_string()
}

pub enum Queue {
    Local,
    Reply,
}

impl AsRef<str> for Queue {
    fn as_ref(&self) -> &str {
        match self {
            Queue::Local => "msgbus.system.local",
            Queue::Reply => "msgbus.system.reply",
        }
    }
}

impl std::fmt::Display for Queue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}
