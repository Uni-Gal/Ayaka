#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#![feature(absolute_path)]
#![feature(iterator_try_collect)]

use gal_runtime::{
    anyhow::{self, anyhow, Result},
    log::{self, info},
    Command, Context, Game, Line, Locale, RawValue,
};
use gal_script::Program;
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
fn choose_locale(locales: Vec<String>) -> CommandResult<Option<String>> {
    let current = Locale::current();
    let locales = locales
        .into_iter()
        .map(|s| s.parse::<Locale>())
        .try_collect::<Vec<_>>()?;
    info!("Choose {} from {:?}", current, locales);
    Ok(current.choose_from(locales.iter())?.map(|loc| {
        info!("Chose locale {}", loc);
        loc.to_string()
    }))
}

#[derive(Default)]
struct Storage {
    context: Mutex<Option<Context>>,
    action: Mutex<Option<Action>>,
    switch_actions: Mutex<Vec<Program>>,
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

#[derive(Debug, Clone, Serialize)]
struct Action {
    pub line: String,
    pub character: Option<String>,
    pub switches: Vec<DisplaySwitch>,
    pub bgm: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct DisplaySwitch {
    pub text: String,
    pub enabled: bool,
}

#[command]
async fn next_run(game: State<'_, Arc<Game>>, storage: State<'_, Storage>) -> CommandResult<bool> {
    let mut context = storage.context.lock().await;
    let action = context
        .as_mut()
        .and_then(|context| context.next_run().map(|text| (context, text)))
        .map(|(context, text)| {
            let mut lines = String::new();
            let mut chname = None;
            let mut switches = vec![];
            let mut switch_actions = vec![];
            let mut bgm = None;
            for line in text.0.into_iter() {
                match line {
                    Line::Str(s) => lines.push_str(&s),
                    Line::Cmd(cmd) => match cmd {
                        Command::Par => lines.push('\n'),
                        Command::Character(_, name) => chname = Some(name),
                        Command::Exec(p) => lines.push_str(&context.call(&p).get_str()),
                        Command::Switch {
                            text,
                            action,
                            enabled,
                        } => {
                            // unwrap: when enabled is None, it means true.
                            let enabled =
                                enabled.map(|p| context.call(&p).get_bool()).unwrap_or(true);
                            switches.push(DisplaySwitch { text, enabled });
                            switch_actions.push(action);
                        }
                        Command::Bgm(index) => bgm = Some(index),
                    },
                }
            }
            let bgm = bgm
                .map(|index| game.bgm_dir().join(format!("{}.mp3", index)))
                .map(|path| {
                    std::path::absolute(path)
                        .unwrap()
                        .to_string_lossy()
                        .into_owned()
                });
            (
                Action {
                    line: lines,
                    character: chname,
                    switches,
                    bgm,
                },
                switch_actions,
            )
        });
    if let Some((action, sactions)) = action {
        info!("Next action: {:?}", action);
        *storage.action.lock().await = Some(action);
        *storage.switch_actions.lock().await = sactions;
        Ok(true)
    } else {
        info!("No action left.");
        *storage.action.lock().await = None;
        *storage.switch_actions.lock().await = vec![];
        Ok(false)
    }
}

#[command]
async fn current_run(storage: State<'_, Storage>) -> CommandResult<Option<Action>> {
    Ok(storage.action.lock().await.clone())
}

#[command]
async fn switch(i: usize, storage: State<'_, Storage>) -> CommandResult<RawValue> {
    info!("Switch {}", i);
    let mut context = storage.context.lock().await;
    let context = context
        .as_mut()
        .ok_or_else(|| anyhow!("Context not initialized."))?;
    let actions = storage.switch_actions.lock().await;
    let action = actions
        .get(i)
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
            info,
            start_new,
            next_run,
            current_run,
            switch
        ])
        .run(tauri::generate_context!())?;
    Ok(())
}
