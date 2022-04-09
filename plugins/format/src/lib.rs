#[cfg(target_arch = "wasm32")]
use gal_bindings::*;
#[cfg(not(target_arch = "wasm32"))]
use gal_script::*;

use rt_format::*;

#[cfg(target_arch = "wasm32")]
#[fp_export_impl(gal_bindings)]
fn dispatch(name: String, args: Vec<RawValue>) -> Option<RawValue> {
    match name {
        "fmt" => Some(fmt(&args)),
        _ => None,
    }
}

fn fmt(args: &[RawValue]) -> RawValue {
    RawValue::Str(
        ParsedFormat::parse(
            &args[0].get_str(),
            &args[1..]
                .iter()
                .map(|v| v.get_str().into_owned())
                .collect::<Vec<_>>(),
            &[],
        )
        .unwrap()
        .to_string(),
    )
}
