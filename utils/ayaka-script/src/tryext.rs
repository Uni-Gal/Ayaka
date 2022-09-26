use log::warn;
use std::{
    fmt::Debug,
    ops::{ControlFlow, Try},
};

#[doc(hidden)]
pub trait TryExt: Try {
    fn ok_or_log(self, msg: &str) -> Option<Self::Output>;

    fn unwrap_or_default_log(self, msg: &str) -> Self::Output;
}

impl<T: Try> TryExt for T
where
    T::Output: Default,
    T::Residual: Debug,
{
    fn ok_or_log(self, msg: &str) -> Option<Self::Output> {
        match self.branch() {
            ControlFlow::Continue(v) => Some(v),
            ControlFlow::Break(e) => {
                warn!("{msg}: {e:?}");
                None
            }
        }
    }

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
