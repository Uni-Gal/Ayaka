use fp_bindgen_support::{
    common::mem::FatPtr,
    host::{
        errors::{InvocationError, RuntimeError},
        mem::{
            deserialize_from_slice, export_to_guest_raw, import_from_guest_raw, serialize_to_vec,
        },
        r#async::resolve_async_value,
        runtime::RuntimeInstanceData,
    },
};
use gal_bindings::*;
use wasmer::{imports, Function, ImportObject, Instance, Module, Store, WasmerEnv};

pub struct Runtime {
    module: Module,
}

impl Runtime {
    pub fn new(wasm_module: impl AsRef<[u8]>) -> Result<Self, RuntimeError> {
        let store = Self::default_store();
        let module = Module::new(&store, wasm_module)?;
        Ok(Self { module })
    }

    fn default_store() -> wasmer::Store {
        let compiler = wasmer::Singlepass::default();
        let engine = wasmer::Universal::new(compiler).engine();
        Store::new(&engine)
    }

    pub fn dispatch(
        &self,
        name: String,
        args: Vec<RawValue>,
    ) -> Result<Option<RawValue>, InvocationError> {
        let name = serialize_to_vec(&name);
        let args = serialize_to_vec(&args);
        let result = self.dispatch_raw(name, args);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn dispatch_raw(&self, name: Vec<u8>, args: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let mut env = RuntimeInstanceData::default();
        let import_object = create_import_object(self.module.store(), &env);
        let instance = Instance::new(&self.module, &import_object).unwrap();
        env.init_with_instance(&instance).unwrap();
        let name = export_to_guest_raw(&env, name);
        let args = export_to_guest_raw(&env, args);
        let function = instance
            .exports
            .get_native_function::<(FatPtr, FatPtr), FatPtr>("__fp_gen_dispatch")
            .map_err(|_| InvocationError::FunctionNotExported)?;
        let result = function.call(name, args)?;
        let result = import_from_guest_raw(&env, result);
        Ok(result)
    }
}

fn create_import_object(store: &Store, env: &RuntimeInstanceData) -> ImportObject {
    imports! {
       "fp" => {
           "__fp_host_resolve_async_value" => Function::new_native_with_env(store, env.clone(), resolve_async_value) ,
        }
    }
}
