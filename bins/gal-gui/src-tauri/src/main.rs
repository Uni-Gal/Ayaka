#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use flexi_logger::{FileSpec, LogSpecification, Logger};
use gal_runtime::{
    anyhow::{self, anyhow, Result},
    load_records, load_settings,
    log::{info, warn},
    save_records, save_settings, tokio,
    tokio_stream::StreamExt,
    *,
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

#[derive(Debug, Serialize)]
struct FullSettings {
    settings: Settings,
    contexts: Vec<RawContext>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "t", content = "data")]
enum OpenGameStatus {
    LoadSettings,
    LoadProfile(String),
    CreateRuntime,
    LoadPlugin(String, usize, usize),
    Loaded,
    LoadRecords,
}

fn emit_open_status(
    handle: &AppHandle,
    status: OpenGameStatus,
) -> std::result::Result<(), tauri::Error> {
    handle.emit_all("gal://open_status", status)
}

#[command]
async fn open_game(handle: AppHandle, storage: State<'_, Storage>) -> CommandResult<()> {
    {
        emit_open_status(&handle, OpenGameStatus::LoadSettings)?;
        *storage.settings.lock().await =
            Some(load_settings(&storage.ident).await.unwrap_or_else(|e| {
                warn!("Load settings failed: {}", e);
                Default::default()
            }));
    }
    {
        let config = &storage.config;
        let context = Context::open(config, FrontendType::Html);
        tokio::pin!(context);
        while let Some(status) = context.next().await {
            match status {
                OpenStatus::LoadProfile => {
                    emit_open_status(&handle, OpenGameStatus::LoadProfile(config.clone()))?
                }
                OpenStatus::CreateRuntime => {
                    emit_open_status(&handle, OpenGameStatus::CreateRuntime)?
                }
                OpenStatus::LoadPlugin(name, i, len) => {
                    emit_open_status(&handle, OpenGameStatus::LoadPlugin(name, i, len))?
                }
            }
        }
        let ctx = context.await??;
        let window = handle.get_window("main").unwrap();
        window.set_title(&ctx.game.title)?;
        emit_open_status(&handle, OpenGameStatus::LoadRecords)?;
        *storage.records.lock().await = load_records(&storage.ident, &ctx.game.title)
            .await
            .unwrap_or_else(|e| {
                warn!("Load records failed: {}", e);
                Default::default()
            });
        *storage.context.lock().await = Some(ctx);
        emit_open_status(&handle, OpenGameStatus::Loaded)?;
    }
    Ok(())
}

#[command]
async fn get_settings(storage: State<'_, Storage>) -> CommandResult<Option<Settings>> {
    Ok(storage.settings.lock().await.as_ref().cloned())
}

#[command]
async fn set_settings(settings: Settings, storage: State<'_, Storage>) -> CommandResult<()> {
    if let Some(context) = storage.context.lock().await.as_mut() {
        context.set_locale(&settings.lang);
    }
    *storage.settings.lock().await = Some(settings);
    Ok(())
}

#[command]
async fn get_records(storage: State<'_, Storage>) -> CommandResult<Vec<RawContext>> {
    Ok(storage.records.lock().await.clone())
}

#[command]
async fn save_record_to(index: usize, storage: State<'_, Storage>) -> CommandResult<()> {
    let mut records = storage.records.lock().await;
    if let Some(ctx) = storage
        .context
        .lock()
        .await
        .as_ref()
        .map(|ctx| ctx.ctx.clone())
    {
        if index >= records.len() {
            records.push(ctx);
        } else {
            records[index] = ctx;
        }
    }
    Ok(())
}

#[command]
async fn save_all(storage: State<'_, Storage>) -> CommandResult<()> {
    if let Some(settings) = storage.settings.lock().await.as_ref() {
        save_settings(&storage.ident, settings).await?;
    }
    if let Some(context) = storage.context.lock().await.as_ref() {
        let game = &context.game.title;
        save_records(&storage.ident, game, &storage.records.lock().await).await?;
    }
    Ok(())
}

#[command]
fn choose_locale(locales: Vec<LocaleBuf>) -> CommandResult<Option<LocaleBuf>> {
    let current = Locale::current();
    info!("Choose {} from {:?}", current, locales);
    Ok(current.choose_from(&locales)?)
}

#[command]
fn locale_native_name(loc: LocaleBuf) -> CommandResult<String> {
    let name = loc.native_name()?;
    Ok(name)
}

#[derive(Default)]
struct Storage {
    ident: String,
    config: String,
    settings: Mutex<Option<Settings>>,
    records: Mutex<Vec<RawContext>>,
    context: Mutex<Option<Context>>,
    action: Mutex<Option<Action>>,
}

impl Storage {
    pub fn new(ident: impl Into<String>, config: impl Into<String>) -> Self {
        Self {
            ident: ident.into(),
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
async fn start_new(locale: LocaleBuf, storage: State<'_, Storage>) -> CommandResult<()> {
    if let Some(ctx) = storage.context.lock().await.as_mut() {
        ctx.init_new();
        info!("Init new context with locale {}.", locale);
    } else {
        warn!("Game hasn't been loaded.")
    }
    Ok(())
}

#[command]
async fn start_record(
    locale: LocaleBuf,
    index: usize,
    storage: State<'_, Storage>,
) -> CommandResult<()> {
    if let Some(ctx) = storage.context.lock().await.as_mut() {
        let raw_ctx = storage.records.lock().await[index].clone();
        let last_line = raw_ctx.history.last().unwrap();
        *storage.action.lock().await = Some(last_line.clone());
        ctx.init_context(raw_ctx);
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
async fn current_run(storage: State<'_, Storage>) -> CommandResult<Option<Action>> {
    Ok(storage
        .action
        .lock()
        .await
        .as_ref()
        .map(|action| action.clone()))
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
async fn history(storage: State<'_, Storage>) -> CommandResult<Vec<Action>> {
    let mut hs = storage
        .context
        .lock()
        .await
        .as_ref()
        .map(|context| context.ctx.history.clone())
        .unwrap_or_default();
    hs.reverse();
    info!("Get history {:?}", hs);
    Ok(hs)
}

fn main() -> Result<()> {
    let port =
        portpicker::pick_unused_port().ok_or_else(|| anyhow!("failed to find unused port"))?;
    info!("Picked port {}", port);
    tauri::Builder::default()
        .plugin(tauri_plugin_localhost::Builder::new(port).build())
        .setup(|app| {
            let ident = app.config().tauri.bundle.identifier.clone();
            let logspec = LogSpecification::parse("info,wasmer=warn")?;
            let log_handle = if cfg!(debug_assertions) {
                Logger::with(logspec)
                    .log_to_stdout()
                    .set_palette("b1;3;2;4;6".to_string())
                    .use_utc()
                    .start()?
            } else {
                Logger::with(logspec)
                    .log_to_file(
                        FileSpec::default()
                            .directory(app.path_resolver().log_dir().unwrap())
                            .basename("gal-gui"),
                    )
                    .use_utc()
                    .start()?
            };
            app.manage(log_handle);
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
            app.manage(Storage::new(ident, config));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_game,
            get_settings,
            set_settings,
            get_records,
            save_record_to,
            save_all,
            choose_locale,
            locale_native_name,
            info,
            start_new,
            start_record,
            next_run,
            current_run,
            switch,
            history,
        ])
        .run(tauri::generate_context!())?;
    Ok(())
}
