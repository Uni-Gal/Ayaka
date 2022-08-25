#![no_std]
#![feature(generators, generator_trait, negative_impls)]

use core::{
    future::Future,
    ops::{Generator, GeneratorState},
    pin::Pin,
    ptr::NonNull,
    task::{Context, Poll},
};
use pin_project::pin_project;

pub use progress_future_impl::progress;
pub use tokio_stream::Stream;

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
    pub fn new(gen: T) -> Self {
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
