// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;

struct Config {
    away_prefix: char,
    home_prefix: char,
}

#[derive(Serialize)]
enum ParsingErrorCode {
    WhoScored,
    InvalidInput,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ParsingError {
    pub key: ParsingErrorCode,
    is_parsing_error: bool,
}

impl ParsingError {
    pub fn new(key: ParsingErrorCode) -> ParsingError {
        ParsingError {
            key,
            is_parsing_error: true,
        }
    }

}

#[derive(Serialize)]
enum Team {
    Away,
    Home,
}

impl Team {
    pub fn from_prefix(config: Config, prefix: char) -> Option<Team> {
        if prefix == config.away_prefix {
            Some(Team::Away)
        } else if prefix == config.home_prefix {
            Some(Team::Home)
        } else {
            None
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ParsedRally {
    team_scored: Team,
}

#[tauri::command]
fn parse_rally(rally: &str) -> Result<ParsedRally, ParsingError> {
    let config = Config {
        away_prefix: '@',
        home_prefix: '!',
    };
    let actions = rally.split(' ').collect::<Vec<&str>>();
    actions
        .into_iter()
        .last()
        .and_then(|action| action.chars().next())
        .and_then(|prefix| Team::from_prefix(config, prefix))
        .map(|who_scored| ParsedRally { team_scored: who_scored})
        .ok_or(ParsingError::new(ParsingErrorCode::WhoScored))
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![parse_rally])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
