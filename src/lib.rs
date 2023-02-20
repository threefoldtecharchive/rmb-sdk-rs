pub mod client;
pub mod server;

mod protocol;
mod util;
use anyhow::Result;
use bb8_redis::{bb8::Pool, RedisConnectionManager};

pub const DEFAULT_URL: &str = "redis://127.0.0.1:6379";

/// create redis pool
pub async fn pool<U: AsRef<str>>(url: U) -> Result<Pool<RedisConnectionManager>> {
    let mgr = RedisConnectionManager::new(url.as_ref())?;
    let pool = Pool::builder().max_size(20).build(mgr).await?;

    Ok(pool)
}

pub async fn default() -> Result<Pool<RedisConnectionManager>> {
    pool(DEFAULT_URL).await
}

// #[cfg(test)]
// mod tests {
//     use std::time::Duration;

//     use handler::handler;
//     use server::{Handler, Router, Server};

//     use anyhow::{Context, Result};
//     use bb8_redis::{
//         bb8::{Pool, PooledConnection},
//         redis::AsyncCommands,
//         RedisConnectionManager,
//     };

//     use crate::{
//         client::Client,
//         client::Request,
//         server::{HandlerInput, HandlerOutput},
//     };

//     use super::*;
//     async fn get_redis_pool() -> Pool<RedisConnectionManager> {
//         let manager = RedisConnectionManager::new("redis://127.0.0.1/")
//             .context("unable to create redis connection manager")
//             .unwrap();
//         let pool = Pool::builder()
//             .build(manager)
//             .await
//             .context("unable to build pool or redis connection manager")
//             .unwrap();

//         pool
//     }

//     async fn _create_rmb_client<'a>() -> Client {
//         let pool = get_redis_pool().await;
//         let client = Client::new(pool);

//         client
//     }

//     async fn create_rmb_server() -> Server<AppData> {
//         let pool = get_redis_pool().await;

//         let server = Server::new(AppData {}, pool, 20);

//         server
//     }

//     fn form_request() -> Request {
//         let req = Request::new("calculator.add");
//         req.args(vec![2, 4]).destination(55)
//     }

//     #[derive(Clone)]
//     struct AppData;

//     #[handler]
//     async fn add<'a>(_data: AppData, args: HandlerInput<'a>) -> Result<HandlerOutput> {
//         let (a, b): (f64, f64) = args.inputs()?;

//         HandlerOutput::from(a + b)
//     }

//     #[handler]
//     async fn mul<'a>(_data: AppData, args: HandlerInput<'a>) -> Result<HandlerOutput> {
//         let (a, b): (f64, f64) = args.inputs()?;

//         HandlerOutput::from(a * b)
//     }

//     #[handler]
//     async fn div<'a>(_data: AppData, args: HandlerInput<'a>) -> Result<HandlerOutput> {
//         let (a, b): (f64, f64) = args.inputs()?;

//         if b == 0.0 {
//             anyhow::bail!("cannot divide by zero");
//         }

//         HandlerOutput::from(a / b)
//     }

//     #[handler]
//     async fn sub<'a>(_data: AppData, args: HandlerInput<'a>) -> Result<HandlerOutput> {
//         let (a, b): (f64, f64) = args.inputs()?;

//         HandlerOutput::from(a - b)
//     }

//     #[handler]
//     async fn sqr<'a>(_data: AppData, args: HandlerInput<'a>) -> Result<HandlerOutput> {
//         let x: f64 = args.inputs()?;

//         HandlerOutput::from(x.sqrt())
//     }

//     #[handler]
//     async fn version<'a>(_data: AppData, _args: HandlerInput<'a>) -> Result<HandlerOutput> {
//         HandlerOutput::from("v1.0")
//     }

//     fn build_deep<M: Router<AppData>>(router: &mut M) {
//         // we can pass a ref to a router. and fill it
//         // up with handler and or even more sub modules.
//         router.handle("test", sub);
//     }

//     fn form_modules_handles(server: &mut Server<AppData>) {
//         server.handle("version", version);

//         let calculator = server.module("calculator");
//         calculator
//             .handle("add", add)
//             .handle("mul", mul)
//             .handle("div", div);

//         let scientific = server.module("scientific");
//         scientific.handle("sqr", sqr);

//         // extend modules that is already there. and pass them around
//         let deep = server.module("calculator").module("deep");
//         build_deep(deep);
//     }

//     struct MockRmb {
//         pool: Pool<RedisConnectionManager>,
//     }

//     impl MockRmb {
//         pub async fn new() -> Self {
//             Self {
//                 pool: get_redis_pool().await,
//             }
//         }

//         pub async fn push_cmd(&self, req: Request) {
//             let mut conn = self.get_connection().await.unwrap();
//             let msg = Message::from(req);
//             let _res: usize = conn
//                 .rpush(format!("msgbus.{}", msg.command), msg)
//                 .await
//                 .unwrap();
//         }

