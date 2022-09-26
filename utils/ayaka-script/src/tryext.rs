use log::warn;
use std::{
    fmt::Debug,
    ops::{ControlFlow, Try},
};

#[doc(hidden)]
pub trait TryExt: Try {
    fn unwrap_or_default_log(self, msg: &str) -> Self::Output;
}

impl<T: Try> TryExt for T
where
    T::Output: Default,
    T::Residual: Debug,
{
    fn unwrap_or_default_log(self, msg: &str) -> Self::Output {
        match self.branch() {
            ControlFlow::Continue(v) => v,
            ControlFlow::Break(e) => {
                warn!("{msg}: {e:?}");
                Default::default()
            }
        }
    }
}
