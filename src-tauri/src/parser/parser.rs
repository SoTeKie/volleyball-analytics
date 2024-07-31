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
    pub point_to: Option<Team>,
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

    fn who_scored_point(action_type: ActionType, team: Team) -> Result<Team, Reason> {
        match action_type {
            ActionType::Serve(_, Some(Zone::OutOfBounds | Zone::Net)) => Ok(team.get_opponent()),
            ActionType::Serve(_, Some(_)) => Ok(team),
            ActionType::Serve(_, None) => Err(Reason::who_scored()),

            ActionType::Receive(_, Some(Zone::Overpass)) => Ok(team),
            ActionType::Receive(_, _) => Ok(team.get_opponent()),

            ActionType::Pass(_, Some(Zone::Overpass)) => Ok(team),
            ActionType::Pass(_, _) => Ok(team.get_opponent()),

            // TODO: Add more info to sets (zone?) to be able to tell who scored instead
            // of assuming setter fault on last action (might be an over-set)
            ActionType::Set => Ok(team.get_opponent()),

            ActionType::Hit(Some(Zone::OutOfBounds | Zone::Net)) => Ok(team.get_opponent()),
            ActionType::Hit(_) => Ok(team),

            ActionType::Block(t, Some(Zone::Net | Zone::OutOfBounds)) => Ok(t),
            ActionType::Block(t, _) => Ok(t.get_opponent()),

            ActionType::Freeball(Some(Zone::OutOfBounds | Zone::Net)) => Ok(team.get_opponent()),
            ActionType::Freeball(_) => Ok(team),
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

fn parse_action(config: Config, action: &str, position: Position) -> Result<Action, Reason> {
    let mut chars = action.chars().peekable();

    let team = chars
        .next()
        .ok_or(Reason::team_prefix())
        .and_then(|c| Team::from_char(config, c))?;

    let player = Player::parse(&mut chars)?;

    let action_type = match position {
        Position::First | Position::Only => ActionType::parse_first(&mut chars)?,
        _ => ActionType::parse_inner(config, &mut chars)?
    };

    let point_to = match position {
        Position::Last | Position::Only => Some(ActionType::who_scored_point(action_type, team)?),
        _ => None
    };

    Ok(Action {
        team,
        player,
        action_type,
        point_to
    })
}

pub fn parse(config: Config, rally: &str) -> Result<Vec<Action>, Reason> {
    rally
        .split(" ")
        .enumerate()
        .with_position()
        .map(|(pos, (idx, action))| {
            parse_action(config.clone(), action, pos).map_err(|e| e.with_location(idx))
        })
        .collect()
}
