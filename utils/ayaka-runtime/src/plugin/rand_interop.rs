use anyhow::{anyhow, Result};
use ayaka_plugin::{Linker, RawModule};
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

static RNG: LazyLock<Mutex<StdRng>> = LazyLock::new(|| Mutex::new(StdRng::from_os_rng()));

pub fn register<M: RawModule>(store: &mut impl Linker<M>) -> Result<()> {
    let rnd_func = store.wrap(|(start, end): (i64, i64)| {
        RNG.lock()
            .map(|mut rng| rng.random_range(start..end))
            .map_err(|_| anyhow!("Cannot lock random engine"))
    });
    store.import("rand", HashMap::from([("__rnd".to_string(), rnd_func)]))?;
    Ok(())
}
