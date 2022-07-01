#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use gal_runtime::{
    anyhow::{self, anyhow, Result},
    log::{self, info},
    Command, Context, Game, Line, RawValue,
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

struct Storage {
    game: Arc<Game>,
    context: Mutex<Option<Context>>,
    switch_actions: Mutex<Vec<Program>>,
}

impl Storage {
    pub fn new(game: Arc<Game>) -> Self {
        Self {
            game,
            context: Mutex::default(),
            switch_actions: Mutex::default(),
        }
    }
}

#[command]
fn info(storage: State<Storage>) -> serde_json::Value {
    let game = &storage.game;
    json!({
        "title": game.title(),
        "author": game.author(),
    })
}

#[command]
async fn start_new(storage: State<'_, Storage>) -> CommandResult<()> {
    *(storage.context.lock().await) = Some(Context::new(storage.game.clone())?);
    info!("Created new context.");
    Ok(())
}

#[derive(Debug, Serialize)]
struct Action {
    pub line: String,
    pub character: Option<String>,
    pub switches: Vec<DisplaySwitch>,
}

#[derive(Debug, Serialize)]
struct DisplaySwitch {
    pub text: String,
    pub enabled: bool,
}

#[command]
async fn next_run(storage: State<'_, Storage>) -> CommandResult<Option<Action>> {
    let mut context = storage.context.lock().await;
    let action = context
        .as_mut()
        .and_then(|context| context.next_run().map(|text| (context, text)))
        .map(|(context, text)| {
            let mut lines = String::new();
            let mut chname = None;
            let mut switches = vec![];
            let mut switch_actions = vec![];
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
                    },
                }
            }
            (
                Action {
                    line: lines,
                    character: chname,
                    switches,
                },
                switch_actions,
            )
        });
    if let Some((action, sactions)) = action {
        *storage.switch_actions.lock().await = sactions;
        Ok(Some(action))
    } else {
        Ok(None)
    }
}

#[command]
async fn switch(i: usize, storage: State<'_, Storage>) -> CommandResult<RawValue> {
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
            let matches = app.get_cli_matches()?;
            let config = matches.args["config"].value.as_str().unwrap_or("");
            info!("Loading config...");
            let game = Arc::new(Game::open(config)?);
            info!("Loaded config \"{}\"", config.escape_default());
            app.manage(Storage::new(game));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![info, start_new, next_run, switch])
        .run(tauri::generate_context!())?;
    Ok(())
}
