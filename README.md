# rmb-sdk-rs
**rmb-sdk** provide tools for developers to quickly build a server that runs behind [RMB](https://github.com/threefoldtech/rmb-rs). Both server and client need to run next to their corresponding `rmb` instance.

The idea is that __rmb__ takes care of message routing. Hence the server only need to build **hooks** to the proper redis queues and then be able to process requests and send back responses.

## Usage
The crate is split into two main parts
- server
- client

## Server
Let's jump directly to code. Here is a fully working example of server code built with the sdk

```rust
use rmb_sdk::server::{handler, Handler, HandlerInput, HandlerOutput, Router, Server};

use anyhow::Result;

use std::time::Duration;

#[handler]
async fn add(_data: (), input: HandlerInput) -> Result<HandlerOutput> {
    let (a, b): (f64, f64) = input.inputs()?;

    HandlerOutput::from(a + b)
}

#[handler]
async fn div(_data: (), input: HandlerInput) -> Result<HandlerOutput> {
    let (a, b): (f64, f64) = input.inputs()?;
    if b == 0.0 {
        anyhow::bail!("cannot divide by zero");
    }

    HandlerOutput::from(a / b)
}

#[tokio::main]
async fn main() {
    let pool = rmb_sdk::pool(rmb_sdk::DEFAULT_URL).await.unwrap();

    let mut server = Server::new((), pool, 2);

    server
        .module("calculator")
        .handle("add", add)
        .handle("div", div);

    server.run().await.unwrap();
}
```

The server can also have a `state` that is shared between all handlers. For example the developer can build his own app state. The state must be cloneable like this

```rust

#[derive(Clone, Default)]
struct State {
    count: Arc<Mutex<usize>>,
}

#[handler]
async fn increment(state: State, input: HandlerInput) -> Result<HandlerOutput> {
    let i: usize = input.inputs()?;
    let mut count = state.count.lock().await;
    *count += i;

    HandlerOutput::from(*count)
}

#[tokio::main]
async fn main() {
    let pool = rmb_sdk::pool(rmb_sdk::DEFAULT_URL).await.unwrap();
    let state = State::default();

    let mut server = Server::new(state, pool, 2);

    server.module("calculator").handle("increment", increment);

    server.run().await.unwrap();
}

```

## Client
This example code shows how the client is intended to be used. The caller should consume all responses, since there is at least one Return per destination.

The `.get()` on the response will return Ok(None) if the message has expired or total number of expected responses has been received
```rust

#[tokio::main]
async fn main2() {
    let pool = rmb_sdk::pool(rmb_sdk::DEFAULT_URL).await.unwrap();

    let client = rmb_sdk::client::Client::new(pool);
    let inputs: (f64, f64) = (10.0, 20.0);
    let request = rmb_sdk::client::Request::new("calculator.add")
        .args(inputs)
        .destination(7)
        .destination(10)
        .expiration(Duration::from_secs(10));

    let mut response = client.send(request).await.unwrap();

    while let Ok(Some(ret)) = response.get().await {
        let out: f64 = ret.outputs().unwrap();
        println!("source: {}, output: {}", ret.source, out);
    }
}

```
