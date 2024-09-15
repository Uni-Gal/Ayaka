#![feature(read_buf, core_io_borrowed_buf)]
#![feature(min_specialization)]

mod asset_resolver;
mod settings;

#[cfg(mobile)]
mod mobile;

use ayaka_model::{anyhow::Result, vfs::VfsPath, *};
use ayaka_plugin_wasmi::{WasmiLinker, WasmiModule};
use clap::Parser;
use serde::{Deserialize, Serialize};
use settings::*;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    net::TcpListener,
    path::PathBuf,
    pin::pin,
};
use tauri::{
    async_runtime::RwLock, command, utils::config::FrontendDist, webview::Url, App, AppHandle,
    Emitter, Manager, State, WebviewWindow,
};
use vfs_tar::TarFS;

type CommandResult<T> = Result<T, CommandError>;

#[derive(Debug, Default, Serialize)]
struct CommandError {
    msg: String,
}

impl<E: Into<tauri::Error>> From<E> for CommandError {
    default fn from(e: E) -> Self {
        Self {
            msg: e.into().to_string(),
        }
    }
}

impl Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.msg)
    }
}

#[command]
fn ayaka_version() -> &'static str {
    ayaka_model::version()
}

struct Storage {
    config: Vec<PathBuf>,
    dist_port: u16,
    model: RwLock<GameViewModel<FileSettingsManager, WasmiModule>>,
}

