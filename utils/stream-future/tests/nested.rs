#![feature(generators)]

use std::future::ready;

use stream_future::*;

#[tokio::test]
async fn nested() {
    #[stream]
    async fn foo() {
        yield;
        async { ready(1).await }.await;
        yield;
    }

    foo().await;
}
