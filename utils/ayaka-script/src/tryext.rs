use log::warn;
use std::{
    fmt::Debug,
    ops::{ControlFlow, Try},
};

pub trait TryExt: Try {
    fn ok_or_inspect(self, f: impl FnOnce(&Self::Residual)) -> Option<Self::Output>;

    fn ok_or_log(self, msg: &str) -> Option<Self::Output>
    where
        Self: Sized,
        Self::Residual: Debug,
    {
        self.ok_or_inspect(|e| warn!("{}: {:?}", msg, e))
    }

    fn unwrap_or_default_inspect(self, f: impl FnOnce(&Self::Residual)) -> Self::Output;

    fn unwrap_or_default_log(self, msg: &str) -> Self::Output
    where
        Self: Sized,
        Self::Residual: Debug,
    {
        self.unwrap_or_default_inspect(|e| warn!("{}: {:?}", msg, e))
    }
}

impl<T: Try> TryExt for T
where
    T::Output: Default,
{
    fn ok_or_inspect(self, f: impl FnOnce(&Self::Residual)) -> Option<Self::Output> {
        match self.branch() {
            ControlFlow::Continue(v) => Some(v),
            ControlFlow::Break(e) => {
                f(&e);
                None
            }
        }
    }

    fn unwrap_or_default_inspect(self, f: impl FnOnce(&Self::Residual)) -> Self::Output {
        match self.branch() {
            ControlFlow::Continue(v) => v,
            ControlFlow::Break(e) => {
                f(&e);
                Default::default()
            }
        }
    }
}
