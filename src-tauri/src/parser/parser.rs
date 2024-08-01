use std::iter::Peekable;
use std::str::Chars;

use itertools::Itertools;
use itertools::Position;

use crate::match_state::{Player, Team};
use crate::parser::error::Reason;
use crate::utils::{Discardable, Tappable};
use crate::Config;

impl Player {
    fn parse(chars: &mut Peekable<Chars>) -> Result<Player, Reason> {
        let first_digit = chars
            .next()
            .and_then(|c| c.to_digit(10))
            .ok_or(Reason::player())?;

        let second_digit = chars
            .peek()
            .ok_or(Reason::invalid_input())
            .map(|c| c.to_digit(10))?
            .utap_some(|| chars.next().unit());

        let number = match second_digit {
            None => first_digit,
            Some(second_digit) => first_digit * 10 + second_digit,
        };

        Ok(Player(
            number
                .try_into()
                .expect("This number should never be larger than an u8"),
        ))
    }
}

impl Team {
    fn from_char(config: Config, c: char) -> Result<Self, Reason> {
        if c == config.away_prefix {
            Ok(Team::Away)
        } else if c == config.home_prefix {
            Ok(Team::Home)
        } else {
            Err(Reason::team_prefix())
        }
    }

    fn get_opponent(self) -> Team {
        match self {
            Self::Away => Self::Home,
            Self::Home => Self::Away,
        }
    }
}

#[derive(Clone, Copy)]
pub enum ServePosition {
    A,
    B,
    C,
    D,
    E,
    F,
}

impl ServePosition {
    fn from_char(c: char) -> Result<Self, Reason> {
        match c {
            'A' => Ok(Self::A),
            'B' => Ok(Self::B),
            'C' => Ok(Self::C),
            'D' => Ok(Self::D),
            'E' => Ok(Self::E),
            'F' => Ok(Self::F),
            _ => Err(Reason::invalid_input()),
        }
    }
}

#[derive(Clone, Copy)]
pub enum SubZone {
    A,
    B,
    C,
    D,
}

impl SubZone {
    fn from_char(c: char) -> Result<Self, Reason> {
        match c {
            'A' => Ok(Self::A),
            'B' => Ok(Self::B),
            'C' => Ok(Self::C),
            'D' => Ok(Self::D),
            _ => Err(Reason::invalid_input()),
        }
    }
}

#[derive(Clone, Copy)]
pub enum Zone {
    One(Option<SubZone>),
    Two(Option<SubZone>),
    Three(Option<SubZone>),
    Four(Option<SubZone>),
    Five(Option<SubZone>),
    Six(Option<SubZone>),
    Seven(Option<SubZone>),
    Eight(Option<SubZone>),
    Nine(Option<SubZone>),
    Overpass,
    OutOfBounds,
    Net,
}

impl Zone {
    fn in_court(self) -> bool {
        match self {
            Zone::Overpass => false,
            Zone::OutOfBounds => false,
            Zone::Net => false,
            _ => true,
        }
    }

    fn from_chars(zone: char, sub_zone: Option<char>) -> Result<Zone, Reason> {
        let sub_zone = sub_zone.map(|s| SubZone::from_char(s)).transpose()?;

        match (zone, sub_zone) {
            ('1', sz) => Ok(Self::One(sz)),
            ('2', sz) => Ok(Self::Two(sz)),
            ('3', sz) => Ok(Self::Three(sz)),
            ('4', sz) => Ok(Self::Four(sz)),
            ('5', sz) => Ok(Self::Five(sz)),
            ('6', sz) => Ok(Self::Six(sz)),
            ('7', sz) => Ok(Self::Seven(sz)),
            ('8', sz) => Ok(Self::Eight(sz)),
            ('9', sz) => Ok(Self::Nine(sz)),
            ('0', None) => Ok(Self::OutOfBounds),
            ('N', None) => Ok(Self::Net),
            ('V', None) => Ok(Self::Overpass),
            _ => Err(Reason::invalid_input()),
        }
    }
}

#[derive(Clone, Copy)]
enum Height {
    Low,
    Mid,
    High,
}

