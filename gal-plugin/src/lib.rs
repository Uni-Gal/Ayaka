#![allow(unused_imports)]

mod bindings;
mod types;

pub use bindings::Runtime;

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
        let reader = std::fs::File::open(path).unwrap();
        let mut reader = BufReader::new(reader);
        let mut buf = vec![];
        reader.read_to_end(&mut buf).unwrap();
        let runtime = Runtime::new(&buf).unwrap();
        assert_eq!(runtime.dispatch("fmt".into(), vec![]).unwrap(), None);
    }
}
