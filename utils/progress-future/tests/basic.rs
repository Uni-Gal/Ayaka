#![feature(generators)]

use progress_future::*;
use std::{future::ready, time::Duration};
use tokio::time::interval;
use tokio_stream::StreamExt;

#[tokio::test]
async fn basic() {
    #[progress(i32)]
    async fn foo() -> bool {
        yield 0;
        yield 1;
        yield (ready(2).await);
        true
    }

    let gf = foo();
    tokio::pin!(gf);
    assert_eq!((&mut gf).collect::<Vec<_>>().await, [0, 1, 2]);
    assert_eq!(gf.await, true);
}

#[tokio::test]
async fn stream() {
    #[progress(i32)]
    async fn foo() {
        yield 0;
        yield (ready(1).await);
        yield 2;
    }

    let gf = foo();
    tokio::pin!(gf);
    assert_eq!(gf.collect::<Vec<_>>().await, [0, 1, 2]);
}

#[tokio::test]
async fn future() {
    #[progress("()")]
    async fn foo() -> bool {
        ready(true).await
    }

    assert_eq!(foo().await, true);

    #[progress]
    async fn bar() -> bool {
        ready(true).await
    }

    assert_eq!(bar().await, true);
}

#[tokio::test]
async fn timeout() {
    #[progress]
    async fn foo() {
        let mut timer = interval(Duration::from_secs(1));
        timer.tick().await;
        timer.tick().await;
    }

    foo().await;
}
