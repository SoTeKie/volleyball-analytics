use serde::{Deserialize, Serialize};

use crate::Config;

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
pub enum ReasonKey {
    WhoScored,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Reason {
    pub key: ReasonKey,
    pub error_msg: &'static str,
}

impl Reason {
    pub fn who_scored() -> Reason {
        Reason {
        key: ReasonKey::WhoScored,
        error_msg: "It's ambiguous which team scored, either fix your last action or place the team prefix after the last action."
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
            },
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

pub fn parse(config: Config, current_stats: &mut Stats, rally: &str) -> ParsingResult {
    let actions = rally.split(' ').collect::<Vec<&str>>();

    let who_scored = actions.into_iter()
        .last()
        .and_then(|action| action.chars().next())
        .and_then(|prefix| Team::from_prefix(config, prefix));

    match who_scored {
        None => ParsingResult::Fail(Reason::who_scored()),
        Some(team) => {
            current_stats.add_point(team);
            ParsingResult::Ok(*current_stats)
        }
    }
}
