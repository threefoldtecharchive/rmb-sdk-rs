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
    use bb8_redis::{bb8::{Pool, PooledConnection}, RedisConnectionManager, redis::AsyncCommands};

    use crate::{server::{HandlerInput, HandlerOutput}, client::Request, msg::Message};

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

        let server = Server::new(AppData {},pool, 20, );

        server
    }

    impl TryInto<Vec<isize>> for HandlerInput {
        type Error = anyhow::Error;

        fn try_into(self) -> Result<Vec<isize>, Self::Error> {
            let data: Vec<isize> = serde_json::from_slice(&self.data)?;

            Ok(data)
        }
    }

    impl TryFrom<isize> for HandlerOutput {
        type Error = anyhow::Error;

        fn try_from(value: isize) -> Result<Self, Self::Error> {
            let data = serde_json::to_vec(&value)?;
            Ok(HandlerOutput {
                data,
                schema: "application/json".to_string(),
            })
        }
    }

    impl TryFrom<Vec<isize>> for HandlerOutput {
        type Error = anyhow::Error;

        fn try_from(value: Vec<isize>) -> Result<Self, Self::Error> {
            let res = serde_json::to_vec(&value)?;
            Ok(HandlerOutput {
                data: res,
                schema: "application/json".to_string(),
            })
        }
    }

    impl TryFrom<&str> for HandlerOutput {
        type Error = anyhow::Error;

        fn try_from(value: &str) -> Result<Self, Self::Error> {
            let res = serde_json::to_vec(value)?;
            Ok(HandlerOutput {
                data: res,
                schema: "application/json".to_string(),
            })
        }
    }



    #[derive(Clone)]
    struct AppData;

    /* async */
    #[handler]
    async fn add(_data: AppData, args: HandlerInput) -> Result<HandlerOutput> {
        let data: Vec<isize> = args.try_into()?;
        let sum: isize = data.into_iter().sum();
        HandlerOutput::try_from(sum)
    }

    #[handler]
    async fn mul(_data: AppData, args: HandlerInput) -> Result<HandlerOutput> {
        let data: Vec<isize> = args.try_into()?;
        let mut result = if data.is_empty() { 0 } else { 1 };

        for el in data.iter() {
            result *= el.to_owned();
        }
        HandlerOutput::try_from(result)
    }

    #[handler]
    async fn div(_data: AppData, args: HandlerInput) -> Result<HandlerOutput> {
        let data: Vec<isize> = args.try_into()?;

        let mut base = if data.is_empty() {
            0
        } else if data[0] == 0 {
            anyhow::bail!("cannot divide by zero");
        } else {
            data[0]
        };

        if data.len() > 1 {
            base = data[1..].iter().fold(base, |acc, el| acc / el);
        }

        HandlerOutput::try_from(base)
    }

    #[handler]
    async fn sub(_data: AppData, args: HandlerInput) -> Result<HandlerOutput> {
        let data: Vec<isize> = args.try_into()?;
        let data = data[1..].iter().fold(data[0], |acc, el| acc - el);
        HandlerOutput::try_from(data)
    }

    #[handler]
    async fn sqr(_data: AppData, args: HandlerInput) -> Result<HandlerOutput> {
        let data: Vec<isize> = args.try_into()?;
        let data = data.iter().map(|el| el * el).collect::<Vec<isize>>();
        HandlerOutput::try_from(data)
    }

    #[handler]
    async fn version(_data: AppData, _args: HandlerInput) -> Result<HandlerOutput> {
        HandlerOutput::try_from("v1.0")
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
            let mut conn = self.get_connection()
                .await
                .unwrap();
                let _res: usize = conn.rpush("msgbus.calculator.add", req.body())
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


        let _handler = tokio::spawn(server.run());
        tokio::time::sleep(Duration::from_secs(3)).await;
        let msg = remote_rmb.pop_reply().await.unwrap();

        println!("{:?}", msg);

    }
}
