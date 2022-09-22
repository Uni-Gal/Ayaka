#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use ayaka_runtime::{
    anyhow::{self, anyhow, Result},
    log::{debug, error, info, warn},
    *,
};
use flexi_logger::{FileSpec, LogSpecification, Logger};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};
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

#[command]
fn ayaka_version() -> &'static str {
    ayaka_runtime::version()
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "t", content = "data")]
enum OpenGameStatus {
    LoadProfile(String),
    CreateRuntime,
    LoadPlugin(String, usize, usize),
    GamePlugin,
    LoadResource,
    LoadParagraph,
    LoadSettings,
    LoadGlobalRecords,
    LoadRecords,
    Loaded,
}

fn emit_open_status(
    handle: &AppHandle,
    status: OpenGameStatus,
) -> std::result::Result<(), tauri::Error> {
    handle.emit_all("ayaka://open_status", status)
}

#[command]
async fn open_game(handle: AppHandle, storage: State<'_, Storage>) -> CommandResult<()> {
    let config = &storage.config;
    let context = Context::open(config, FrontendType::Html);
    pin_mut!(context);
    while let Some(status) = context.next().await {
        match status {
            OpenStatus::LoadProfile => {
                emit_open_status(&handle, OpenGameStatus::LoadProfile(config.clone()))?
            }
            OpenStatus::CreateRuntime => emit_open_status(&handle, OpenGameStatus::CreateRuntime)?,
            OpenStatus::LoadPlugin(name, i, len) => {
                emit_open_status(&handle, OpenGameStatus::LoadPlugin(name, i, len))?
            }
            OpenStatus::GamePlugin => emit_open_status(&handle, OpenGameStatus::GamePlugin)?,
            OpenStatus::LoadResource => emit_open_status(&handle, OpenGameStatus::LoadResource)?,
            OpenStatus::LoadParagraph => emit_open_status(&handle, OpenGameStatus::LoadParagraph)?,
        }
    }
    let ctx = context.await?;

    let window = handle.get_window("main").unwrap();
    window.set_title(&ctx.game.config.title)?;
    let settings = {
        emit_open_status(&handle, OpenGameStatus::LoadSettings)?;
        load_settings(&storage.ident).unwrap_or_else(|e| {
            warn!("Load settings failed: {}", e);
            Settings::new()
        })
    };
    *storage.settings.lock().await = Some(settings);

    emit_open_status(&handle, OpenGameStatus::LoadGlobalRecords)?;
    let global_record =
        load_global_record(&storage.ident, &ctx.game.config.title).unwrap_or_else(|e| {
            warn!("Load global records failed: {}", e);
            Default::default()
        });
    *storage.global_record.lock().await = Some(global_record);

    emit_open_status(&handle, OpenGameStatus::LoadRecords)?;
    *storage.records.lock().await = load_records(&storage.ident, &ctx.game.config.title)
        .unwrap_or_else(|e| {
            warn!("Load records failed: {}", e);
            Default::default()
        });
    *storage.context.lock().await = Some(ctx);

    emit_open_status(&handle, OpenGameStatus::Loaded)?;
    Ok(())
}

#[command]
async fn get_settings(storage: State<'_, Storage>) -> CommandResult<Option<Settings>> {
    Ok(storage.settings.lock().await.clone())
}

#[command]
async fn set_settings(settings: Settings, storage: State<'_, Storage>) -> CommandResult<()> {
    *storage.settings.lock().await = Some(settings);
    Ok(())
}

#[command]
async fn get_records(storage: State<'_, Storage>) -> CommandResult<Vec<ActionRecord>> {
    Ok(storage.records.lock().await.clone())
}

#[command]
async fn save_record_to(index: usize, storage: State<'_, Storage>) -> CommandResult<()> {
    let mut records = storage.records.lock().await;
    if let Some(record) = storage
        .context
        .lock()
        .await
        .as_ref()
        .map(|ctx| ctx.record.clone())
    {
        if index >= records.len() {
            records.push(record);
        } else {
            records[index] = record;
        }
    }
    Ok(())
}

#[command]
async fn save_all(storage: State<'_, Storage>) -> CommandResult<()> {
    if let Some(context) = storage.context.lock().await.as_ref() {
        let game = &context.game.config.title;
        if let Some(settings) = storage.settings.lock().await.as_ref() {
            save_settings(&storage.ident, settings)?;
        }
        if let Some(record) = storage.global_record.lock().await.as_ref() {
            save_global_record(&storage.ident, game, record)?;
        }
        save_records(&storage.ident, game, &storage.records.lock().await)?;
    }
    Ok(())
}

#[command]
async fn avaliable_locale(
    storage: State<'_, Storage>,
    locales: HashSet<Locale>,
) -> CommandResult<HashSet<Locale>> {
    if let Some(context) = storage.context.lock().await.as_ref() {
        let avaliable = context.game.paras.keys().cloned().collect();
        Ok(locales.intersection(&avaliable).cloned().collect())
    } else {
        Ok(locales)
    }
}

#[command]
async fn choose_locale(
    storage: State<'_, Storage>,
    locales: HashSet<Locale>,
) -> CommandResult<Option<Locale>> {
    let locales = avaliable_locale(storage, locales).await?;
    let current = Locale::current();
    debug!("Choose {} from {:?}", current, locales);
    Ok(current.choose_from(&locales).cloned())
}