impl Height {
    fn from_char(c: char) -> Result<Self, Reason> {
        match c {
            'L' => Ok(Self::Low),
            'M' => Ok(Self::Mid),
            'H' => Ok(Self::High),
            _ => Err(Reason::invalid_input()),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Action {
    pub team: Team,
    pub player: Player,
    pub action_type: ActionType,
}

#[derive(Clone, Copy)]
pub enum ActionType {
    Serve(Option<ServePosition>, Option<Zone>),
    Receive(Option<Height>, Option<Zone>),
    Pass(Option<Height>, Option<Zone>),
    Set,
    Hit(Option<Zone>),
    Block(Team, Option<Zone>),
    Freeball(Option<Zone>),
}

#[derive(Clone, Copy)]
pub struct Scored {
    player: Player,
    action_type: ActionType,
}

#[derive(Clone, Copy)]
pub struct WhoScored {
    pub scored: Option<Scored>,
    pub faulted: Option<Scored>,
    point_to: Team,
}

impl WhoScored {
    fn new_fault(player: Player, action_type: ActionType, point_to: Team) -> Self {
        Self {
            scored: None,
            faulted: Some(Scored {
                player,
                action_type,
            }),
            point_to,
        }
    }

    fn new_scored(player: Player, action_type: ActionType, point_to: Team) -> Self {
        Self {
            scored: Some(Scored {
                player,
                action_type,
            }),
            faulted: None,
            point_to,
        }
    }

    fn new(
        scored_player: Player,
        scored_action_type: ActionType,
        faulted_player: Player,
        faulted_action_type: ActionType,
        point_to: Team,
    ) -> Self {
        Self {
            scored: Some(Scored {
                player: scored_player,
                action_type: scored_action_type,
            }),
            faulted: Some(Scored {
                player: faulted_player,
                action_type: faulted_action_type,
            }),
            point_to,
        }
    }
}

#[derive(Clone)]
pub struct Rally {
    pub actions: Vec<Action>,
    pub who: WhoScored,
}

impl ActionType {
    fn parse_first(chars: &mut Peekable<Chars>) -> Result<ActionType, Reason> {
        chars
            .next()
            .ok_or(Reason::invalid_input())
            .and_then(|c| match c {
                'S' => {
                    let serve_position = chars
                        .peek()
                        .and_then(|c| ServePosition::from_char(*c).ok())
                        .utap_some(|| chars.next().unit());

                    let zone = chars
                        .next()
                        .map(|zone| {
                            let sub_zone = chars.next();
                            Zone::from_chars(zone, sub_zone)
                        })
                        .transpose()?;

                    Ok(ActionType::Serve(serve_position, zone))
                }
                _ => Err(Reason::first_action_not_serve()),
            })
    }

    fn who_scored_point(action: Action, related_action: Option<Action>) -> WhoScored {
        let player_faulted = WhoScored::new_fault(
            action.player,
            action.action_type,
            action.team.get_opponent(),
        );

        let player_scored = WhoScored::new_scored(
            action.player,
            action.action_type,
            action.team.get_opponent(),
        );

        let related_faulted = |related_action: Action| {
            WhoScored::new(
                action.player,
                action.action_type,
                related_action.player,
                related_action.action_type,
                action.team,
            )
        };

        let related_scored = |related_action: Action| {
            WhoScored::new(
                related_action.player,
                related_action.action_type,
                action.player,
                action.action_type,
                related_action.team,
            )
        };

        match action.action_type {
            ActionType::Serve(_, Some(Zone::OutOfBounds | Zone::Net)) => player_faulted,
            ActionType::Serve(_, _) => player_scored,

            ActionType::Receive(_, Some(Zone::Overpass)) => player_scored,
            ActionType::Receive(_, _) => match related_action {
                None => player_faulted,
                Some(related) => related_scored(related)
            },

            ActionType::Pass(_, Some(Zone::Overpass)) => player_scored,
            ActionType::Pass(_, _) => match related_action {
                None => player_faulted,
                Some(related) => related_scored(related)
            },

            // TODO: Add more info to sets (zone?) to be able to tell who scored instead
            // of assuming setter fault on last action (might be an over-set)
            ActionType::Set => player_faulted,

            ActionType::Hit(Some(Zone::OutOfBounds | Zone::Net)) => player_faulted,
            ActionType::Hit(_) => player_scored,
            ActionType::Block(t, zone)
                if t != action.team && zone.map_or(true, |z| z.in_court()) =>
            {
                match related_action {
                    Some(related) => related_faulted(related),
                    None => player_scored,
                }
            }
            ActionType::Block(_, _) => match related_action {
                Some(related) => related_scored(related),
                None => player_faulted,
            },

            ActionType::Freeball(Some(Zone::OutOfBounds | Zone::Net)) => player_faulted,
            ActionType::Freeball(_) => player_scored,
        }
    }

    fn parse_inner(config: Config, chars: &mut Peekable<Chars>) -> Result<ActionType, Reason> {
        chars
            .next()
            .ok_or(Reason::invalid_input())
            .and_then(|c| match c {
                'R' => {
                    let height = chars
                        .peek()
                        .and_then(|c| Height::from_char(*c).ok())
                        .utap_some(|| chars.next().unit());

                    let zone = chars
                        .next()
                        .map(|zone| {
                            let sub_zone = chars.next();
                            Zone::from_chars(zone, sub_zone)
                        })
                        .transpose()?;

                    Ok(ActionType::Receive(height, zone))
                }
                'P' => {
                    let height = chars
                        .peek()
                        .and_then(|c| Height::from_char(*c).ok())
                        .utap_some(|| chars.next().unit());

                    let zone = chars
                        .next()
                        .map(|zone| {
                            let sub_zone = chars.next();
                            Zone::from_chars(zone, sub_zone)
                        })
                        .transpose()?;

                    Ok(ActionType::Pass(height, zone))
                }
                'E' => Ok(ActionType::Set),
                'H' => {
                    let zone = chars
                        .next()
                        .map(|zone| {
                            let sub_zone = chars.next();
                            Zone::from_chars(zone, sub_zone)
                        })
                        .transpose()?;

                    Ok(ActionType::Hit(zone))
                }
                'B' => {
                    let team = chars
                        .next()
                        .ok_or(Reason::team_prefix())
                        .and_then(|team| Team::from_char(config, team))?;

                    let zone = chars
                        .next()
                        .map(|zone| {
                            let sub_zone = chars.next();
                            Zone::from_chars(zone, sub_zone)
                        })
                        .transpose()?;

                    Ok(ActionType::Block(team, zone))
                }
                'F' => {
                    let zone = chars
                        .next()
                        .map(|zone| {
                            let sub_zone = chars.next();
                            Zone::from_chars(zone, sub_zone)
                        })
                        .transpose()?;

                    Ok(ActionType::Freeball(zone))
                }
                'S' => Err(Reason::serve_not_first_action()),
                _ => Err(Reason::invalid_input()),
            })
    }
}

fn parse_action(config: Config, action: &str, is_first: bool) -> Result<Action, Reason> {
    let mut chars = action.chars().peekable();

    let team = chars
        .next()
        .ok_or(Reason::team_prefix())
        .and_then(|c| Team::from_char(config, c))?;

    let player = Player::parse(&mut chars)?;

    let action_type = match is_first {
        true => ActionType::parse_first(&mut chars)?,
        false => ActionType::parse_inner(config, &mut chars)?,
    };

    Ok(Action {
        team,
        player,
        action_type,
    })
}

pub fn parse(config: Config, rally: &str) -> Result<Rally, Reason> {
    let actions: Result<Vec<Action>, Reason> = rally
        .split(" ")
        .enumerate()
        .map(|(idx, action)| {
            parse_action(config.clone(), action, idx == 0).map_err(|e| e.with_location(idx))
        })
        .collect();

    actions.and_then(|a| {
        let mut reversed = a.clone().into_iter().rev();
        let (last_action, related_action) = (reversed.next().ok_or(Reason::invalid_input())?, reversed.next());
        let who_scored = ActionType::who_scored_point(last_action, related_action);

        Ok(Rally { actions: a, who: who_scored })
    })
}
