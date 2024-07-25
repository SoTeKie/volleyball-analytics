use std::string::ParseError;

use serde::{Deserialize, Serialize};

use crate::Config;

#[derive(Serialize, Clone, Copy)]
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
pub enum ReasonKey {
    WhoScored,
    MissingTeamPrefix,
    InvalidInput,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Reason {
    pub key: ReasonKey,
    pub error_msg: &'static str,
    // TODO: Add location of error
}

impl Reason {
    pub fn who_scored() -> Reason {
        Reason {
            key: ReasonKey::WhoScored,
            error_msg: "It's ambiguous which team scored, either fix your last action or place the team prefix after the last action."
        }
    }

    pub fn missing_team_prefix() -> Reason {
        Reason {
            key: ReasonKey::MissingTeamPrefix,
            error_msg: "You're missing a team prefix in one of the actions",
        }
    }

    pub fn invalid_input() -> Reason {
        Reason {
            key: ReasonKey::InvalidInput,
            error_msg: "There's a mistake somewhere in your input",
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
struct TeamStats {
    sets: u8,
    points: u8,
}

impl TeamStats {
    fn add_point(&mut self) {
        self.points += 1;
    }
}

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    away_team: TeamStats,
    home_team: TeamStats,
}

impl Stats {
    fn get_set_winner(self) -> Option<Team> {
        let (winning_team, losing_team, team) = if self.away_team.points > self.home_team.points {
            (self.away_team, self.home_team, Team::Away)
        } else {
            (self.home_team, self.away_team, Team::Home)
        };

        let set_point_cieling = match winning_team.sets + losing_team.sets == 5 {
            true => 15,
            false => 25,
        };

        match (winning_team.points, losing_team.points) {
            (wp, lp) if wp >= set_point_cieling && wp - lp > 1 => Some(team),
            (_, _) => None,
        }
    }

    fn add_point(&mut self, point_to: Team) -> () {
        match point_to {
            Team::Away => self.away_team.add_point(),
            Team::Home => self.home_team.add_point(),
        };

        self.get_set_winner().into_iter().for_each(|t| match t {
            Team::Away => {
                self.away_team.points = 0;
                self.away_team.sets += 1;
            }
            Team::Home => {
                self.home_team.points = 0;
                self.home_team.sets += 1;
            }
        });
    }
}

#[derive(Serialize)]
pub enum ParsingResult {
    Ok(Stats),
    Fail(Reason),
}

enum ServePosition {
    A,
    B,
    C,
    D,
    E,
    F,
}

enum SubZone {
    A,
    B,
    C,
    D,
}

struct Zone {
    position: u8,
    sub_zone: SubZone,
}

struct Serve {
    team: Team,
    player: u8,
    serve_pos: Option<ServePosition>,
    zone: Option<Zone>,
}

enum Action {
    Serve(Serve),
    Receive,
    Pass,
    Set,
    Hit,
    Block,
    Freeball,
}

fn parse_action(config: Config, action: &str) -> Result<Action, Reason> {
    let team = action.chars()
        .nth(0)
        .and_then(|prefix| Team::from_prefix(config, prefix))
        .ok_or(Reason::missing_team_prefix())?;

    let first_digit = action.chars()
        .nth(1)
        .and_then(|c| c.to_digit(10))
        .ok_or(Reason::invalid_input())?;

    let player = action.chars()
        .nth(2)
        .and_then(|c| c.to_digit(10))
        .map(|second_digit| first_digit * 10 + second_digit)
        .unwrap_or(first_digit) as u8;

    let action_type_idx = if player > 9 { 3 } else { 2 };
    let action_type = action.chars().nth(action_type_idx).ok_or(Reason::invalid_input())?;

    match action_type {
        'S' => Ok(Action::Serve(Serve {
            team,
            player,
            serve_pos: None,
            zone: None,
        })),
        'R' => Ok(Action::Receive),
        'P' => Ok(Action::Pass),
        'E' => Ok(Action::Set),
        'H' => Ok(Action::Hit),
        'B' => Ok(Action::Block),
        'F' => Ok(Action::Freeball),
        _ => Err(Reason::invalid_input()),
    }
}

pub fn parse(config: Config, current_stats: &mut Stats, rally: &str) -> ParsingResult {
    let actions: Result<Vec<Action>, Reason> = rally
        .split(' ')
        .map(|a| parse_action(config, a))
        .collect();

    match actions {
        Ok(actions) => {
            match actions.last() {
                Some(Action::Serve(serve)) =>  {
                    current_stats.add_point(serve.team);
                    ParsingResult::Ok(*current_stats)
                }
                _ => ParsingResult::Fail(Reason::invalid_input())
            }
        },
        Err(reason) => ParsingResult::Fail(reason),
    }
}
