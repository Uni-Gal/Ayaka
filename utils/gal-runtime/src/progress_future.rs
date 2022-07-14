use std::{
    fmt::Debug,
    future::Future,
    pin::Pin,
    result::Result,
    task::{Context, Poll},
};
use tokio::{
    sync::watch::{channel, Sender},
    task::{JoinError, JoinHandle},
};
use tokio_stream::{wrappers::WatchStream, Stream};

pub trait Progress = Debug + Clone + Send + Sync + 'static;

pub struct ProgressFuture<T, P> {
    future: JoinHandle<T>,
    progress: WatchStream<P>,
}

impl<T: Send + 'static, P: Progress> ProgressFuture<T, P> {
    pub fn new<F: Future<Output = T> + Send + 'static>(
        init: P,
        f: impl FnOnce(Sender<P>) -> F,
    ) -> Self {
        let (tx, rx) = channel(init);
        Self {
            future: tokio::spawn(f(tx)),
            progress: WatchStream::new(rx),
        }
    }
}

impl<T, P> Future for ProgressFuture<T, P> {
    type Output = Result<T, JoinError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe { Pin::new_unchecked(&mut Pin::into_inner_unchecked(self).future) }.poll(cx)
    }
}

impl<T, P: Progress> Stream for ProgressFuture<T, P> {
    type Item = P;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        unsafe { Pin::new_unchecked(&mut Pin::into_inner_unchecked(self).progress) }.poll_next(cx)
    }
}