impl Storage {
    pub fn new(app: &App, config: Vec<PathBuf>, dist_port: u16) -> Self {
        let manager = FileSettingsManager::new(app);
        Self {
            config,
            dist_port,
            model: RwLock::new(GameViewModel::new(manager)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct GameInfo {
    pub title: String,
    pub author: String,
    pub props: HashMap<String, String>,
}

impl GameInfo {
    pub fn new(game: &Game) -> Self {
        Self {
            title: game.config.title.clone(),
            author: game.config.author.clone(),
            props: game.config.props.clone(),
        }
    }
}

#[command]
fn dist_port(storage: State<Storage>) -> u16 {
    storage.dist_port
}

#[cfg(desktop)]
async fn show_pick_files(handle: &AppHandle, _window: &WebviewWindow) -> Result<Vec<VfsPath>> {
    use tauri_plugin_dialog::DialogExt;

    let (tx, rx) = tokio::sync::oneshot::channel();
    tauri_plugin_dialog::FileDialogBuilder::new(handle.dialog().clone())
        .set_parent(&_window)
        .add_filter("Ayaka package", &["ayapack"])
        .pick_files(move |files| {
            let files = files
                .unwrap_or_default()
                .into_iter()
                .map(|p| {
                    TarFS::new_mmap(p.into_path()?)
                        .map(VfsPath::from)
                        .map_err(anyhow::Error::from)
                })
                .collect::<Vec<_>>();
            tx.send(files).ok();
        });
    rx.await?.into_iter().collect()
}

#[cfg(target_os = "ios")]
async fn show_pick_files(
    _handle: &AppHandle,
    window: &WebviewWindow,
) -> tauri::Result<Vec<VfsPath>> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    window.with_webview(move |webview| {
        let picked = file_picker_ios::pick_files(webview.view_controller(), &["ayapack"]);
        tx.send(picked).ok();
    })?;
    let picked = rx.await.map_err(|e| tauri::Error::Anyhow(e.into()))?;
    let paths = picked
        .map(|file| {
            TarFS::new(file)
                .map(VfsPath::from)
                .map_err(anyhow::Error::from)
        })
        .try_collect::<Vec<_>>()
        .await?;
    Ok(paths)
}

#[cfg(target_os = "android")]
async fn show_pick_files(handle: &AppHandle, _window: &WebviewWindow) -> Result<Vec<VfsPath>> {
    handle
        .state::<file_picker_android::PickerPlugin<tauri::Wry>>()
        .pick_files()?
        .into_iter()
        .map(|p| {
            TarFS::new_mmap(p)
                .map(VfsPath::from)
                .map_err(anyhow::Error::from)
        })
        .collect()
}

#[command]
async fn open_game(handle: AppHandle, storage: State<'_, Storage>) -> CommandResult<()> {
    let window = handle
        .get_webview_window("main")
        .expect("cannot get main window");

    const OPEN_STATUS_EVENT: &str = "ayaka://open_status";
    let mut model = storage.model.write().await;
    let linker = WasmiLinker::new(())?;
    let builder = ContextBuilder::<WasmiModule>::new(FrontendType::Html, linker);
    let builder = if storage.config.is_empty() {
        let files = show_pick_files(&handle, &window).await?;
        builder.with_vfs(&files)?
    } else {
        builder.with_paths(&storage.config)?
    };
    {
        let context = builder.open();
        let mut context = pin!(context);
        while let Some(status) = context.next().await {
            handle.emit(OPEN_STATUS_EVENT, status)?;
        }
        let context = model.open_game(context.await?);
        let mut context = pin!(context);
        while let Some(status) = context.next().await {
            handle.emit(OPEN_STATUS_EVENT, status)?;
        }
        context.await?;
    }

    asset_resolver::ROOT_PATH
        .set(model.context().root_path().clone())
        .expect("cannot set ROOT_PATH");

    #[cfg(desktop)]
    {
        window.set_title(&model.context().game().config.title)?;
    }

    Ok(())
}

#[command]
async fn get_settings(storage: State<'_, Storage>) -> CommandResult<Settings> {
    Ok(storage.model.read().await.settings().clone())
}

#[command]
async fn set_settings(settings: Settings, storage: State<'_, Storage>) -> CommandResult<()> {
    storage.model.write().await.set_settings(settings);
    Ok(())
}

#[command]
async fn get_records(storage: State<'_, Storage>) -> CommandResult<Vec<ActionText>> {
    Ok(storage.model.read().await.records_text().collect())
}

#[command]
async fn save_record_to(index: usize, storage: State<'_, Storage>) -> CommandResult<()> {
    storage.model.write().await.save_current_to(index);
    Ok(())
}

#[command]
async fn save_all(storage: State<'_, Storage>) -> CommandResult<()> {
    storage.model.read().await.save_settings()?;
    Ok(())
}

#[command]
async fn avaliable_locale(
    storage: State<'_, Storage>,
    locales: HashSet<Locale>,
) -> CommandResult<HashSet<Locale>> {
    Ok(storage
        .model
        .read()
        .await
        .avaliable_locale()
        .cloned()
        .collect::<HashSet<_>>()
        .intersection(&locales)
        .cloned()
        .collect())
}

#[command]
async fn choose_locale(
    storage: State<'_, Storage>,
    locales: HashSet<Locale>,
) -> CommandResult<Option<Locale>> {
    let locales = avaliable_locale(storage, locales).await?;
    let current = Locale::current();
    log::debug!("Choose {} from {:?}", current, locales);
    Ok(current.choose_from(&locales).cloned())
}

#[command]
async fn info(storage: State<'_, Storage>) -> CommandResult<Option<GameInfo>> {
    Ok(Some(GameInfo::new(
        storage.model.read().await.context().game(),
    )))
}

#[command]
async fn start_new(storage: State<'_, Storage>) -> CommandResult<()> {
    storage.model.write().await.init_new();
    Ok(())
}

#[command]
async fn start_record(index: usize, storage: State<'_, Storage>) -> CommandResult<()> {
    storage.model.write().await.init_context_by_index(index);
    Ok(())
}

#[command]
async fn next_run(storage: State<'_, Storage>) -> CommandResult<bool> {
    loop {
        let mut model = storage.model.write().await;
        if model.next_run() {
            let is_empty = {
                let action = model
                    .current_action()
                    .expect("current action cannot be None because next_run succeeds");
                match action {
                    Action::Empty => true,
                    Action::Custom(vars) => !vars.contains_key("video"),
                    _ => false,
                }
            };
            if !is_empty {
                return Ok(true);
            }
        } else {
            return Ok(false);
        }
    }
}

#[command]
async fn next_back_run(storage: State<'_, Storage>) -> CommandResult<bool> {
    Ok(storage.model.write().await.next_back_run())
}

#[command]
async fn current_visited(storage: State<'_, Storage>) -> CommandResult<bool> {
    Ok(storage.model.read().await.current_visited())
}

#[command]
async fn current_run(storage: State<'_, Storage>) -> CommandResult<Option<RawContext>> {
    Ok(storage.model.read().await.current_run().cloned())
}

#[command]
async fn current_action(
    storage: State<'_, Storage>,
) -> CommandResult<Option<(Action, Option<Action>)>> {
    Ok(storage.model.read().await.current_actions())
}

#[command]
async fn current_title(storage: State<'_, Storage>) -> CommandResult<Option<String>> {
    Ok(storage.model.read().await.current_title().cloned())
}

#[command]
async fn switch(i: usize, storage: State<'_, Storage>) -> CommandResult<()> {
    storage.model.write().await.switch(i);
    Ok(())
}

#[command]
async fn history(storage: State<'_, Storage>) -> CommandResult<Vec<(Action, Option<Action>)>> {
    Ok(storage.model.read().await.current_history().rev().collect())
}

#[derive(Debug, Parser)]
struct Config {
    #[clap()]
    config: Option<Vec<PathBuf>>,
}

pub fn run() -> tauri::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    listener.set_nonblocking(true)?;
    let port = listener.local_addr()?.port();
    let builder = tauri::Builder::default().plugin(asset_resolver::init(listener));
    #[cfg(desktop)]
    let builder = builder
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_dialog::init());
    #[cfg(target_os = "android")]
    let builder = builder.plugin(file_picker_android::init());
    let builder = builder.plugin(tauri_plugin_os::init());
    builder
        .setup(move |app| {
            #[cfg(target_os = "android")]
            {
                use android_logger::{Config, FilterBuilder};
                use log::LevelFilter;

                android_logger::init_once(
                    Config::default().with_filter(
                        FilterBuilder::new()
                            .filter_module("ayaka", LevelFilter::Debug)
                            .filter_module("tower_http", LevelFilter::Debug)
                            .filter(None, LevelFilter::Warn)
                            .build(),
                    ),
                );
            }
            #[cfg(desktop)]
            {
                use flexi_logger::{FileSpec, LogSpecification, Logger};

                let spec = LogSpecification::parse("warn,ayaka=debug,tower_http=debug")?;
                let log_handle = if cfg!(debug_assertions) {
                    Logger::with(spec)
                        .log_to_stdout()
                        .set_palette("b1;3;2;4;6".to_string())
                        .use_utc()
                        .start()?
                } else {
                    Logger::with(spec)
                        .log_to_file(
                            FileSpec::default()
                                .directory(
                                    app.path().app_log_dir().expect("cannot get app log dir"),
                                )
                                .basename("ayaka-gui"),
                        )
                        .use_utc()
                        .start()?
                };
                app.manage(log_handle);
            }
            #[cfg(debug_assertions)]
            {
                let window = app
                    .get_webview_window("main")
                    .expect("cannot get main window");
                window.open_devtools();
            }

            let args = Config::try_parse()?;
            let config = match args.config {
                Some(arr) => arr,
                _ => {
                    let current = std::env::current_exe()?;
                    let current = current
                        .parent()
                        .expect("cannot get parent dir of current exe");
                    let mut paths = vec![];

                    let data = current.join("data.ayapack");
                    if data.exists() {
                        paths.push(data);
                        paths.extend(
                            ('a'..='z')
                                .map(|c| current.join(format!("data.{}.ayapack", c)))
                                .filter(|p| p.exists()),
                        );
                    } else {
                        let current_config = current.join("config.yaml");
                        if current_config.exists() {
                            paths.push(current_config);
                        }
                    }

                    paths
                }
            };
            app.manage(Storage::new(app, config, port));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ayaka_version,
            dist_port,
            open_game,
            get_settings,
            set_settings,
            get_records,
            save_record_to,
            save_all,
            avaliable_locale,
            choose_locale,
            info,
            start_new,
            start_record,
            next_run,
            next_back_run,
            current_run,
            current_action,
            current_title,
            current_visited,
            switch,
            history,
        ])
        .run({
            let mut context = tauri::generate_context!();
            context.config_mut().build.frontend_dist = Some(FrontendDist::Url(
                Url::parse(&format!("http://127.0.0.1:{port}")).expect("cannot parse url"),
            ));
            context
        })?;
    Ok(())
}