#[derive(Default)]
struct Storage {
    ident: String,
    config: String,
    records: Mutex<Vec<ActionRecord>>,
    context: Mutex<Option<Context>>,
    current: Mutex<Option<RawContext>>,
    settings: Mutex<Option<Settings>>,
    global_record: Mutex<Option<GlobalRecord>>,
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
async fn info(storage: State<'_, Storage>) -> CommandResult<Option<GameInfo>> {
    let ctx = storage.context.lock().await;
    if let Some(ctx) = ctx.as_ref() {
        Ok(Some(GameInfo::new(&ctx.game)))
    } else {
        warn!("Game hasn't been loaded.");
        Ok(None)
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
async fn start_record(
    locale: Locale,
    index: usize,
    storage: State<'_, Storage>,
) -> CommandResult<()> {
    if let Some(ctx) = storage.context.lock().await.as_mut() {
        let raw_ctx = storage.records.lock().await[index].clone();
        ctx.init_context(raw_ctx);
        info!("Init new context with locale {}.", locale);
    } else {
        warn!("Game hasn't been loaded.")
    }
    Ok(())
}

#[command]
async fn next_run(storage: State<'_, Storage>) -> CommandResult<bool> {
    loop {
        let mut context = storage.context.lock().await;
        let raw_ctx = context.as_mut().and_then(|context| context.next_run());
        if let Some(raw_ctx) = raw_ctx {
            debug!("Next action: {:?}", raw_ctx);
            let is_empty = context
                .as_mut()
                .map(|context| {
                    matches!(
                        context
                            .get_action(&context.game.config.base_lang, &raw_ctx)
                            .unwrap_or_default(),
                        Action::Empty
                    )
                })
                .unwrap_or(true);
            let mut record = storage.global_record.lock().await;
            if let Some(record) = record.as_mut() {
                record.update(&raw_ctx);
                *storage.current.lock().await = Some(raw_ctx);
            }
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
    let mut context = storage.context.lock().await;
    let raw_ctx = context.as_mut().and_then(|context| context.next_back_run());
    if let Some(raw_ctx) = raw_ctx {
        debug!("Last action: {:?}", raw_ctx);
        Ok(true)
    } else {
        debug!("No action in the history.");
        Ok(false)
    }
}

#[command]
async fn current_visited(storage: State<'_, Storage>) -> CommandResult<bool> {
    let raw_ctx = storage.current.lock().await;
    let visited = if let Some(raw_ctx) = raw_ctx.as_ref() {
        let record = storage.global_record.lock().await;
        record
            .as_ref()
            .map(|record| record.visited(raw_ctx))
            .unwrap_or_default()
    } else {
        false
    };
    Ok(visited)
}

#[command]
async fn current_run(storage: State<'_, Storage>) -> CommandResult<Option<RawContext>> {
    let raw_ctx = storage.current.lock().await;
    Ok(raw_ctx.as_ref().cloned())
}

#[command]
async fn current_action(storage: State<'_, Storage>) -> CommandResult<Option<Action>> {
    let context = storage.context.lock().await;
    let raw_ctx = storage.current.lock().await;
    let settings = storage.settings.lock().await;
    let lang = settings
        .as_ref()
        .map(|settings| settings.lang.clone())
        .unwrap_or_else(Locale::current);
    Ok(context.as_ref().and_then(|context| {
        raw_ctx.as_ref().map(|raw_ctx| {
            context.get_action(&lang, raw_ctx).unwrap_or_else(|e| {
                error!("Cannot get action: {}", e);
                Action::default()
            })
        })
    }))
}

#[command]
async fn current_title(storage: State<'_, Storage>) -> CommandResult<Option<String>> {
    let settings = storage.settings.lock().await;
    let lang = settings
        .as_ref()
        .map(|settings| settings.lang.clone())
        .unwrap_or_else(Locale::current);
    Ok(storage
        .context
        .lock()
        .await
        .as_ref()
        .and_then(|context| context.current_paragraph_title(&lang).cloned()))
}

#[command]
async fn switch(i: usize, storage: State<'_, Storage>) -> CommandResult<()> {
    debug!("Switch {}", i);
    let mut context = storage.context.lock().await;
    let context = context
        .as_mut()
        .ok_or_else(|| anyhow!("Context not initialized."))?;
    context.switch(i);
    Ok(())
}

#[command]
async fn history(storage: State<'_, Storage>) -> CommandResult<Vec<Action>> {
    let context = storage.context.lock().await;
    let settings = storage.settings.lock().await;
    let lang = settings
        .as_ref()
        .map(|settings| settings.lang.clone())
        .unwrap_or_else(Locale::current);
    let mut hs = context
        .as_ref()
        .map(|context| {
            context
                .record
                .history
                .iter()
                .map(|raw_ctx| {
                    context.get_action(&lang, raw_ctx).unwrap_or_else(|e| {
                        error!("Cannot get action: {}", e);
                        Action::default()
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    hs.reverse();
    debug!("Get history {:?}", hs);
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
            let log_handle = if cfg!(debug_assertions) {
                Logger::with(LogSpecification::parse("warn,ayaka=debug,ayalog=debug")?)
                    .log_to_stdout()
                    .set_palette("b1;3;2;4;6".to_string())
                    .use_utc()
                    .start()?
            } else {
                Logger::with(LogSpecification::parse("info,wasmer=warn")?)
                    .log_to_file(
                        FileSpec::default()
                            .directory(app.path_resolver().log_dir().unwrap())
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
            let matches = app.get_cli_matches()?;
            let config = matches.args["config"]
                .value
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| {
                    std::env::current_exe()
                        .unwrap()
                        .parent()
                        .unwrap()
                        .join("config.yaml")
                        .to_string_lossy()
                        .into_owned()
                });
            app.manage(Storage::new(ident, config));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ayaka_version,
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
        .run(tauri::generate_context!())?;
    Ok(())
}
