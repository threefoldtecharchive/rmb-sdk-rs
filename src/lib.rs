mod client;
mod server;
mod msg;

pub use client::{Client, request::Request, response::Response};
pub use server::Server;


// mod msg;
// use anyhow::{Context, Result};
// use bb8_redis::{
//     bb8::{Pool, PooledConnection},
//     redis::AsyncCommands,
//     RedisConnectionManager,
// };
// use msg::Message;

// enum Queue {
//     Local,
//     Reply,
// }

// impl AsRef<str> for Queue {
//     fn as_ref(&self) -> &str {
//         match self {
//             Queue::Local => "msgbus.system.local",
//             Queue::Reply => "msgbus.system.reply",
//         }
//     }
// }

// impl std::fmt::Display for Queue {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.as_ref())
//     }
// }

// pub struct RmbClient {
//     pool: Pool<RedisConnectionManager>,
// }

// impl RmbClient {
//     pub fn new(pool: Pool<RedisConnectionManager>) -> Self {
//         Self { pool }
//     }

//     async fn get_connection(&self) -> Result<PooledConnection<'_, RedisConnectionManager>> {
//         let conn = self
//             .pool
//             .get()
//             .await
//             .context("unable to retrieve a redis connection from the pool")?;

//         Ok(conn)
//     }

//     /// Send [command] Message to a remote/local twin
//     pub async fn send(&self, msg: &Message) -> Result<()> {
//         let mut conn = self.get_connection().await?;
//         conn.rpush(Queue::Local.as_ref(), msg)
//             .await
//             .context("unable to send your message")?;
//         Ok(())
//     }

//     /// Get response from the remote/local twin
//     /// [call this after use the send function]
//     pub async fn response<C: AsRef<str>>(&self, ret: C) -> Result<Message> {
//         let mut conn = self.get_connection().await?;
//         let msg: Message = conn
//             .lpop(ret.as_ref(), None)
//             .await
//             .context("failed to get a response message")?;
//         Ok(msg)
//     }

//     /// Get a command [Message] from another twin
//     pub async fn cmd<C: AsRef<str>>(&self, cmd: C) -> Result<Message> {
//         let mut conn = self.get_connection().await?;
//         let msg: Message = conn
//             .lpop(format!("msgbus.{}", cmd.as_ref()), None)
//             .await
//             .context("failed to get any commands")?;
//         Ok(msg)
//     }

//     /// Send a reply to a previously command received
//     pub async fn reply<C: AsRef<str>>(&self, reply: C, msg: &Message) -> Result<()> {
//         let mut conn = self.get_connection().await?;

//         conn.rpush(reply.as_ref(), msg)
//             .await
//             .context("unable to send your reply")?;

//         Ok(())
//     }
// }

// #[cfg(test)]
// mod tests {

//     use super::*;
//     async fn create_rmb_client<'a>() -> RmbClient {
//         let manager = RedisConnectionManager::new("redis://127.0.0.1/")
//             .context("unable to create redis connection manager")
//             .unwrap();
//         let pool = Pool::builder()
//             .build(manager)
//             .await
//             .context("unable to build pool or redis connection manager")
//             .unwrap();
//         let client = RmbClient::new(pool);

//         client
//     }

//     fn create_test_msg() -> Message {
//         let mut msg = Message::default();
//         // msg.id = uuid::Uuid::new_v4().to_string();
//         msg.destination = vec![55];
//         msg.command = "test".to_string();
//         msg.data = "some test data".to_string();
//         msg.retry = 1;
//         msg.source = 55;
//         msg.expiration = 1000;

//         msg
//     }

//     #[tokio::test]
//     async fn test_whole_process() {
//         let client = create_rmb_client().await;

//         // prepare message
//         let mut msg = create_test_msg();
//         msg.data = "sending...".to_string();

//         //send the message [cmd]
//         client.send(&msg).await.unwrap();
//         tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

//         // get the cmd [message]
//         let mut cmd = client.cmd("test").await.unwrap();
//         let rep = uuid::Uuid::new_v4().to_string();
//         cmd.reply = rep.clone();
//         // assert that the data is the same
//         assert_eq!(msg.data, cmd.data);

//         // reply
//         cmd.data = "return response".to_string();
//         client.reply(rep.clone(), &cmd).await.unwrap();
//         tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

//         // get the response
//         let received_msg = client.response(rep).await.unwrap();

//         assert_eq!(cmd.data, received_msg.data);
//     }
// }
