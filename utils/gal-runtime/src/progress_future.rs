use std::{
    fmt::Debug,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio_stream::{wrappers::UnboundedReceiverStream, Stream};

pub trait Progress = Debug + Send + Sync + 'static;

pub struct ProgressFuture<F: Future, P> {
    future: F,
    progress: UnboundedReceiverStream<P>,
    result: Option<F::Output>,
}

impl<F: Future, P: Progress> ProgressFuture<F, P> {
    pub fn new(f: impl FnOnce(UnboundedSender<P>) -> F) -> Self {
        let (tx, rx) = unbounded_channel();
        Self {
            future: f(tx),
            progress: UnboundedReceiverStream::new(rx),
            result: None,
        }
    }
}

impl<F: Future, P> Future for ProgressFuture<F, P> {
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

impl<F: Future, P: Progress> Stream for ProgressFuture<F, P> {
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
