//! A [`Future`] with item yielded.
//!
//! ```
//! #![feature(generators)]
//!
//! # use anyhow::{Result, Ok};
//! # use stream_future::stream;
//! #[derive(Debug)]
//! enum Prog {
//!     Stage1,
//!     Stage2,
//!     End,
//! }
//!
//! #[stream(Prog)]
//! async fn foo() -> Result<i32> {
//!     yield Prog::Stage1;
//!     // some works...
//!     yield Prog::Stage2;
//!     // some other works...
//!     yield Prog::End;
//!     Ok(0)
//! }
//!
//! # use tokio_stream::StreamExt;
//! # #[tokio::main(flavor = "current_thread")]
//! # async fn main() -> Result<()> {
//! let bar = foo();
//! tokio::pin!(bar);
//! while let Some(prog) = bar.next().await {
//!     println!("{:?}", prog);
//! }
//! let bar = bar.await?;
//! assert_eq!(bar, 0);
//! # Ok(())
//! # }
//! ```
//!
//! If a lifetime is needed, specify it in the attribute:
//!
//! ```
//! #![feature(generators)]
//!
//! # use stream_future::stream;
//! enum Prog {
//!     Stage1,
//!     Stage2,
//! }
//!
//! #[stream(Prog, lifetime = "'a")]
//! async fn foo<'a>(s: &'a str) {
//!     yield Prog::Stage1;
//!     println!("{}", s);
//!     yield Prog::Stage2;
//! }
//!
//! # #[tokio::main(flavor = "current_thread")]
//! # async fn main() {
//! foo("Hello world!").await;
//! # }
//! ```

#![no_std]
#![feature(generator_trait)]

use core::{
    future::Future,
    ops::{Generator, GeneratorState},
    pin::Pin,
    ptr::NonNull,
    task::{Context, Poll},
};
use pin_project::pin_project;

pub use futures_core::Stream;
pub use stream_future_impl::stream;

/// See [`core::future::ResumeTy`].
#[doc(hidden)]
#[derive(Debug, Copy, Clone)]
pub struct ResumeTy(NonNull<Context<'static>>);

unsafe impl Send for ResumeTy {}
unsafe impl Sync for ResumeTy {}

impl ResumeTy {
    pub fn get_context<'a, 'b>(self) -> &'a mut Context<'b> {
        unsafe { &mut *self.0.as_ptr().cast() }
    }

    pub fn poll_future<F: Future>(self, f: Pin<&mut F>) -> Poll<F::Output> {
        f.poll(self.get_context())
    }
}

#[doc(hidden)]
#[pin_project]
pub struct GenFuture<P, T: Generator<ResumeTy, Yield = Poll<P>>> {
    #[pin]
    gen: T,
    ret: Option<T::Return>,
}

impl<P, T: Generator<ResumeTy, Yield = Poll<P>>> GenFuture<P, T> {
    pub const fn new(gen: T) -> Self {
        Self { gen, ret: None }
    }
}

impl<P, T: Generator<ResumeTy, Yield = Poll<P>>> Future for GenFuture<P, T> {
    type Output = T::Return;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let cx = NonNull::from(cx);
        let this = self.project();
        if let Some(x) = this.ret.take() {
            Poll::Ready(x)
        } else {
            let gen = this.gen;
            match gen.resume(ResumeTy(cx.cast())) {
                GeneratorState::Yielded(p) => match p {
                    Poll::Pending => Poll::Pending,
                    Poll::Ready(_) => {
                        unsafe { cx.as_ref() }.waker().wake_by_ref();
                        Poll::Pending
                    }
                },
                GeneratorState::Complete(x) => Poll::Ready(x),
            }
        }
    }
}

impl<P, T: Generator<ResumeTy, Yield = Poll<P>>> Stream for GenFuture<P, T> {
    type Item = P;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let gen = this.gen;
        match gen.resume(ResumeTy(NonNull::from(cx).cast())) {
            GeneratorState::Yielded(p) => match p {
                Poll::Pending => Poll::Pending,
                Poll::Ready(p) => Poll::Ready(Some(p)),
            },
            GeneratorState::Complete(x) => {
                *this.ret = Some(x);
                Poll::Ready(None)
            }
        }
    }
}
