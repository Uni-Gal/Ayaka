#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use clap::Parser;
use gal_runtime::{
    anyhow::{self, anyhow, Result},
    log::{self, info},
    Context, Game,
};
use serde::Serialize;
use serde_json::json;
use std::{ffi::OsString, fmt::Display, sync::Arc};
use tauri::{async_runtime::Mutex, command, State};

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
}

impl Storage {
    pub fn new(game: Arc<Game>) -> Self {
        Self {
            game,
            context: Mutex::default(),
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

#[derive(Debug, Parser)]
#[clap(about, version, author)]
pub struct Options {
    input: OsString,
}

fn main() -> Result<()> {
    let opts = Options::parse();
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()?;
    let game = Arc::new(Game::open(&opts.input)?);
    let port =
        portpicker::pick_unused_port().ok_or_else(|| anyhow!("failed to find unused port"))?;
    info!("Picked port {}", port);
    tauri::Builder::default()
        .plugin(tauri_plugin_localhost::Builder::new(port).build())
        .manage(Storage::new(game))
        .invoke_handler(tauri::generate_handler![info, start_new])
        .run(tauri::generate_context!())?;
    Ok(())
}
