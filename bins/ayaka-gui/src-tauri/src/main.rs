#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#![feature(once_cell)]
#![feature(return_position_impl_trait_in_trait)]
#![allow(incomplete_features)]

mod asset_resolver;
mod settings;

use ayaka_model::{
    anyhow::{self, Result},
    *,
};
use flexi_logger::{FileSpec, LogSpecification, Logger};
use serde::{Deserialize, Serialize};
use settings::*;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    net::TcpListener,
};
use tauri::{
    async_runtime::RwLock, command, utils::config::AppUrl, AppHandle, Manager, PathResolver, State,
    WindowUrl,
};

type CommandResult<T> = Result<T, CommandError>;

#[derive(Debug, Default, Serialize)]
struct CommandError {
    msg: String,
}

impl<E: Into<anyhow::Error>> From<E> for CommandError {
    fn from(e: E) -> Self {
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
    config: Vec<String>,
    dist_port: u16,
    model: RwLock<GameViewModel<FileSettingsManager>>,
}

impl Storage {
    pub fn new(resolver: &PathResolver, config: Vec<String>, dist_port: u16) -> Self {
        let manager = FileSettingsManager::new(resolver);
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

#[command]
async fn open_game(handle: AppHandle, storage: State<'_, Storage>) -> CommandResult<()> {
    let config = &storage.config;
    let mut model = storage.model.write().await;
    {
        let context = model.open_game(config, FrontendType::Html);
        pin_mut!(context);
        while let Some(status) = context.next().await {
            handle.emit_all("ayaka://open_status", status)?;
        }
        context.await?;
    }

    asset_resolver::ROOT_PATH
        .set(model.context().root_path().clone())
        .unwrap();

    let window = handle.get_window("main").unwrap();
    window.set_title(&model.context().game().config.title)?;

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
                let action = model.current_action().unwrap();
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

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    tauri::Builder::default()
        .plugin(asset_resolver::init(listener))
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(move |app| {
            let resolver = app.path_resolver();
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
                            .directory(resolver.app_log_dir().unwrap())
                            .basename("ayaka-gui"),
                    )
                    .use_utc()
                    .start()?
            };
            app.manage(log_handle);
            #[cfg(debug_assertions)]
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
            }

            use serde_json::Value;

            let matches = app.get_cli_matches()?;
            let config = match &matches.args["config"].value {
                Value::String(s) => vec![s.to_string()],
                Value::Array(arr) => arr
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>(),
                _ => {
                    let current = std::env::current_exe().unwrap();
                    let current = current.parent().unwrap();
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
                        paths.push(current.join("config.yaml"));
                    }

                    paths
                        .into_iter()
                        .map(|p| p.to_string_lossy().into_owned())
                        .collect()
                }
            };
            app.manage(Storage::new(&resolver, config, port));
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
            context.config_mut().build.dist_dir = AppUrl::Url(WindowUrl::External(
                format!("http://127.0.0.1:{port}").parse().unwrap(),
            ));
            context
        })?;
    Ok(())
}
