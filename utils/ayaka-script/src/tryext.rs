use log::warn;
use std::{
    fmt::Debug,
    ops::{ControlFlow, Try},
};

fn try_unwrap_or_default_inspect<T: Try>(t: T, f: impl FnOnce(&T::Residual)) -> T::Output
where
    T::Output: Default,
{
    match t.branch() {
        ControlFlow::Continue(v) => v,
        ControlFlow::Break(e) => {
            f(&e);
            Default::default()
        }
    }
}

#[doc(hidden)]
pub trait TryExt: Try {
    fn unwrap_or_default_log(self, msg: &str) -> Self::Output;

    fn unwrap_or_default_log_with(self, f: impl FnOnce() -> String) -> Self::Output;
}

impl<T: Try> TryExt for T
where
    T::Output: Default,
    T::Residual: Debug,
{
    fn unwrap_or_default_log(self, msg: &str) -> Self::Output {
        try_unwrap_or_default_inspect(self, |e| warn!("{msg}: {e:?}"))
    }

    fn unwrap_or_default_log_with(self, f: impl FnOnce() -> String) -> Self::Output {
        try_unwrap_or_default_inspect(self, |e| warn!("{}: {e:?}", f()))
    }
}
