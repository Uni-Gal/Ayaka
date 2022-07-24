use std::{
    fmt::Debug,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::sync::watch::{channel, Sender};
use tokio_stream::{wrappers::WatchStream, Stream};

pub trait Progress = Debug + Clone + Send + Sync + 'static;

pub struct ProgressFuture<'a, F: Future + 'a, P> {
    future: F,
    progress: WatchStream<P>,
    result: Option<F::Output>,
    _marker: PhantomData<&'a ()>,
}

impl<'a, F: Future + 'a, P: Progress> ProgressFuture<'a, F, P> {
    pub fn new(init: P, f: impl FnOnce(Sender<P>) -> F) -> Self {
        let (tx, rx) = channel(init);
        Self {
            future: f(tx),
            progress: WatchStream::new(rx),
            result: None,
            _marker: PhantomData::default(),
        }
    }
}

impl<'a, F: Future + 'a, P> Future for ProgressFuture<'a, F, P> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let unpinned_self = unsafe { Pin::into_inner_unchecked(self) };
        if let Some(res) = unpinned_self.result.take() {
            Poll::Ready(res)
        } else {
            unsafe { Pin::new_unchecked(&mut unpinned_self.future) }.poll(cx)
        }
    }
}

impl<'a, F: Future + 'a, P: Progress> Stream for ProgressFuture<'a, F, P> {
    type Item = P;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let unpinned_self = unsafe { Pin::into_inner_unchecked(self) };
        match unsafe { Pin::new_unchecked(&mut unpinned_self.progress) }.poll_next(cx) {
            Poll::Pending => {
                match unsafe { Pin::new_unchecked(&mut unpinned_self.future) }.poll(cx) {
                    Poll::Pending => {}
                    Poll::Ready(res) => unpinned_self.result = Some(res),
                };
                Poll::Pending
            }
            Poll::Ready(value) => Poll::Ready(value),
        }
    }
}
