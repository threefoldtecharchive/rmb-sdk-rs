pub mod client;
mod msg;
pub mod server;
mod util;

pub use client::Client;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use handler::handler;
    use server::{Handler, Router, Server};

    use anyhow::{Context, Result};
    use bb8_redis::{
        bb8::{Pool, PooledConnection},
        redis::AsyncCommands,
        RedisConnectionManager,
    };

    use crate::{
        client::Request,
        msg::Message,
        server::{HandlerInput, HandlerOutput},
    };

    use super::*;
    async fn get_redis_pool() -> Pool<RedisConnectionManager> {
        let manager = RedisConnectionManager::new("redis://127.0.0.1/")
            .context("unable to create redis connection manager")
            .unwrap();
        let pool = Pool::builder()
            .build(manager)
            .await
            .context("unable to build pool or redis connection manager")
            .unwrap();

        pool
    }

    async fn _create_rmb_client<'a>() -> Client {
        let pool = get_redis_pool().await;
        let client = Client::new(pool);

        client
    }

    async fn create_rmb_server() -> Server<AppData> {
        let pool = get_redis_pool().await;

        let server = Server::new(AppData {}, pool, 20);

        server
    }

    #[derive(Clone)]
    struct AppData;

    /* async */
    #[handler]
    async fn add(_data: AppData, args: HandlerInput) -> Result<HandlerOutput> {
        let (a, b): (f64, f64) = args.inputs()?;

        HandlerOutput::from(a + b)
    }

    #[handler]
    async fn mul(_data: AppData, args: HandlerInput) -> Result<HandlerOutput> {
        let (a, b): (f64, f64) = args.inputs()?;

        HandlerOutput::from(a * b)
    }

    #[handler]
    async fn div(_data: AppData, args: HandlerInput) -> Result<HandlerOutput> {
        let (a, b): (f64, f64) = args.inputs()?;

        if b == 0.0 {
            anyhow::bail!("cannot divide by zero");
        }

        HandlerOutput::from(a / b)
    }

    #[handler]
    async fn sub(_data: AppData, args: HandlerInput) -> Result<HandlerOutput> {
        let (a, b): (f64, f64) = args.inputs()?;

        HandlerOutput::from(a - b)
    }

    #[handler]
    async fn sqr(_data: AppData, args: HandlerInput) -> Result<HandlerOutput> {
        let x: f64 = args.inputs()?;

        HandlerOutput::from(x.sqrt())
    }

    #[handler]
    async fn version(_data: AppData, _args: HandlerInput) -> Result<HandlerOutput> {
        HandlerOutput::from("v1.0")
    }

    fn build_deep<M: Router<AppData>>(router: &mut M) {
        // we can pass a ref to a router. and fill it
        // up with handler and or even more sub modules.
        router.handle("test", sub);
    }

    struct MockRmb {
        pool: Pool<RedisConnectionManager>,
    }

    impl MockRmb {
        pub async fn new() -> Self {
            Self {
                pool: get_redis_pool().await,
            }
        }

        pub async fn push_cmd(&self) {
            let req = Request::new("calculator.add").args([1, 3, 4]);
            let mut conn = self.get_connection().await.unwrap();
            let _res: usize = conn
                .rpush("msgbus.calculator.add", req.body())
                .await
                .unwrap();
        }

        pub async fn pop_reply(&self) -> Result<Message> {
            let mut conn = self.get_connection().await?;
            let res: (String, Message) = conn.brpop("msgbus.system.reply", 0).await?;
            Ok(res.1)
        }

        #[inline]
        async fn get_connection(&self) -> Result<PooledConnection<'_, RedisConnectionManager>> {
            let conn = self
                .pool
                .get()
                .await
                .context("unable to retrieve a redis connection from the pool")?;

            Ok(conn)
        }
    }

    #[tokio::test]
    async fn test_server_process() {
        let remote_rmb = MockRmb::new().await;
        remote_rmb.push_cmd().await;

        let mut server: Server<AppData> = create_rmb_server().await;
        server.handle("version", version);

        let calculator = server.module("calculator");
        calculator
            .handle("add", add)
            .handle("mul", mul)
            .handle("div", div);

        let scientific = server.module("scientific");
        scientific.handle("sqr", sqr);

        // extend modules that is already there. and pass them around
        let deep = server.module("calculator").module("deep");
        build_deep(deep);

        assert!(matches!(server.lookup("version"), Some(_)));
        assert!(matches!(server.lookup("calculator.add"), Some(_)));
        assert!(matches!(server.lookup("scientific.sqr"), Some(_)));
        assert!(matches!(server.lookup("calculator.wrong"), None));
        assert!(matches!(server.lookup("calculator.deep.test"), Some(_)));

        let input = HandlerInput {
            schema: "application/json".into(),
            data: serde_json::to_vec(&(10.0, 20)).unwrap(),
        };
        // test add
        let handler = server.lookup("calculator.add").unwrap();
        let result = handler.call(AppData, input).await.unwrap();

        assert_eq!(result.schema, "application/json");
        let result: f64 = serde_json::from_slice(&result.data).unwrap();

        assert_eq!(result, 30.0);
    }
}
