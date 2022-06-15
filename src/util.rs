use std::time::SystemTime;

pub fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn unique_id() -> impl ToString {
    uuid::Uuid::new_v4().to_string()
}
