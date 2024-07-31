use serde::Serialize;

type Location = usize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Reason {
    pub error_msg: &'static str,
    pub location: Location,
}

impl Reason {
    pub fn with_location(&self, location: Location) -> Self {
        Self {
            location,
            ..*self
        }

    }

    pub fn who_scored() -> Self {
        Reason {
            error_msg: "It's ambiguous which team scored, either fix your last action or place the team prefix after the last action.",
            location: 0
        }
    }

    pub fn team_prefix() -> Self {
        Reason {
            error_msg: "Expected team prefix here.",
            location: 0
        }
    }

    pub fn invalid_input() -> Self {
        Reason {
            error_msg: "There's a mistake somewhere in your input",
            location: 0
        }
    }

    pub fn player() -> Self {
        Reason {
            error_msg: "Expected the players number here.",
            location: 0
        }
    }

    pub fn first_action_not_serve() -> Reason {
        Reason {
            error_msg: "The first action must be a serve.",
            location: 0
        }
    }

    pub fn serve_not_first_action() -> Reason {
        Reason {
            error_msg: "A serve can only be used for the first action",
            location: 0
        }
    }


    pub fn no_actions() -> Reason {
        Reason {
            error_msg: "At least 1 action required.",
            location: 0
        }
    }
}
