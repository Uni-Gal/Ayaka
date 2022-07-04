#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#![feature(absolute_path)]
#![feature(iterator_try_collect)]

use gal_runtime::{
    anyhow::{self, anyhow, Result},
    log::{self, info},
    Action, ActionData, Context, Game, Locale, RawValue,
};
use serde::Serialize;
use serde_json::json;
use std::{fmt::Display, sync::Arc};
use tauri::{async_runtime::Mutex, command, Manager, State};

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
fn choose_locale(locales: Vec<Locale>) -> CommandResult<Option<Locale>> {
    let current = Locale::current();
    info!("Choose {} from {:?}", current, locales);
    Ok(current.choose_from(locales.iter())?)
}

#[command]
fn locale_native_name(loc: Locale) -> CommandResult<String> {
    let name = loc.native_name()?;
    info!("{} is {}", loc, name);
    Ok(name)
}

#[derive(Default)]
struct Storage {
    context: Mutex<Option<Context>>,
    action: Mutex<Option<Action>>,
}

#[command]
fn info(game: State<Arc<Game>>) -> serde_json::Value {
    json!({
        "title": game.title(),
        "author": game.author(),
    })
}

#[command]
async fn start_new(
    locale: Locale,
    game: State<'_, Arc<Game>>,
    storage: State<'_, Storage>,
) -> CommandResult<()> {
    *(storage.context.lock().await) = Some(Context::new((*game).clone(), locale.clone())?);
    info!("Created new context with locale {}.", locale);
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

fn main() -> Result<()> {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()?;
    let port =
        portpicker::pick_unused_port().ok_or_else(|| anyhow!("failed to find unused port"))?;
    info!("Picked port {}", port);
    tauri::Builder::default()
        .plugin(tauri_plugin_localhost::Builder::new(port).build())
        .setup(|app| {
            #[cfg(debug_assertions)]
            app.get_window("main").unwrap().open_devtools();
            let matches = app.get_cli_matches()?;
            let config = matches.args["config"].value.as_str().unwrap_or("");
            info!("Loading config...");
            let game = Arc::new(Game::open(config)?);
            info!("Loaded config \"{}\"", config.escape_default());
            app.manage(game);
            app.manage(Storage::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
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
