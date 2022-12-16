use anyhow::Result;
use ayaka_bindings_types::Record;
use ayaka_plugin::{Linker, RawModule};
use std::collections::HashMap;

pub fn register<M: RawModule>(store: &mut impl Linker<M>) -> Result<()> {
    let log_func = store.wrap(|(data,): (Record,)| {
        let target = format!("{}::<plugin>::{}", module_path!(), data.target);
        log::logger().log(
            &log::Record::builder()
                .level(data.level)
                .target(&target)
                .args(format_args!("{}", data.msg))
                .module_path(data.module_path.as_deref())
                .file(data.file.as_deref())
                .line(data.line)
                .build(),
        );
        Ok(())
    });
    let log_flush_func = store.wrap(|_: ()| {
        log::logger().flush();
        Ok(())
    });
    store.import(
        "log",
        HashMap::from([
            ("__log".to_string(), log_func),
            ("__log_flush".to_string(), log_flush_func),
        ]),
    )?;
    Ok(())
}
