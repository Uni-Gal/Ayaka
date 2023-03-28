#[cfg(target_os = "android")]
mod r#impl {
    use std::path::{Path, PathBuf};
    use tauri::App;

    pub struct DirResolver(String);

    impl DirResolver {
        pub fn new(app: &App) -> Self {
            Self(app.config().tauri.bundle.identifier.clone())
        }

        fn app_data_dir_base() -> &'static Path {
            Path::new("/data/app")
        }

        pub fn app_local_data_dir(&self) -> Option<PathBuf> {
            Some(Self::app_data_dir_base().join(&self.0))
        }

        pub fn app_config_dir(&self) -> Option<PathBuf> {
            Some(Self::app_data_dir_base().join(&self.0))
        }
    }
}

#[cfg(not(target_os = "android"))]
mod r#impl {
    use std::ops::Deref;
    use tauri::{App, PathResolver};

    pub struct DirResolver(PathResolver);

    impl DirResolver {
        pub fn new(app: &App) -> Self {
            Self(app.path_resolver())
        }
    }

    impl Deref for DirResolver {
        type Target = PathResolver;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}

pub use r#impl::*;
