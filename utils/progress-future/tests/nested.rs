#![feature(generators)]

use std::future::ready;

use progress_future::*;

#[tokio::test]
async fn nested() {
    #[progress]
    async fn foo() {
        yield;
        async { ready(1).await }.await;
        yield;
    }

    foo().await;
}
