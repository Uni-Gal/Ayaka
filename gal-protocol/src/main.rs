use fp_bindgen::{prelude::*, types::*};
use gal_script::RawValue;
use std::collections::{BTreeMap, BTreeSet};

fp_import! {}

fp_export! {
    use RawValue;

    fn dispatch(name: String, args: Vec<RawValue>) -> Option<RawValue>;
}

const VERSION: &str = "0.1.0";
const AUTHORS: &str = r#"["Berrysoft <Strawberry_Str@hotmail.com>"]"#;
const NAME: &str = "gal-bindings";

fn main() {
    {
        let bindings_type = BindingsType::RustPlugin(RustPluginConfig {
            name: NAME,
            authors: AUTHORS,
            version: VERSION,
            dependencies: BTreeMap::from([(
                "fp-bindgen-support",
                CargoDependency {
                    version: Some("1.0"),
                    features: BTreeSet::from(["async", "guest"]),
                    ..CargoDependency::default()
                },
            )]),
        });
        fp_bindgen!(BindingConfig {
            bindings_type,
            path: "gal-bindings"
        });
    }

    {
        fp_bindgen!(BindingConfig {
            bindings_type: BindingsType::RustWasmerRuntime,
            path: "gal-plugin/src"
        });
    }
}
