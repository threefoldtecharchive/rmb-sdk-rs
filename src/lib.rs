mod client;
mod msg;
mod server;
mod util;

pub use client::{request::Request, Client};
pub use server::{handler, Handler, Server};

#[cfg(test)]
mod tests {

    use anyhow::Context;
    use bb8_redis::{bb8::Pool, RedisConnectionManager};

    use crate::server::{HandlerInput, HandlerOutput};
    use crate::server::{Router, Server};

    use super::*;
    async fn _create_rmb_client<'a>() -> Client {
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
    async fn create_rmb_server() -> Server {
        let manager = RedisConnectionManager::new("redis://127.0.0.1/")
            .context("unable to create redis connection manager")
            .unwrap();
        let pool = Pool::builder()
            .build(manager)
            .await
            .context("unable to build pool or redis connection manager")
            .unwrap();

        let server = Server::new(pool);

        server
    }

    /* async */
    #[handler]
    async fn add(_args: HandlerInput) -> HandlerOutput {
        unimplemented!()
    }

    #[handler]
    async fn mul(_args: HandlerInput) -> HandlerOutput {
        unimplemented!()
    }

    #[handler]
    async fn div(_args: HandlerInput) -> HandlerOutput {
        unimplemented!()
    }

    #[handler]
    async fn sub(_args: HandlerInput) -> HandlerOutput {
        unimplemented!()
    }

    #[handler]
    async fn sqr(_args: HandlerInput) -> HandlerOutput {
        unimplemented!()
    }

    #[handler]
    async fn version(_args: HandlerInput) -> HandlerOutput {
        unimplemented!()
    }

    async fn build_deep<M: Router>(router: &mut M) {
        // we can pass a ref to a router. and fill it
        // up with handler and or even more sub modules.
        router.handle("test", sub);
    }

    #[tokio::test]
    async fn test_whole_process() {
        let mut server = create_rmb_server().await;
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
    }
}
