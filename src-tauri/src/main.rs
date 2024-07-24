// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod parser;

struct Config {
    away_prefix: char,
    home_prefix: char,
}

#[tauri::command]
fn parse_rally(rally: &str, current_stats: parser::Stats) -> parser::ParsingResult {
    let config = Config {
        away_prefix: '@',
        home_prefix: '!',
    };

    parser::parse(config, &mut current_stats.clone(), rally)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![parse_rally])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
