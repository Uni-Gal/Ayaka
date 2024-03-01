#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::Result;

pub fn main() -> Result<()> {
    ayaka_gui_lib::run()
}
