mod bindings;
mod types;

pub use bindings::Runtime;

use fp_bindgen_support::common::mem::FatPtr;

// avoid link errors
#[no_mangle]
fn __fp_host_resolve_async_value(_async_value_ptr: FatPtr, _result_ptr: FatPtr) {
    unimplemented!()
}

#[cfg(test)]
mod test {
    use super::Runtime;
    use std::io::{BufReader, Read};

    #[test]
    fn format() {
        let path = format!(
            "{}/../target/wasm32-unknown-unknown/debug/format.wasm",
            env!("CARGO_MANIFEST_DIR")
        );
        println!("{}", path);
        let reader = std::fs::File::open(path).unwrap();
        let mut reader = BufReader::new(reader);
        let mut buf = vec![];
        reader.read_to_end(&mut buf).unwrap();
        let runtime = Runtime::new(&buf).unwrap();
        assert_eq!(runtime.dispatch("fmt".into(), vec![]).unwrap(), None);
    }
}
