#[cfg(target_arch = "wasm32")]
use gal_bindings::*;

#[cfg(target_arch = "wasm32")]
#[fp_export_impl(gal_bindings)]
fn dispatch(_name: String, _args: Vec<RawValue>) -> Option<RawValue> {
    None
}
