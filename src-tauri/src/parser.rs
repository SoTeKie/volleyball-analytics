use crate::match_state::{Team, UpdateMatchState};
use crate::Config;
use serde::Serialize;

use chumsky::prelude::*;
use text::TextParser;

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

enum Height {
    Low,
    Mid,
    High,
}

type Player = u8;

struct Action {
    team: Team,
    player: Player,
    action_type: Option<ActionType>,
}

enum ActionType {
    Serve(Option<ServePosition>, Option<Zone>),
    Receive(Option<Zone>, Option<Height>),
    Pass(Option<Zone>, Option<Height>),
    Set,
    Hit(Option<Zone>),
    Block,
    Freeball(Option<Zone>),
}

fn parser(config: Config) -> impl Parser<char, Vec<Action>, Error = Simple<char>> {
    let team_prefix = just(config.away_prefix)
        .to(Team::Away)
        .or(just(config.home_prefix).to(Team::Home));

    let player = filter(|c: &char| c.is_ascii_digit())
        .repeated()
        .at_least(1)
        .at_most(2)
        .map(|cs| {
            cs.into_iter()
                .collect::<String>()
                .parse::<Player>()
                .unwrap()
        });

    let action = team_prefix
        .then(player)
        .padded()
        .map(|(team, player)| Action {
            team,
            player,
            action_type: None,
        });

    action.repeated().at_least(1).then_ignore(end())
}

pub fn parse(config: Config, rally: &str) -> Result<UpdateMatchState, Vec<Simple<char>>> {
    parser(config).parse(rally).and_then(|actions| {
        actions
            .last()
            .map(|a| UpdateMatchState { point_to: a.team })
            .ok_or(vec![Simple::custom(0..0, "???")])
    })
}