//         pub async fn pop_reply(&self) -> Result<Message> {
//             let mut conn = self.get_connection().await?;
//             let res: (String, Message) = conn.brpop("msgbus.system.reply", 0).await?;
//             let msg = res.1;

//             Ok(msg)
//         }

//         pub async fn push_response(&self) -> Result<()> {
//             let reply = self.pop_reply().await?;

//             let mut conn = self.get_connection().await.unwrap();
//             let _res: usize = conn.rpush(reply.reply.clone(), reply).await.unwrap();
//             Ok(())
//         }

//         pub async fn pop_request(&self) -> Result<()> {
//             let mut conn = self.get_connection().await?;
//             let res: (String, Message) = conn.brpop("msgbus.system.local", 0).await?;
//             let request = Request::from(res.1);

//             self.push_cmd(request).await;

//             Ok(())
//         }

//         #[inline]
//         async fn get_connection(&self) -> Result<PooledConnection<'_, RedisConnectionManager>> {
//             let conn = self
//                 .pool
//                 .get()
//                 .await
//                 .context("unable to retrieve a redis connection from the pool")?;

//             Ok(conn)
//         }
//     }

//     #[tokio::test]
//     async fn test_server_routing() {
//         let mut server: Server<AppData> = create_rmb_server().await;
//         server.handle("version", version);

//         let calculator = server.module("calculator");
//         calculator
//             .handle("add", add)
//             .handle("mul", mul)
//             .handle("div", div);

//         let scientific = server.module("scientific");
//         scientific.handle("sqr", sqr);

//         // extend modules that is already there. and pass them around
//         let deep = server.module("calculator").module("deep");
//         build_deep(deep);

//         assert!(matches!(server.lookup("version"), Some(_)));
//         assert!(matches!(server.lookup("calculator.add"), Some(_)));
//         assert!(matches!(server.lookup("scientific.sqr"), Some(_)));
//         assert!(matches!(server.lookup("calculator.wrong"), None));
//         assert!(matches!(server.lookup("calculator.deep.test"), Some(_)));

//         let input = HandlerInput {
//             source: 0,
//             schema: "application/json".into(),
//             data: serde_json::to_vec(&(10.0, 20)).unwrap(),
//         };
//         // test add
//         let handler = server.lookup("calculator.add").unwrap();
//         let result = handler.call(AppData, input).await.unwrap();

//         assert_eq!(result.schema, "application/json");
//         let result: f64 = serde_json::from_slice(&result.data).unwrap();

//         assert_eq!(result, 30.0);

//         let input = HandlerInput {
//             source: 0,
//             schema: "application/json".into(),
//             data: serde_json::to_vec(&(10.0, 0)).unwrap(),
//         };

//         // test divide by zero
//         let handler = server.lookup("calculator.div").unwrap();
//         assert!(handler.call(AppData, input).await.is_err());
//     }

//     #[tokio::test]
//     async fn test_server_process() {
//         // start rmb
//         let rmb = MockRmb::new().await;

//         // rmb received a command somewhere and add it to the cmd's queue
//         rmb.push_cmd(form_request()).await;

//         // server will process commands
//         let mut server: Server<AppData> = create_rmb_server().await;
//         form_modules_handles(&mut server);

//         assert!(matches!(server.lookup("version"), Some(_)));
//         assert!(matches!(server.lookup("calculator.add"), Some(_)));
//         assert!(matches!(server.lookup("scientific.sqr"), Some(_)));
//         assert!(matches!(server.lookup("calculator.wrong"), None));
//         assert!(matches!(server.lookup("calculator.deep.test"), Some(_)));

//         let _handler = tokio::spawn(server.run());
//         tokio::time::sleep(Duration::from_secs(1)).await;

//         // rmb received a reply from the server
//         let mut reply = rmb.pop_reply().await.unwrap();

//         // assert the result
//         let data = base64::decode(reply.data).unwrap();
//         reply.data = String::from_utf8(data).unwrap();
//         let result: f64 = serde_json::from_str(&reply.data).unwrap();

//         assert_eq!(result, 6.0);
//     }

//     #[tokio::test]
//     async fn test_client_process() {
//         // start rmb
//         let rmb = MockRmb::new().await;

//         // create request
//         let request = form_request();

//         let msg: Message = request.clone().into();
//         // client
//         let client = Client::new(get_redis_pool().await);

//         // send request
//         let mut response = client.send(request).await.unwrap();

//         // server to handle request
//         let mut server: Server<AppData> = create_rmb_server().await;
//         form_modules_handles(&mut server);
//         let _handler = tokio::spawn(server.run());

//         // rmb transfer the request
//         rmb.pop_request().await.unwrap();
//         tokio::time::sleep(Duration::from_secs(1)).await;

//         rmb.push_response().await.unwrap();
//         tokio::time::sleep(Duration::from_secs(1)).await;

//         // get the response
//         let response_body = response.get().await.unwrap().unwrap();
//         let result: f64 = response_body.outputs().unwrap();

//         assert_eq!(result, 6.0);
//     }
// }
