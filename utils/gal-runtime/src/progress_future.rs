use pin_project::pin_project;
use std::{
    fmt::Debug,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio_stream::{wrappers::UnboundedReceiverStream, Stream};

pub trait Progress = Debug + Send + Sync + 'static;

/// A [`Future`] with progress yielded.
///
/// ```
/// #![feature(async_closure)]
///
/// # use std::future::Future;
/// # use anyhow::{Result, Ok};
/// # use gal_runtime::ProgressFuture;
/// #[derive(Debug)]
/// enum Prog {
///     Stage1,
///     Stage2,
///     End,
/// }
///
/// fn foo() -> ProgressFuture<impl Future<Output = Result<i32>>, Prog> {
///     ProgressFuture::new(async move |tx| {
///         tx.send(Prog::Stage1)?;
///         // some works...
///         tx.send(Prog::Stage2)?;
///         // some other works...
///         tx.send(Prog::End)?;
///         Ok(0)
///     })
/// }
///
/// # use tokio_stream::StreamExt;
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() -> Result<()> {
/// let bar = foo();
/// tokio::pin!(bar);
/// while let Some(prog) = bar.next().await {
///     println!("{:?}", prog);
/// }
/// let bar = bar.await?;
/// assert_eq!(bar, 0);
/// # Ok(())
/// # }
/// ```
#[pin_project]
pub struct ProgressFuture<F: Future, P> {
    #[pin]
    future: F,
    #[pin]
    progress: UnboundedReceiverStream<P>,
    result: Option<F::Output>,
}

impl<F: Future, P: Progress> ProgressFuture<F, P> {
    /// Creates a new [`ProgressFuture`].
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
        let this = self.project();
        if let Some(res) = this.result.take() {
            Poll::Ready(res)
        } else {
            this.future.poll(cx)
        }
    }
}

impl<F: Future, P: Progress> Stream for ProgressFuture<F, P> {
    type Item = P;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        match this.progress.poll_next(cx) {
            Poll::Pending => {
                match this.future.poll(cx) {
                    Poll::Pending => {}
                    Poll::Ready(res) => *this.result = Some(res),
                };
                Poll::Pending
            }
            Poll::Ready(value) => Poll::Ready(value),
        }
    }
}
