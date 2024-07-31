use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::parser::error::Reason;
use crate::parser::parser::{Action, ActionType};

#[derive(Serialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Team {
    Away,
    Home,
}

#[derive(Clone, Serialize, Deserialize, Copy, Hash, PartialEq, Eq)]
pub struct Player(pub u8);

#[derive(Clone, Serialize, Deserialize, Copy)]
#[serde(rename_all = "camelCase")]
pub struct PlayerScores {
    pub scored: i16,
    pub faults: i16,
    pub all: i16,
}

impl PlayerScores {
    pub fn new() -> Self {
        Self {
            scored: 0,
            faults: 0,
            all: 0,
        }
    }

    pub fn merge(self, other: Self) -> Self {
        Self {
            scored: self.scored + other.scored,
            faults: self.faults + other.faults,
            all: self.all + other.all,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Copy)]
#[serde(rename_all = "camelCase")]
pub struct PlayerStats {
    pub player: Player,
    pub hits: PlayerScores,
    pub blocks: PlayerScores,
    pub serves: PlayerScores,
}

impl PlayerStats {
    pub fn new(player: Player) -> Self {
        Self {
            player,
            hits: PlayerScores::new(),
            blocks: PlayerScores::new(),
            serves: PlayerScores::new(),
        }
    }

    fn scored(self) -> i16 {
        self.hits.scored + self.blocks.scored + self.serves.scored
    }

    fn faulted(self) -> i16 {
        self.hits.faults + self.blocks.faults + self.serves.faults
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct StatsByPlayer(pub HashMap<Player, PlayerStats>);

impl StatsByPlayer {
    fn merge(&self, other: Self) -> StatsByPlayer {
        let new_map: HashMap<Player, PlayerStats> = self
            .0
            .clone()
            .into_iter()
            .map(|(player, stats)| {
                let player_update = other.0.get(&player);

                let new_stats = match player_update {
                    None => stats,
                    Some(player_update) => PlayerStats {
                        player,
                        hits: stats.hits.merge(player_update.hits),
                        blocks: stats.blocks.merge(player_update.blocks),
                        serves: stats.serves.merge(player_update.serves),
                    },
                };

                (player, new_stats)
            })
            .collect();

        StatsByPlayer(new_map)
    }
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TeamStats {
    sets: u8,
    points: u8,
    player_stats: StatsByPlayer,
}

#[derive(Serialize, Clone)]
pub struct UpdateMatchState {
    pub point_to: Team,
    pub away_player_stats: StatsByPlayer,
    pub home_player_stats: StatsByPlayer,
}

impl UpdateMatchState {
    // TODO: Move this logic/type to remove cyclical dependency
    pub fn new(actions: Vec<Action>) -> UpdateMatchState {
        let mut stats: HashMap<Team, HashMap<Player, PlayerStats>> = HashMap::new();

        actions.clone().into_iter().for_each(|action| {
            let mut default_stats = PlayerStats::new(action.player);

            let player_stats = stats
                .get_mut(&action.team)
                .and_then(|m| m.get_mut(&action.player))
                .unwrap_or(&mut default_stats);

            match action.action_type {
                ActionType::Serve(_, _) => player_stats.serves.all += 1,
                ActionType::Hit(_) => player_stats.hits.all += 1,
                ActionType::Block(_, _) => player_stats.blocks.all += 1,
                _ => (),
            }
        });

        UpdateMatchState {
            point_to: actions.last().and_then(|a| a.point_to).expect("Missing actions or point_to!"),
            away_player_stats: stats
                .get(&Team::Away)
                .map(|stats| StatsByPlayer(stats.clone()))
                .unwrap_or(StatsByPlayer(HashMap::new())),
            home_player_stats: stats
                .get(&Team::Home)
                .map(|stats| StatsByPlayer(stats.clone()))
                .unwrap_or(StatsByPlayer(HashMap::new())),
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
enum MatchStatus {
    InProgress,
    Finished,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MatchState {
    away_team: TeamStats,
    home_team: TeamStats,
    status: MatchStatus,
}

impl MatchState {
    fn get_set_winner(&self) -> Option<Team> {
        let (winning_team, losing_team, team) = if self.away_team.points > self.home_team.points {
            (&self.away_team, &self.home_team, Team::Away)
        } else {
            (&self.home_team, &self.away_team, Team::Home)
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

    pub fn update(self, update: UpdateMatchState) -> MatchState {
        let mut new_state = self.clone();

        match update.point_to {
            Team::Away => new_state.away_team.points += 1,
            Team::Home => new_state.home_team.points += 1,
        };

        new_state
            .get_set_winner()
            .into_iter()
            .for_each(|t| match t {
                Team::Away => {
                    new_state.away_team.points = 0;
                    new_state.home_team.points = 0;
                    new_state.away_team.sets += 1;
                }
                Team::Home => {
                    new_state.home_team.points = 0;
                    new_state.away_team.points = 0;
                    new_state.home_team.sets += 1;
                }
            });

        if new_state.home_team.sets == 3 || new_state.away_team.sets == 3 {
            new_state.status = MatchStatus::Finished;
        }

        new_state.away_team.player_stats = new_state
            .away_team
            .player_stats
            .merge(update.away_player_stats);
        new_state.home_team.player_stats = new_state
            .home_team
            .player_stats
            .merge(update.home_player_stats);

        new_state
    }
}
