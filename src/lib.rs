mod client;
mod msg;
mod server;
mod util;

pub use client::{request::Request, Client};
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

#[cfg(test)]
mod tests {

    use anyhow::Context;
    use bb8_redis::{bb8::Pool, RedisConnectionManager};

    use super::*;
    async fn create_rmb_client<'a>() -> Client {
        let manager = RedisConnectionManager::new("redis://127.0.0.1/")
            .context("unable to create redis connection manager")
            .unwrap();
        let pool = Pool::builder()
            .build(manager)
            .await
            .context("unable to build pool or redis connection manager")
            .unwrap();
        let client = Client::new(pool);

        client
    }

    #[tokio::test]
    async fn test_whole_process() {
        // let client = create_rmb_client().await;

        // let args = [1, 2];

        // // create request
        // let req = Request::new("calc.add").args(args).destination(55);

        // // send it
        // let mut resp = client.send(req).await.unwrap();

        // // get response
        // while let Some(value) = resp.get().await {
        //     match value {
        //         Ok(resp) => {
        //             let resp = resp.body().to_string();
        //         }
        //         Err(err) => {}
        //     }
        // }
    }
}
