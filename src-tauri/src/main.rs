// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use parser::Reason;
use serde::Serialize;

mod parser;
mod match_state;

#[derive(Clone, Copy)]
struct Config {
    away_prefix: char,
    home_prefix: char,
}

#[derive(Serialize)]
enum ParseRallyResult {
    Ok(match_state::MatchState),
    Fail(parser::Reason),
}

#[tauri::command]
fn parse_rally(rally: &str, current_stats: match_state::MatchState) -> ParseRallyResult {
    let config = Config {
        away_prefix: '@',
        home_prefix: '!',
    };

    match parser::parse(config, rally) {
        Ok(update) => ParseRallyResult::Ok(current_stats.update(update)),
        Err(chumsky_msg) => {
            println!("Real error: {:?}", chumsky_msg);
            ParseRallyResult::Fail(Reason::invalid_input())
        },
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![parse_rally])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
