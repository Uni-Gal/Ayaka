#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use clap::Parser;
use gal_runtime::{
    anyhow::{anyhow, Result},
    log::info,
    Game,
};
use serde::Serialize;
use serde_json::json;
use std::{error::Error, ffi::OsString, fmt::Display, sync::Arc};
use tauri::{command, State};

#[derive(Debug, Parser)]
#[clap(about, version, author)]
pub struct Options {
    input: OsString,
}

type CommandResult<T> = std::result::Result<T, CommandError>;

#[derive(Debug, Default, Serialize)]
struct CommandError {
    msg: String,
}

impl From<gal_runtime::anyhow::Error> for CommandError {
    fn from(e: gal_runtime::anyhow::Error) -> Self {
        Self { msg: e.to_string() }
    }
}

impl<T: Into<CommandError>> From<&T> for CommandError {
    fn from(e: &T) -> Self {
        e.into()
    }
}

impl Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.msg)
    }
}

impl Error for CommandError {}

struct Storage {
    game: Arc<Game>,
}

#[command]
fn info(storage: State<Storage>) -> CommandResult<serde_json::Value> {
    let game = &storage.game;
    Ok(json!({
        "title": game.title(),
        "author": game.author(),
    }))
}

fn main() -> Result<()> {
    let opts = Options::parse();
    simple_logger::SimpleLogger::new()
        .with_level(gal_runtime::log::LevelFilter::Info)
        .init()?;
    let game = Arc::new(Game::open(&opts.input)?);
    let port =
        portpicker::pick_unused_port().ok_or_else(|| anyhow!("failed to find unused port"))?;
    info!("Picker port {}", port);
    tauri::Builder::default()
        .plugin(tauri_plugin_localhost::Builder::new(port).build())
        .manage(Storage { game })
        .invoke_handler(tauri::generate_handler![info])
        .run(tauri::generate_context!())?;
    Ok(())
}
