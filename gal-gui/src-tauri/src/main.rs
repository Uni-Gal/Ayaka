#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use flexi_logger::{FileSpec, LogSpecification, Logger};
use gal_runtime::{
    anyhow::{self, anyhow, Result},
    log::{info, warn},
    tokio,
    tokio_stream::StreamExt,
    Action, ActionData, ActionHistoryData, Context, FrontendType, Locale, OpenStatus, RawValue,
};
use serde::Serialize;
use serde_json::json;
use std::fmt::Display;
use tauri::{async_runtime::Mutex, command, AppHandle, Manager, State};

type CommandResult<T> = std::result::Result<T, CommandError>;

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

#[derive(Debug, Clone, Serialize)]
struct OpenGameStatus {
    t: OpenGameStatusType,
    data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, Serialize)]
enum OpenGameStatusType {
    LoadProfile,
    CreateRuntime,
    LoadPlugin,
    Loaded,
}

impl OpenGameStatus {
    pub fn load_profile(path: impl Into<String>) -> Self {
        Self {
            t: OpenGameStatusType::LoadProfile,
            data: Some(json! {path.into()}),
        }
    }

    pub fn create_runtime() -> Self {
        Self {
            t: OpenGameStatusType::CreateRuntime,
            data: None,
        }
    }

    pub fn load_plugin(name: impl Into<String>, i: usize, len: usize) -> Self {
        Self {
            t: OpenGameStatusType::LoadPlugin,
            data: Some(json! {[name.into(), i, len]}),
        }
    }

    pub fn loaded() -> Self {
        Self {
            t: OpenGameStatusType::Loaded,
            data: None,
        }
    }
}

#[command]
async fn open_game(handle: AppHandle, storage: State<'_, Storage>) -> CommandResult<()> {
    let window = handle.get_window("main").unwrap();
    let config = &storage.config;
    let open = Context::open(config, FrontendType::Html);
    tokio::pin!(open);
    while let Some(status) = open.try_next().await? {
        match status {
            OpenStatus::LoadProfile => handle.emit_all(
                "gal://open_status",
                OpenGameStatus::load_profile(config.clone()),
            )?,
            OpenStatus::CreateRuntime => {
                handle.emit_all("gal://open_status", OpenGameStatus::create_runtime())?
            }
            OpenStatus::LoadPlugin(name, i, len) => handle.emit_all(
                "gal://open_status",
                OpenGameStatus::load_plugin(name, i, len),
            )?,
            OpenStatus::Loaded(ctx) => {
                window.set_title(&ctx.game.title)?;
                handle.emit_all("gal://open_status", OpenGameStatus::loaded())?;
                info!("Loaded config \"{}\"", config.escape_default());
                *storage.context.lock().await = Some(ctx);
            }
        }
    }
    Ok(())
}

#[command]
fn choose_locale(locales: Vec<Locale>) -> CommandResult<Option<Locale>> {
    let current = Locale::current();
    info!("Choose {} from {:?}", current, locales);
    Ok(current.choose_from(&locales)?)
}

#[command]
fn locale_native_name(loc: Locale) -> CommandResult<String> {
    let name = loc.native_name()?;
    Ok(name)
}

#[derive(Default)]
struct Storage {
    config: String,
    context: Mutex<Option<Context>>,
    action: Mutex<Option<Action>>,
}

impl Storage {
    pub fn new(config: impl Into<String>) -> Self {
        Self {
            config: config.into(),
            ..Default::default()
        }
    }
}

#[command]
async fn info(storage: State<'_, Storage>) -> CommandResult<serde_json::Value> {
    let ctx = storage.context.lock().await;
    if let Some(ctx) = ctx.as_ref() {
        Ok(json!({
            "title": ctx.game.title,
            "author": ctx.game.author,
        }))
    } else {
        warn!("Game hasn't been loaded.");
        Ok(json!({}))
    }
}

#[command]
async fn start_new(locale: Locale, storage: State<'_, Storage>) -> CommandResult<()> {
    if let Some(ctx) = storage.context.lock().await.as_mut() {
        ctx.init_new();
        info!("Init new context with locale {}.", locale);
    } else {
        warn!("Game hasn't been loaded.")
    }
    Ok(())
}

#[command]
async fn next_run(storage: State<'_, Storage>) -> CommandResult<bool> {
    let mut context = storage.context.lock().await;
    let action = context.as_mut().and_then(|context| context.next_run());
    if let Some(action) = action {
        info!("Next action: {:?}", action);
        *storage.action.lock().await = Some(action);
        Ok(true)
    } else {
        info!("No action left.");
        *storage.action.lock().await = None;
        Ok(false)
    }
}

#[command]
async fn current_run(storage: State<'_, Storage>) -> CommandResult<Option<ActionData>> {
    Ok(storage
        .action
        .lock()
        .await
        .as_ref()
        .map(|action| action.data.clone()))
}

#[command]
async fn switch(i: usize, storage: State<'_, Storage>) -> CommandResult<RawValue> {
    info!("Switch {}", i);
    let mut context = storage.context.lock().await;
    let context = context
        .as_mut()
        .ok_or_else(|| anyhow!("Context not initialized."))?;
    let action = storage.action.lock().await;
    let action = action
        .as_ref()
        .and_then(|action| action.switch_actions.get(i))
        .ok_or_else(|| anyhow!("Index error: {}", i))?;
    Ok(context.call(action))
}

#[command]
async fn history(storage: State<'_, Storage>) -> CommandResult<Vec<ActionHistoryData>> {
    Ok(storage
        .context
        .lock()
        .await
        .as_ref()
        .map(|context| context.ctx.history.clone())
        .unwrap_or_default())
}

fn main() -> Result<()> {
    let _log_handle = if cfg!(debug_assertions) {
        Logger::with(LogSpecification::info())
            .log_to_stdout()
            .set_palette("b1;3;2;4;6".to_string())
            .start()?
    } else {
        Logger::with(LogSpecification::info())
            .log_to_file(FileSpec::default().directory("logs").basename("gal-gui"))
            .start()?
    };
    let port =
        portpicker::pick_unused_port().ok_or_else(|| anyhow!("failed to find unused port"))?;
    info!("Picked port {}", port);
    tauri::Builder::default()
        .plugin(tauri_plugin_localhost::Builder::new(port).build())
        .setup(|app| {
            let window = app.get_window("main").unwrap();
            #[cfg(debug_assertions)]
            window.open_devtools();
            #[cfg(not(debug_assertions))]
            window.center()?;
            let matches = app.get_cli_matches()?;
            let config = matches.args["config"]
                .value
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_default();
            app.manage(Storage::new(config));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_game,
            choose_locale,
            locale_native_name,
            info,
            start_new,
            next_run,
            current_run,
            switch
        ])
        .run(tauri::generate_context!())?;
    Ok(())
}
