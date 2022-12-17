#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#![feature(generators)]
#![feature(once_cell)]
#![feature(type_alias_impl_trait)]

mod asset_resolver;
mod settings;

use ayaka_runtime::{
    anyhow::{self, Result},
    log::{debug, info},
    settings::*,
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
    async_runtime::Mutex, command, utils::config::AppUrl, AppHandle, Manager, PathResolver, State,
    WindowUrl,
};
use trylog::macros::*;

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
    ayaka_runtime::version()
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "t", content = "data")]
enum OpenGameStatus {
    LoadProfile,
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

impl OpenGameStatus {
    pub fn emit(self, handle: &AppHandle) -> Result<(), tauri::Error> {
        handle.emit_all("ayaka://open_status", self)
    }
}

#[derive(Default)]
struct Storage {
    config: Vec<String>,
    dist_port: u16,
    manager: FileSettingsManager,
    records: Mutex<Vec<ActionRecord>>,
    context: Mutex<Option<Context>>,
    current: Mutex<Option<RawContext>>,
    settings: Mutex<Option<Settings>>,
    global_record: Mutex<Option<GlobalRecord>>,
}

impl Storage {
    pub fn new(resolver: &PathResolver, config: Vec<String>, dist_port: u16) -> Self {
        Self {
            config,
            dist_port,
            manager: FileSettingsManager::new(resolver),
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
fn dist_port(storage: State<Storage>) -> u16 {
    storage.dist_port
}

#[command]
async fn open_game(handle: AppHandle, storage: State<'_, Storage>) -> CommandResult<()> {
    let config = &storage.config;
    let context = Context::open(config, FrontendType::Html);
    pin_mut!(context);
    while let Some(status) = context.next().await {
        match status {
            OpenStatus::LoadProfile => {
                OpenGameStatus::LoadProfile.emit(&handle)?;
            }
            OpenStatus::CreateRuntime => OpenGameStatus::CreateRuntime.emit(&handle)?,
            OpenStatus::LoadPlugin(name, i, len) => {
                OpenGameStatus::LoadPlugin(name, i, len).emit(&handle)?
            }
            OpenStatus::GamePlugin => OpenGameStatus::GamePlugin.emit(&handle)?,
            OpenStatus::LoadResource => OpenGameStatus::LoadResource.emit(&handle)?,
            OpenStatus::LoadParagraph => OpenGameStatus::LoadParagraph.emit(&handle)?,
        }
    }
    let ctx = context.await?;
    asset_resolver::ROOT_PATH
        .set(ctx.root_path.clone())
        .unwrap();

    let window = handle.get_window("main").unwrap();
    window.set_title(&ctx.game.config.title)?;
    let settings = {
        OpenGameStatus::LoadSettings.emit(&handle)?;
        unwrap_or_default_log!(storage.manager.load_settings(), "Load settings failed")
    };
    *storage.settings.lock().await = Some(settings);

    OpenGameStatus::LoadGlobalRecords.emit(&handle)?;
    let global_record = unwrap_or_default_log!(
        storage.manager.load_global_record(&ctx.game.config.title),
        "Load global records failed"
    );
    *storage.global_record.lock().await = Some(global_record);

    OpenGameStatus::LoadRecords.emit(&handle)?;
    *storage.records.lock().await = unwrap_or_default_log!(
        storage.manager.load_records(&ctx.game.config.title),
        "Load records failed"
    );
    *storage.context.lock().await = Some(ctx);

    OpenGameStatus::Loaded.emit(&handle)?;
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
async fn get_records(storage: State<'_, Storage>) -> CommandResult<Vec<ActionText>> {
    let context = storage.context.lock().await;
    let context = context.as_ref().unwrap();
    let settings = storage.settings.lock().await;
    let settings = settings.as_ref().unwrap();
    let mut res = vec![];
    for record in storage.records.lock().await.iter() {
        let raw_ctx = record.history.last().unwrap();
        let action = context.get_action(&settings.lang, raw_ctx)?;
        if let Action::Text(action) = action {
            res.push(action);
        } else {
            unreachable!()
        }
    }
    Ok(res)
}

#[command]
async fn save_record_to(index: usize, storage: State<'_, Storage>) -> CommandResult<()> {
    let mut records = storage.records.lock().await;
    let record = storage
        .context
        .lock()
        .await
        .as_ref()
        .unwrap()
        .record
        .clone();
    if index >= records.len() {
        records.push(record);
    } else {
        records[index] = record;
    }
    Ok(())
}

#[command]
async fn save_all(storage: State<'_, Storage>) -> CommandResult<()> {
    let context = storage.context.lock().await;
    let game = &context.as_ref().unwrap().game.config.title;
    storage
        .manager
        .save_settings(storage.settings.lock().await.as_ref().unwrap())?;
    storage
        .manager
        .save_global_record(game, storage.global_record.lock().await.as_ref().unwrap())?;
    storage
        .manager
        .save_records(game, &storage.records.lock().await)?;
    Ok(())
}

#[command]
async fn avaliable_locale(
    storage: State<'_, Storage>,
    locales: HashSet<Locale>,
) -> CommandResult<HashSet<Locale>> {
    let avaliable = storage
        .context
        .lock()
        .await
        .as_ref()
        .unwrap()
        .game
        .paras
        .keys()
        .cloned()
        .collect();
    Ok(locales.intersection(&avaliable).cloned().collect())
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

#[command]
async fn info(storage: State<'_, Storage>) -> CommandResult<Option<GameInfo>> {
    let ctx = storage.context.lock().await;
    Ok(Some(GameInfo::new(&ctx.as_ref().unwrap().game)))
}

#[command]
async fn start_new(locale: Locale, storage: State<'_, Storage>) -> CommandResult<()> {
    storage.context.lock().await.as_mut().unwrap().init_new();
    info!("Init new context with locale {}.", locale);
    Ok(())
}

#[command]
async fn start_record(
    locale: Locale,
    index: usize,
    storage: State<'_, Storage>,
) -> CommandResult<()> {
    let record = storage.records.lock().await[index].clone();
    storage
        .context
        .lock()
        .await
        .as_mut()
        .unwrap()
        .init_context(record);
    info!("Init new context with locale {}.", locale);
    Ok(())
}

#[command]
async fn next_run(storage: State<'_, Storage>) -> CommandResult<bool> {
    loop {
        let mut context = storage.context.lock().await;
        let context = context.as_mut().unwrap();
        if let Some(raw_ctx) = context.next_run() {
            debug!("Next action: {:?}", raw_ctx);
            let is_empty = {
                let action = context.get_action(&context.game.config.base_lang, &raw_ctx)?;
                if let Action::Empty = action {
                    true
                } else if let Action::Custom(vars) = action {
                    vars.is_empty()
                } else {
                    false
                }
            };
            storage
                .global_record
                .lock()
                .await
                .as_mut()
                .unwrap()
                .update(&raw_ctx);
            *storage.current.lock().await = Some(raw_ctx);
            if !is_empty {
                return Ok(true);
            }
        } else {
            *storage.current.lock().await = None;
            return Ok(false);
        }
    }
}

#[command]
async fn next_back_run(storage: State<'_, Storage>) -> CommandResult<bool> {
    let mut context = storage.context.lock().await;
    let context = context.as_mut().unwrap();
    if let Some(raw_ctx) = context.next_back_run() {
        debug!("Last action: {:?}", raw_ctx);
        *storage.current.lock().await = Some(raw_ctx.clone());
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
        record.as_ref().unwrap().visited(raw_ctx)
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

fn get_actions(
    context: &Context,
    settings: &Settings,
    raw_ctx: &RawContext,
) -> (Action, Option<Action>) {
    let action = unwrap_or_default_log!(
        context.get_action(&settings.lang, raw_ctx),
        "Cannot get action"
    );
    let base_action = settings.sub_lang.as_ref().map(|sub_lang| {
        unwrap_or_default_log!(
            context.get_action(sub_lang, raw_ctx),
            "Cannot get sub action"
        )
    });
    (action, base_action)
}

#[command]
async fn current_action(
    storage: State<'_, Storage>,
) -> CommandResult<Option<(Action, Option<Action>)>> {
    let context = storage.context.lock().await;
    let context = context.as_ref().unwrap();
    let raw_ctx = storage.current.lock().await;
    let settings = storage.settings.lock().await;
    let settings = settings.as_ref().unwrap();
    Ok(raw_ctx
        .as_ref()
        .map(|raw_ctx| get_actions(context, settings, raw_ctx)))
}

#[command]
async fn current_title(storage: State<'_, Storage>) -> CommandResult<Option<String>> {
    let settings = storage.settings.lock().await;
    let settings = settings.as_ref().unwrap();
    Ok(storage
        .context
        .lock()
        .await
        .as_ref()
        .unwrap()
        .current_paragraph_title(&settings.lang)
        .cloned())
}

#[command]
async fn switch(i: usize, storage: State<'_, Storage>) -> CommandResult<()> {
    debug!("Switch {}", i);
    storage.context.lock().await.as_mut().unwrap().switch(i);
    Ok(())
}

#[command]
async fn history(storage: State<'_, Storage>) -> CommandResult<Vec<(Action, Option<Action>)>> {
    let context = storage.context.lock().await;
    let context = context.as_ref().unwrap();
    let settings = storage.settings.lock().await;
    let settings = settings.as_ref().unwrap();
    let mut hs = context
        .record
        .history
        .iter()
        .map(|raw_ctx| get_actions(context, settings, raw_ctx))
        .collect::<Vec<_>>();
    hs.reverse();
    debug!("Get history {:?}", hs);
    Ok(hs)
}

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    tauri::Builder::default()
        .plugin(asset_resolver::init(listener))
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(move |app| {
            let resolver = app.path_resolver();
            let spec = LogSpecification::parse("warn,ayaka=debug")?;
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

                    let data = current.join("data.frfs");
                    if data.exists() {
                        paths.push(data);
                        paths.extend(
                            ('a'..'z')
                                .into_iter()
                                .map(|c| current.join(format!("data.{}.frfs", c)))
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
