// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use match_state::UpdateMatchState;
use serde::Serialize;

mod match_state;
mod parser;
mod utils;

#[derive(Clone, Copy)]
struct Config {
    away_prefix: char,
    home_prefix: char,
}

#[derive(Serialize)]
enum ParseRallyResult {
    Ok(match_state::MatchState),
    Fail(parser::error::Reason),
}

#[tauri::command]
fn parse_rally(rally: &str, current_stats: match_state::MatchState) -> ParseRallyResult {
    let config = Config {
        away_prefix: '@',
        home_prefix: '!',
    };

    let update_state =
        parser::parser::parse(config, rally).map(|actions| UpdateMatchState::new(actions));

    match update_state {
        Ok(update) => ParseRallyResult::Ok(current_stats.update(update)),
        Err(reasons) => ParseRallyResult::Fail(reasons),
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![parse_rally])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
