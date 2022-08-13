# Run a game
The CLI tool in `bins/gal` is a full example to run a game.

## Open a config file
``` rust,ignore
use gal_runtime::*;
let mut context = Context::open("../../../examples/Fibonacci/config.yaml", FrontendType::Text).await?;
```
The context object should be initialized first to start from an initial record.
``` rust,ignore
context.init_new();
```
Then you can iterate the actions:
``` rust,ignore
while let Some(action) = context.next_run() {
    //...
}
```

## Get the open status
The `context` is a [`ProgressFuture`](./pfuture.md) object.
The `OpenStatus` could be iterated before the future awaited.
``` rust,ignore
use gal_runtime::*;
use tokio_stream::StreamExt;
let context = Context::open("../../../examples/Fibonacci/config.yaml", FrontendType::Text);
tokio::pin!(context);
while let Some(status) = context.next().await {
    println!("{:?}", status);
}
let mut context = context.await?;
```
