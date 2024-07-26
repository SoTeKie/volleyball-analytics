use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone, Copy)]
pub enum Team {
    Away,
    Home,
}

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
struct TeamStats {
    sets: u8,
    points: u8,
}

#[derive(Serialize, Clone, Copy)]
pub struct UpdateMatchState {
    pub point_to: Team
}

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct MatchState {
    away_team: TeamStats,
    home_team: TeamStats,
}

impl MatchState {
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

    pub fn update(self, update: UpdateMatchState) -> MatchState {
        let mut new_state = self.clone();

        match update.point_to {
            Team::Away => new_state.away_team.points += 1,
            Team::Home => new_state.home_team.points += 1,
        };

        new_state.get_set_winner().into_iter().for_each(|t| match t {
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

        new_state
    }
}
