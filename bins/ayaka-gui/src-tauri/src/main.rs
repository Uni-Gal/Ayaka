#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#![feature(absolute_path)]

use ayaka_runtime::{
    anyhow::{self, anyhow, Result},
    log::{debug, info},
    *,
};
use flexi_logger::{FileSpec, LogSpecification, Logger};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    path::PathBuf,
};
use tauri::{
    async_runtime::Mutex, command, utils::config::AppUrl, AppHandle, Manager, State, WindowUrl,
};
use tiny_http::{Header, Server};
use trylog::TryLog;

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

impl OpenGameStatus {
    pub fn emit(self, handle: &AppHandle) -> std::result::Result<(), tauri::Error> {
        handle.emit_all("ayaka://open_status", self)
    }
}

#[derive(Default)]
struct Storage {
    ident: String,
    config: String,
    root_path: PathBuf,
    records: Mutex<Vec<ActionRecord>>,
    context: Mutex<Option<Context>>,
    current: Mutex<Option<RawContext>>,
    settings: Mutex<Option<Settings>>,
    global_record: Mutex<Option<GlobalRecord>>,
}

impl Storage {
    pub fn new(ident: impl Into<String>, config: impl Into<String>) -> Self {
        let config = config.into();
        let root_path = std::path::absolute(&config)
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        Self {
            ident: ident.into(),
            config,
            root_path,
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
fn absolute_path(storage: State<'_, Storage>, path: String) -> CommandResult<String> {
    Ok(storage.root_path.join(path).to_string_lossy().into_owned())
}

#[command]
async fn open_game(handle: AppHandle, storage: State<'_, Storage>) -> CommandResult<()> {
    let config = &storage.config;
    let context = Context::open(config, FrontendType::Html);
    pin_mut!(context);
    while let Some(status) = context.next().await {
        match status {
            OpenStatus::LoadProfile => {
                OpenGameStatus::LoadProfile(config.clone()).emit(&handle)?;
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

    let window = handle.get_window("main").unwrap();
    window.set_title(&ctx.game.config.title)?;
    let settings = {
        OpenGameStatus::LoadSettings.emit(&handle)?;
        load_settings(&storage.ident).unwrap_or_default_log("Load settings failed")
    };
    *storage.settings.lock().await = Some(settings);

    OpenGameStatus::LoadGlobalRecords.emit(&handle)?;
    let global_record = load_global_record(&storage.ident, &ctx.game.config.title)
        .unwrap_or_default_log("Load global records failed");
    *storage.global_record.lock().await = Some(global_record);

    OpenGameStatus::LoadRecords.emit(&handle)?;
    *storage.records.lock().await = load_records(&storage.ident, &ctx.game.config.title)
        .unwrap_or_default_log("Load records failed");
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
    save_settings(
        &storage.ident,
        storage.settings.lock().await.as_ref().unwrap(),
    )?;
    save_global_record(
        &storage.ident,
        game,
        storage.global_record.lock().await.as_ref().unwrap(),
    )?;
    save_records(&storage.ident, game, &storage.records.lock().await)?;
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
    let action = context
        .get_action(&settings.lang, raw_ctx)
        .unwrap_or_default_log("Cannot get action");
    let base_action = settings.sub_lang.as_ref().map(|sub_lang| {
        context
            .get_action(sub_lang, raw_ctx)
            .unwrap_or_default_log("Cannot get sub action")
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
    let port =
        portpicker::pick_unused_port().ok_or_else(|| anyhow!("failed to find unused port"))?;
    info!("Picked port {}", port);
    let mut context = tauri::generate_context!();
    let window_url = WindowUrl::External(format!("http://127.0.0.1:{port}").parse().unwrap());
    let dev_url = context.config().build.dev_path.to_string();
    context.config_mut().build.dist_dir = AppUrl::Url(window_url.clone());
    context.config_mut().build.dev_path = AppUrl::Url(window_url);
    tauri::Builder::default()
        .setup(move |app| {
            let asset_resolver = app.asset_resolver();
            std::thread::spawn(move || {
                let server = Server::http(format!("127.0.0.1:{port}"))
                    .expect("Unable to start local server");
                for req in server.incoming_requests() {
                    #[cfg(debug_assertions)]
                    let _ = asset_resolver;
                    #[cfg(not(debug_assertions))]
                    if req.url().starts_with("/assets/")
                        || req.url() == "/"
                        || req.url() == "/live2d.min.js"
                        || req.url() == "/live2dcubismcore.min.js"
                    {
                        let asset = asset_resolver.get(req.url().into()).unwrap();
                        let mut resp = if let Some(csp) = asset.csp_header {
                            #[cfg(target_os = "linux")]
                            let mut resp = {
                                let html = String::from_utf8_lossy(&asset.bytes);
                                let body = html.replacen(tauri::utils::html::CSP_TOKEN, &csp, 1);
                                tiny_http::Response::from_data(body)
                            };
                            #[cfg(not(target_os = "linux"))]
                            let mut resp = Response::from_data(asset.bytes);
                            resp.add_header(
                                Header::from_bytes("Content-Security-Policy", csp).expect(
                                    "Unable to convert csp_header to Content-Security-Policy",
                                ),
                            );
                            resp
                        } else {
                            tiny_http::Response::from_data(asset.bytes)
                        };
                        resp.add_header(
                            Header::from_bytes("Content-Type", asset.mime_type)
                                .expect("Unable to convert mime_type to Content-Type"),
                        );
                        req.respond(resp).expect("Unable to setup response");
                        continue;
                    }
                    match (req.url(), std::fs::canonicalize(req.url())) {
                        ("/", _) | (_, Err(_)) => {
                            #[cfg(debug_assertions)]
                            {
                                let path = if req.url().ends_with('/') {
                                    req.url().to_string() + "index.html"
                                } else {
                                    req.url().to_string()
                                };
                                let resp =
                                    minreq::get(dev_url.trim_end_matches('/').to_string() + &path)
                                        .send()
                                        .expect("Unable to send request");
                                req.respond(tiny_http::Response::new(
                                    resp.status_code.into(),
                                    resp.headers
                                        .iter()
                                        .map(|(k, v)| {
                                            Header::from_bytes(k.as_bytes(), v.as_bytes())
                                                .expect("Unable to convert Header")
                                        })
                                        .collect(),
                                    resp.as_bytes(),
                                    None,
                                    None,
                                ))
                                .expect("Unable to setup response")
                            }
                            #[cfg(not(debug_assertions))]
                            req.respond(tiny_http::Response::empty(404))
                                .expect("Unable to setup response")
                        }
                        (_, Ok(path)) => {
                            let _ = dev_url;
                            let file = std::fs::File::open(&path).unwrap();
                            let file = if file.metadata().unwrap().is_dir() {
                                let mut path = path.clone();
                                path.push("index.html");
                                std::fs::File::open(path).unwrap()
                            } else {
                                file
                            };
                            let mut resp = tiny_http::Response::from_file(file);
                            if let Some(mime) = mime_guess::from_path(req.url()).first() {
                                resp.add_header(
                                    Header::from_bytes("Content-Type", mime.essence_str())
                                        .expect("Unable to convert mime_type to Content-Type"),
                                );
                            }
                            req.respond(resp).expect("Unable to setup response")
                        }
                    };
                }
            });
            let ident = app.config().tauri.bundle.identifier.clone();
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
                            .directory(app.path_resolver().app_log_dir().unwrap())
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
            absolute_path,
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
        .run(context)?;
    Ok(())
}
