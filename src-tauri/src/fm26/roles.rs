use std::collections::HashMap;

use serde::Serialize;
use serde_json::json;

use super::structs::POSITION_NAMES;

#[derive(Clone, Copy, Debug)]
pub(crate) struct RoleDefinition {
    pub(crate) key: &'static str,
    pub(crate) label: &'static str,
    pub(crate) short_label: &'static str,
    pub(crate) mask: u64,
    pub(crate) positions: &'static [&'static str],
    primary_attributes: &'static [&'static str],
    secondary_attributes: &'static [&'static str],
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct DutyDefinition {
    pub(crate) key: &'static str,
    pub(crate) label: &'static str,
    pub(crate) short_label: &'static str,
    pub(crate) mask: u64,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct OutOfPossessionRoleDefinition {
    pub(crate) key: &'static str,
    pub(crate) label: &'static str,
    pub(crate) short_label: &'static str,
    pub(crate) mask: u64,
    pub(crate) positions: &'static [&'static str],
    primary_attributes: &'static [&'static str],
    secondary_attributes: &'static [&'static str],
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PlayerRoleFit {
    pub(crate) role_key: String,
    pub(crate) role: String,
    pub(crate) short_role: String,
    pub(crate) role_id_mask: String,
    pub(crate) positions: &'static [&'static str],
    pub(crate) score: u8,
    pub(crate) position_fit: u8,
    pub(crate) attribute_fit: Option<u8>,
    pub(crate) evidence: Vec<String>,
    pub(crate) phase: &'static str,
    pub(crate) in_possession_role: Option<&'static str>,
    pub(crate) out_of_possession_role: Option<&'static str>,
    pub(crate) in_possession_fit: Option<u8>,
    pub(crate) out_of_possession_fit: Option<u8>,
    pub(crate) red_flags: Vec<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct PlayerRoleEvaluation {
    pub(crate) best: Option<PlayerRoleFit>,
    pub(crate) playable: Vec<PlayerRoleFit>,
    pub(crate) secondary: Vec<PlayerRoleFit>,
    pub(crate) reasoning: Vec<String>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct DecodedRoleDuty {
    pub(crate) role: Option<&'static RoleDefinition>,
    pub(crate) duty: Option<&'static DutyDefinition>,
    pub(crate) unknown_bits: u64,
}

macro_rules! ip_role {
    ($key:expr, $label:expr, $short:expr, $mask:expr, [$($pos:expr),* $(,)?], [$($primary:expr),* $(,)?], [$($secondary:expr),* $(,)?]) => {
        RoleDefinition {
            key: $key,
            label: $label,
            short_label: $short,
            mask: $mask,
            positions: &[$($pos),*],
            primary_attributes: &[$($primary),*],
            secondary_attributes: &[$($secondary),*],
        }
    };
}

macro_rules! oop_role {
    ($key:expr, $label:expr, $short:expr, $mask:expr, [$($pos:expr),* $(,)?], [$($primary:expr),* $(,)?], [$($secondary:expr),* $(,)?]) => {
        OutOfPossessionRoleDefinition {
            key: $key,
            label: $label,
            short_label: $short,
            mask: $mask,
            positions: &[$($pos),*],
            primary_attributes: &[$($primary),*],
            secondary_attributes: &[$($secondary),*],
        }
    };
}

pub(crate) const ROLE_DEFINITIONS: &[RoleDefinition] = &[
    ip_role!(
        "goalkeeper",
        "Goalkeeper",
        "GK",
        1,
        ["GK"],
        [
            "Aerial Reach",
            "Command of Area",
            "Communication",
            "Handling",
            "Reflexes",
            "Agility",
            "Concentration",
            "Positioning"
        ],
        [
            "Kicking",
            "One on Ones",
            "Throwing",
            "Anticipation",
            "Decisions"
        ]
    ),
    ip_role!(
        "ball_playing_goalkeeper",
        "Ball-Playing Goalkeeper",
        "BPGK",
        4_096,
        ["GK"],
        [
            "Aerial Reach",
            "Command of Area",
            "Communication",
            "Handling",
            "Kicking",
            "Reflexes",
            "Agility",
            "Concentration",
            "Positioning"
        ],
        [
            "Eccentricity",
            "One on Ones",
            "Throwing",
            "Anticipation",
            "Composure",
            "Decisions",
            "Passing"
        ]
    ),
    ip_role!(
        "no_nonsense_goalkeeper",
        "No-Nonsense Goalkeeper",
        "NNGK",
        9_007_199_254_740_992,
        ["GK"],
        [
            "Aerial Reach",
            "Command of Area",
            "Communication",
            "Handling",
            "Reflexes",
            "Agility",
            "Concentration",
            "Positioning"
        ],
        ["One on Ones", "Anticipation", "Decisions"]
    ),
    ip_role!(
        "centre_back",
        "Centre-Back",
        "CB",
        2,
        ["DC"],
        [
            "Heading",
            "Marking",
            "Tackling",
            "Anticipation",
            "Positioning",
            "Jumping Reach",
            "Strength"
        ],
        [
            "Aggression",
            "Bravery",
            "Composure",
            "Concentration",
            "Decisions",
            "Pace"
        ]
    ),
    ip_role!(
        "ball_playing_centre_back",
        "Ball-Playing Centre-Back",
        "BPCB",
        16_777_216,
        ["DC"],
        [
            "Heading",
            "Marking",
            "Passing",
            "Tackling",
            "Anticipation",
            "Composure",
            "Positioning",
            "Jumping Reach",
            "Strength"
        ],
        [
            "First Touch",
            "Technique",
            "Aggression",
            "Bravery",
            "Concentration",
            "Decisions",
            "Vision",
            "Pace"
        ]
    ),
    ip_role!(
        "no_nonsense_centre_back",
        "No-Nonsense Centre-Back",
        "NNCB",
        536_870_912,
        ["DC"],
        [
            "Heading",
            "Marking",
            "Tackling",
            "Anticipation",
            "Positioning",
            "Jumping Reach",
            "Strength"
        ],
        ["Aggression", "Bravery", "Concentration", "Pace"]
    ),
    ip_role!(
        "wide_centre_back",
        "Wide Centre-Back",
        "WCB",
        2_251_799_813_685_248,
        ["DC"],
        [
            "Heading",
            "Marking",
            "Tackling",
            "Anticipation",
            "Positioning",
            "Jumping Reach",
            "Strength"
        ],
        [
            "Dribbling",
            "Aggression",
            "Bravery",
            "Composure",
            "Concentration",
            "Decisions",
            "Work Rate",
            "Acceleration",
            "Agility",
            "Pace",
            "Stamina"
        ]
    ),
    ip_role!(
        "advanced_centre_back",
        "Advanced Centre-Back",
        "ACB",
        16_384,
        ["DC", "SW"],
        [
            "Heading",
            "Marking",
            "Passing",
            "Tackling",
            "Technique",
            "Anticipation",
            "Composure",
            "Decisions",
            "Positioning",
            "Teamwork",
            "Jumping Reach",
            "Strength"
        ],
        [
            "Dribbling",
            "First Touch",
            "Aggression",
            "Bravery",
            "Concentration",
            "Vision",
            "Pace",
            "Stamina"
        ]
    ),
    ip_role!(
        "overlapping_centre_back",
        "Overlapping Centre-Back",
        "OCB",
        18_014_398_509_481_984,
        ["DC"],
        [
            "Crossing",
            "Heading",
            "Marking",
            "Tackling",
            "Anticipation",
            "Work Rate",
            "Jumping Reach",
            "Pace",
            "Stamina",
            "Strength"
        ],
        [
            "Dribbling",
            "Technique",
            "Aggression",
            "Bravery",
            "Composure",
            "Concentration",
            "Decisions",
            "Off the Ball",
            "Positioning",
            "Acceleration",
            "Agility"
        ]
    ),
    ip_role!(
        "full_back",
        "Full-Back",
        "FB",
        4,
        ["DL", "DR"],
        [
            "Marking",
            "Tackling",
            "Anticipation",
            "Concentration",
            "Positioning",
            "Teamwork",
            "Acceleration"
        ],
        [
            "Crossing",
            "Dribbling",
            "Passing",
            "Technique",
            "Decisions",
            "Work Rate",
            "Agility",
            "Pace",
            "Stamina"
        ]
    ),
    ip_role!(
        "inside_full_back",
        "Inside Full-Back",
        "IFB",
        4_503_599_627_370_496,
        ["DL", "DR"],
        [
            "Heading",
            "Marking",
            "Tackling",
            "Anticipation",
            "Positioning",
            "Strength"
        ],
        [
            "Dribbling",
            "Aggression",
            "Bravery",
            "Composure",
            "Concentration",
            "Decisions",
            "Work Rate",
            "Acceleration",
            "Agility",
            "Jumping Reach",
            "Pace",
            "Stamina"
        ]
    ),
    ip_role!(
        "inside_wing_back",
        "Inside Wing-Back",
        "IWB",
        17_592_186_044_416,
        ["DL", "DR", "WBL", "WBR"],
        [
            "Passing",
            "Tackling",
            "Anticipation",
            "Composure",
            "Decisions",
            "Positioning",
            "Teamwork",
            "Acceleration"
        ],
        [
            "First Touch",
            "Marking",
            "Technique",
            "Concentration",
            "Work Rate",
            "Agility",
            "Pace",
            "Stamina"
        ]
    ),
    ip_role!(
        "playmaking_wing_back",
        "Playmaking Wing-Back",
        "PWB",
        36_028_797_018_963_968,
        ["DL", "DR", "WBL", "WBR"],
        [
            "First Touch",
            "Passing",
            "Tackling",
            "Technique",
            "Composure",
            "Decisions",
            "Positioning",
            "Teamwork",
            "Vision",
            "Acceleration"
        ],
        [
            "Crossing",
            "Dribbling",
            "Marking",
            "Anticipation",
            "Concentration",
            "Off the Ball",
            "Work Rate",
            "Agility",
            "Pace",
            "Stamina"
        ]
    ),
    ip_role!(
        "wing_back",
        "Wing-Back",
        "WB",
        8,
        ["DL", "DR", "WBL", "WBR"],
        [
            "Crossing",
            "Marking",
            "Tackling",
            "Teamwork",
            "Work Rate",
            "Acceleration",
            "Pace",
            "Stamina"
        ],
        [
            "Dribbling",
            "First Touch",
            "Passing",
            "Technique",
            "Anticipation",
            "Concentration",
            "Decisions",
            "Off the Ball",
            "Positioning",
            "Agility",
            "Balance"
        ]
    ),
    ip_role!(
        "advanced_wing_back",
        "Advanced Wing-Back",
        "AWB",
        274_877_906_944,
        ["DL", "DR", "WBL", "WBR"],
        [
            "Crossing",
            "Dribbling",
            "Technique",
            "Off the Ball",
            "Teamwork",
            "Work Rate",
            "Acceleration",
            "Agility",
            "Pace",
            "Stamina"
        ],
        [
            "First Touch",
            "Marking",
            "Passing",
            "Tackling",
            "Anticipation",
            "Decisions",
            "Flair",
            "Positioning",
            "Balance"
        ]
    ),
    ip_role!(
        "defensive_midfielder",
        "Defensive Midfielder",
        "DM",
        16,
        ["DM"],
        [
            "Tackling",
            "Anticipation",
            "Concentration",
            "Positioning",
            "Teamwork"
        ],
        [
            "First Touch",
            "Marking",
            "Passing",
            "Aggression",
            "Composure",
            "Decisions",
            "Work Rate",
            "Stamina",
            "Strength"
        ]
    ),
    ip_role!(
        "box_to_box_midfielder",
        "Box-to-Box Midfielder",
        "BBM",
        65_536,
        ["DM", "MC"],
        [
            "Passing",
            "Tackling",
            "Off the Ball",
            "Teamwork",
            "Work Rate",
            "Stamina"
        ],
        [
            "Dribbling",
            "Finishing",
            "First Touch",
            "Long Shots",
            "Technique",
            "Aggression",
            "Anticipation",
            "Composure",
            "Decisions",
            "Positioning",
            "Acceleration",
            "Balance",
            "Pace",
            "Strength"
        ]
    ),
    ip_role!(
        "box_to_box_playmaker",
        "Box-to-Box Playmaker",
        "BBP",
        70_368_744_177_664,
        ["DM", "MC"],
        [
            "First Touch",
            "Passing",
            "Technique",
            "Composure",
            "Decisions",
            "Off the Ball",
            "Teamwork",
            "Vision",
            "Work Rate",
            "Stamina"
        ],
        [
            "Dribbling",
            "Marking",
            "Tackling",
            "Anticipation",
            "Positioning",
            "Acceleration",
            "Agility",
            "Balance",
            "Pace"
        ]
    ),
    ip_role!(
        "deep_lying_playmaker",
        "Deep-Lying Playmaker",
        "DLP",
        32_768,
        ["DM", "MC"],
        [
            "First Touch",
            "Passing",
            "Technique",
            "Composure",
            "Decisions",
            "Off the Ball",
            "Teamwork",
            "Vision"
        ],
        [
            "Marking",
            "Tackling",
            "Anticipation",
            "Concentration",
            "Positioning",
            "Work Rate",
            "Balance",
            "Stamina"
        ]
    ),
    ip_role!(
        "half_back",
        "Half-Back",
        "HB",
        34_359_738_368,
        ["DM"],
        [
            "Heading",
            "Marking",
            "Tackling",
            "Anticipation",
            "Concentration",
            "Positioning",
            "Teamwork",
            "Jumping Reach",
            "Strength"
        ],
        [
            "First Touch",
            "Passing",
            "Aggression",
            "Bravery",
            "Composure",
            "Decisions",
            "Work Rate",
            "Stamina"
        ]
    ),
    ip_role!(
        "central_midfielder",
        "Central Midfielder",
        "CM",
        32,
        ["MC"],
        [
            "First Touch",
            "Passing",
            "Tackling",
            "Decisions",
            "Teamwork"
        ],
        [
            "Technique",
            "Anticipation",
            "Composure",
            "Concentration",
            "Off the Ball",
            "Positioning",
            "Vision",
            "Work Rate",
            "Stamina"
        ]
    ),
    ip_role!(
        "advanced_playmaker",
        "Advanced Playmaker",
        "AP",
        131_072,
        ["MC", "AMC"],
        [
            "First Touch",
            "Passing",
            "Technique",
            "Composure",
            "Decisions",
            "Off the Ball",
            "Teamwork",
            "Vision"
        ],
        [
            "Crossing",
            "Dribbling",
            "Anticipation",
            "Flair",
            "Acceleration",
            "Agility"
        ]
    ),
    ip_role!(
        "midfield_playmaker",
        "Midfield Playmaker",
        "MP",
        144_115_188_075_855_872,
        ["MC"],
        [
            "First Touch",
            "Passing",
            "Technique",
            "Composure",
            "Decisions",
            "Off the Ball",
            "Teamwork",
            "Vision"
        ],
        [
            "Dribbling",
            "Tackling",
            "Anticipation",
            "Flair",
            "Positioning",
            "Work Rate",
            "Agility",
            "Stamina"
        ]
    ),
    ip_role!(
        "wide_central_midfielder",
        "Wide Central Midfielder",
        "WCM",
        281_474_976_710_656,
        ["MC"],
        [
            "First Touch",
            "Passing",
            "Tackling",
            "Decisions",
            "Teamwork"
        ],
        [
            "Crossing",
            "Dribbling",
            "Technique",
            "Anticipation",
            "Composure",
            "Concentration",
            "Off the Ball",
            "Positioning",
            "Vision",
            "Work Rate",
            "Agility",
            "Stamina"
        ]
    ),
    ip_role!(
        "wide_midfielder",
        "Wide Midfielder",
        "WM",
        64,
        ["ML", "MR"],
        [
            "Crossing",
            "Passing",
            "Technique",
            "Teamwork",
            "Work Rate",
            "Pace",
            "Stamina"
        ],
        [
            "Dribbling",
            "First Touch",
            "Anticipation",
            "Composure",
            "Off the Ball",
            "Vision",
            "Acceleration",
            "Agility"
        ]
    ),
    ip_role!(
        "inside_winger",
        "Inside Winger",
        "IW",
        562_949_953_421_312,
        ["ML", "MR", "AML", "AMR"],
        [
            "Dribbling",
            "First Touch",
            "Technique",
            "Composure",
            "Teamwork",
            "Acceleration",
            "Agility"
        ],
        [
            "Crossing",
            "Long Shots",
            "Passing",
            "Anticipation",
            "Flair",
            "Off the Ball",
            "Vision",
            "Work Rate",
            "Balance",
            "Pace",
            "Stamina"
        ]
    ),
    ip_role!(
        "playmaking_winger",
        "Playmaking Winger",
        "PW",
        8_796_093_022_208,
        ["ML", "MR", "AML", "AMR"],
        [
            "Crossing",
            "Dribbling",
            "First Touch",
            "Passing",
            "Technique",
            "Composure",
            "Decisions",
            "Off the Ball",
            "Teamwork",
            "Vision",
            "Acceleration"
        ],
        [
            "Anticipation",
            "Flair",
            "Work Rate",
            "Agility",
            "Pace",
            "Stamina"
        ]
    ),
    ip_role!(
        "winger",
        "Winger",
        "W",
        128,
        ["ML", "MR", "AML", "AMR"],
        [
            "Crossing",
            "Dribbling",
            "Technique",
            "Teamwork",
            "Acceleration",
            "Agility",
            "Pace"
        ],
        [
            "First Touch",
            "Passing",
            "Anticipation",
            "Flair",
            "Off the Ball",
            "Work Rate",
            "Balance",
            "Stamina"
        ]
    ),
    ip_role!(
        "attacking_midfielder",
        "Attacking Midfielder",
        "AM",
        512,
        ["AMC"],
        [
            "First Touch",
            "Long Shots",
            "Passing",
            "Technique",
            "Composure",
            "Flair",
            "Off the Ball"
        ],
        [
            "Crossing",
            "Dribbling",
            "Finishing",
            "Anticipation",
            "Decisions",
            "Vision",
            "Acceleration",
            "Agility"
        ]
    ),
    ip_role!(
        "channel_midfielder",
        "Channel Midfielder",
        "CHM",
        72_057_594_037_927_936,
        ["AMC", "MC"],
        [
            "Crossing",
            "First Touch",
            "Passing",
            "Technique",
            "Composure",
            "Off the Ball",
            "Work Rate",
            "Acceleration"
        ],
        [
            "Dribbling",
            "Long Shots",
            "Anticipation",
            "Decisions",
            "Flair",
            "Vision",
            "Agility",
            "Pace",
            "Stamina"
        ]
    ),
    ip_role!(
        "free_role",
        "Free Role",
        "FR",
        4_294_967_296,
        ["AMC"],
        [
            "Dribbling",
            "First Touch",
            "Long Shots",
            "Passing",
            "Technique",
            "Composure",
            "Flair",
            "Off the Ball",
            "Vision"
        ],
        [
            "Crossing",
            "Finishing",
            "Anticipation",
            "Decisions",
            "Acceleration",
            "Agility"
        ]
    ),
    ip_role!(
        "second_striker",
        "Second Striker",
        "SS",
        2_199_023_255_552,
        ["AMC"],
        [
            "Finishing",
            "First Touch",
            "Anticipation",
            "Composure",
            "Off the Ball",
            "Acceleration"
        ],
        [
            "Dribbling",
            "Long Shots",
            "Passing",
            "Technique",
            "Concentration",
            "Decisions",
            "Work Rate",
            "Agility",
            "Pace",
            "Stamina"
        ]
    ),
    ip_role!(
        "wide_forward",
        "Wide Forward",
        "WF",
        35_184_372_088_832,
        ["AML", "AMR"],
        [
            "Dribbling",
            "First Touch",
            "Technique",
            "Anticipation",
            "Off the Ball",
            "Acceleration",
            "Agility",
            "Pace"
        ],
        [
            "Crossing",
            "Finishing",
            "Passing",
            "Composure",
            "Flair",
            "Work Rate",
            "Balance",
            "Stamina"
        ]
    ),
    ip_role!(
        "inside_forward",
        "Inside Forward",
        "IF",
        134_217_728,
        ["AML", "AMR"],
        [
            "Dribbling",
            "First Touch",
            "Technique",
            "Anticipation",
            "Composure",
            "Off the Ball",
            "Acceleration",
            "Agility"
        ],
        [
            "Crossing",
            "Finishing",
            "Long Shots",
            "Passing",
            "Flair",
            "Vision",
            "Work Rate",
            "Balance",
            "Pace",
            "Stamina"
        ]
    ),
    ip_role!(
        "centre_forward",
        "Centre Forward",
        "CF",
        2_048,
        ["ST"],
        [
            "Finishing",
            "First Touch",
            "Heading",
            "Technique",
            "Composure",
            "Off the Ball",
            "Acceleration",
            "Strength"
        ],
        [
            "Dribbling",
            "Passing",
            "Anticipation",
            "Decisions",
            "Agility",
            "Balance",
            "Jumping Reach",
            "Pace"
        ]
    ),
    ip_role!(
        "channel_forward",
        "Channel Forward",
        "CHF",
        2_147_483_648,
        ["ST"],
        [
            "Dribbling",
            "Finishing",
            "First Touch",
            "Technique",
            "Composure",
            "Off the Ball",
            "Work Rate",
            "Acceleration"
        ],
        [
            "Crossing",
            "Heading",
            "Passing",
            "Anticipation",
            "Decisions",
            "Agility",
            "Balance",
            "Pace",
            "Stamina"
        ]
    ),
    ip_role!(
        "deep_lying_forward",
        "Deep-Lying Forward",
        "DLF",
        1_024,
        ["ST"],
        [
            "Finishing",
            "First Touch",
            "Technique",
            "Composure",
            "Off the Ball",
            "Strength"
        ],
        [
            "Dribbling",
            "Passing",
            "Anticipation",
            "Decisions",
            "Teamwork",
            "Vision",
            "Balance"
        ]
    ),
    ip_role!(
        "false_nine",
        "False Nine",
        "F9",
        1_099_511_627_776,
        ["ST"],
        [
            "Dribbling",
            "First Touch",
            "Passing",
            "Technique",
            "Composure",
            "Decisions",
            "Off the Ball",
            "Teamwork",
            "Vision",
            "Acceleration"
        ],
        ["Finishing", "Anticipation", "Flair", "Agility", "Balance"]
    ),
    ip_role!(
        "poacher",
        "Poacher",
        "P",
        524_288,
        ["ST"],
        [
            "Finishing",
            "Heading",
            "Anticipation",
            "Composure",
            "Concentration",
            "Off the Ball",
            "Acceleration"
        ],
        ["First Touch", "Technique", "Decisions", "Balance"]
    ),
    ip_role!(
        "target_forward",
        "Target Forward",
        "TF",
        262_144,
        ["ST"],
        [
            "Finishing",
            "Heading",
            "Aggression",
            "Bravery",
            "Composure",
            "Off the Ball",
            "Balance",
            "Jumping Reach",
            "Strength"
        ],
        ["First Touch", "Anticipation", "Decisions", "Teamwork"]
    ),
];

pub(crate) const DUTY_DEFINITIONS: &[DutyDefinition] = &[];

pub(crate) const OUT_OF_POSSESSION_ROLE_DEFINITIONS: &[OutOfPossessionRoleDefinition] = &[
    oop_role!(
        "line_holding_keeper",
        "Line-Holding Keeper",
        "LHK",
        1,
        ["GK"],
        ["Positioning", "Concentration"],
        []
    ),
    oop_role!(
        "sweeper_keeper",
        "Sweeper Keeper",
        "SK",
        2,
        ["GK"],
        ["Rushing Out", "Anticipation", "Decisions"],
        []
    ),
    oop_role!(
        "covering_centre_back",
        "Covering Centre-Back",
        "CCB",
        4,
        ["DC"],
        ["Anticipation", "Pace", "Marking"],
        []
    ),
    oop_role!(
        "stopping_centre_back",
        "Stopping Centre-Back",
        "SCB",
        8,
        ["DC"],
        ["Aggression", "Tackling", "Strength"],
        []
    ),
    oop_role!(
        "covering_wide_centre_back",
        "Covering Wide Centre-Back",
        "CWCB",
        16,
        ["DC"],
        ["Anticipation", "Pace", "Marking"],
        []
    ),
    oop_role!(
        "stopping_wide_centre_back",
        "Stopping Wide Centre-Back",
        "SWCB",
        32,
        ["DC"],
        ["Aggression", "Tackling", "Strength"],
        []
    ),
    oop_role!(
        "holding_full_back",
        "Holding Full-Back",
        "HFB",
        64,
        ["DL", "DR"],
        ["Positioning", "Concentration", "Marking"],
        []
    ),
    oop_role!(
        "pressing_full_back",
        "Pressing Full-Back",
        "PFB",
        128,
        ["DL", "DR"],
        ["Aggression", "Work Rate", "Anticipation"],
        []
    ),
    oop_role!(
        "holding_wing_back",
        "Holding Wing-Back",
        "HWB",
        256,
        ["DL", "DR", "WBL", "WBR"],
        ["Positioning", "Concentration", "Marking"],
        []
    ),
    oop_role!(
        "pressing_wing_back",
        "Pressing Wing-Back",
        "PWB",
        512,
        ["DL", "DR", "WBL", "WBR"],
        ["Aggression", "Work Rate", "Anticipation"],
        []
    ),
    oop_role!(
        "dropping_defensive_midfielder",
        "Dropping Defensive Midfielder",
        "DDM",
        1_024,
        ["DM"],
        ["Positioning", "Decisions", "Anticipation"],
        []
    ),
    oop_role!(
        "pressing_defensive_midfielder",
        "Pressing Defensive Midfielder",
        "PDM",
        2_048,
        ["DM"],
        ["Aggression", "Work Rate", "Anticipation"],
        []
    ),
    oop_role!(
        "screening_defensive_midfielder",
        "Screening Defensive Midfielder",
        "SDM",
        4_096,
        ["DM"],
        ["Positioning", "Concentration", "Marking"],
        []
    ),
    oop_role!(
        "wide_covering_defensive_midfielder",
        "Wide Covering Defensive Midfielder",
        "WCDM",
        8_192,
        ["DM"],
        ["Anticipation", "Pace", "Work Rate"],
        []
    ),
    oop_role!(
        "pressing_central_midfielder",
        "Pressing Central Midfielder",
        "PCM",
        16_384,
        ["MC"],
        ["Aggression", "Work Rate", "Anticipation"],
        []
    ),
    oop_role!(
        "screening_central_midfielder",
        "Screening Central Midfielder",
        "SCM",
        32_768,
        ["MC"],
        ["Positioning", "Concentration", "Marking"],
        []
    ),
    oop_role!(
        "wide_covering_central_midfielder",
        "Wide Covering Central Midfielder",
        "WCCM",
        65_536,
        ["MC"],
        ["Anticipation", "Pace", "Work Rate"],
        []
    ),
    oop_role!(
        "tracking_wide_midfielder",
        "Tracking Wide Midfielder",
        "TWM",
        131_072,
        ["ML", "MR"],
        ["Marking", "Work Rate", "Stamina"],
        []
    ),
    oop_role!(
        "wide_outlet_wide_midfielder",
        "Wide Outlet Wide Midfielder",
        "WOWM",
        262_144,
        ["ML", "MR"],
        ["Off the Ball", "Pace", "Anticipation"],
        []
    ),
    oop_role!(
        "tracking_winger",
        "Tracking Winger",
        "TW",
        524_288,
        ["ML", "MR", "AML", "AMR"],
        ["Marking", "Work Rate", "Stamina"],
        []
    ),
    oop_role!(
        "wide_outlet_winger",
        "Wide Outlet Winger",
        "WOW",
        1_048_576,
        ["ML", "MR", "AML", "AMR"],
        ["Off the Ball", "Pace", "Anticipation"],
        []
    ),
    oop_role!(
        "inside_outlet_winger",
        "Inside Outlet Winger",
        "IOW",
        2_097_152,
        ["ML", "MR", "AML", "AMR"],
        ["Off the Ball", "Decisions", "Anticipation"],
        []
    ),
    oop_role!(
        "central_outlet_attacking_midfielder",
        "Central Outlet Attacking Midfielder",
        "COAM",
        4_194_304,
        ["AMC"],
        ["Off the Ball", "Decisions", "Anticipation"],
        []
    ),
    oop_role!(
        "splitting_outlet_attacking_midfielder",
        "Splitting Outlet Attacking Midfielder",
        "SOAM",
        8_388_608,
        ["AMC"],
        ["Off the Ball", "Pace", "Anticipation"],
        []
    ),
    oop_role!(
        "tracking_attacking_midfielder",
        "Tracking Attacking Midfielder",
        "TAM",
        16_777_216,
        ["AMC"],
        ["Marking", "Work Rate", "Stamina"],
        []
    ),
    oop_role!(
        "central_outlet_centre_forward",
        "Central Outlet Centre Forward",
        "COCF",
        33_554_432,
        ["ST"],
        ["Off the Ball", "Decisions", "Anticipation"],
        []
    ),
    oop_role!(
        "splitting_outlet_centre_forward",
        "Splitting Outlet Centre Forward",
        "SOCF",
        67_108_864,
        ["ST"],
        ["Off the Ball", "Pace", "Anticipation"],
        []
    ),
    oop_role!(
        "tracking_centre_forward",
        "Tracking Centre Forward",
        "TCF",
        134_217_728,
        ["ST"],
        ["Marking", "Work Rate", "Stamina"],
        []
    ),
];

pub(crate) fn evaluate_player_roles(
    position_bytes: &[u8],
    attributes: &HashMap<String, u8>,
) -> PlayerRoleEvaluation {
    let mut ip_fits = ROLE_DEFINITIONS
        .iter()
        .filter_map(|role| score_in_possession_role(role, position_bytes, attributes))
        .collect::<Vec<_>>();
    let mut oop_fits = OUT_OF_POSSESSION_ROLE_DEFINITIONS
        .iter()
        .filter_map(|role| score_out_of_possession_role(role, position_bytes, attributes))
        .collect::<Vec<_>>();
    sort_fits(&mut ip_fits);
    sort_fits(&mut oop_fits);

    let mut combined = combine_phase_roles(&ip_fits, &oop_fits, attributes);
    sort_fits(&mut combined);

    let best = combined
        .first()
        .cloned()
        .or_else(|| ip_fits.first().cloned())
        .or_else(|| oop_fits.first().cloned());
    let playable = combined
        .iter()
        .filter(|fit| fit.position_fit >= 55 && fit.score >= 45)
        .take(8)
        .cloned()
        .collect::<Vec<_>>();
    let secondary = combined
        .iter()
        .filter(|fit| fit.position_fit >= 30 && fit.score >= 35)
        .skip(playable.len())
        .take(8)
        .cloned()
        .collect::<Vec<_>>();

    let reasoning = match &best {
        Some(fit) if fit.phase == "combined" => vec![
            format!(
                "{} is the highest FM26 combined tactical fit from split IP/OOP role scoring.",
                fit.role
            ),
            format!(
                "IP fit {}%, OOP fit {}%, combined {}%.",
                fit.in_possession_fit
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "unknown".to_string()),
                fit.out_of_possession_fit
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "unknown".to_string()),
                fit.score
            ),
            "FM26 has no old Defend/Support/Attack duty model here; GlassScout scores role behaviour separately by phase.".to_string(),
        ],
        Some(fit) => vec![
            format!("{} is the highest FM26 {} role fit.", fit.role, fit.phase),
            "Only one phase could be scored from the mapped position/attribute evidence.".to_string(),
        ],
        None => vec![
            "No FM26 role could be scored from the mapped position and attribute data.".to_string(),
        ],
    };

    PlayerRoleEvaluation {
        best,
        playable,
        secondary,
        reasoning,
    }
}

pub(crate) fn decode_role_duty_mask(value: u64) -> DecodedRoleDuty {
    let role = ROLE_DEFINITIONS
        .iter()
        .find(|definition| value & definition.mask == definition.mask && definition.mask != 0);
    let known_role_bits = role.map(|definition| definition.mask).unwrap_or_default();
    DecodedRoleDuty {
        role,
        duty: None,
        unknown_bits: value & !known_role_bits,
    }
}

pub(crate) fn role_definition_for_mask(mask: u64) -> Option<&'static RoleDefinition> {
    ROLE_DEFINITIONS
        .iter()
        .find(|definition| definition.mask == mask)
}

pub(crate) fn duty_definition_for_mask(_mask: u64) -> Option<&'static DutyDefinition> {
    None
}

pub(crate) fn role_supports_slot(role: &RoleDefinition, slot: &str) -> bool {
    canonical_slot(slot)
        .map(|slot| role.positions.iter().any(|position| *position == slot))
        .unwrap_or(false)
}

pub(crate) fn role_catalogue_status() -> serde_json::Value {
    json!({
        "inPossessionRoleCount": ROLE_DEFINITIONS.len(),
        "dutyCount": DUTY_DEFINITIONS.len(),
        "outOfPossessionRoleCount": OUT_OF_POSSESSION_ROLE_DEFINITIONS.len(),
        "dutyModel": "fm26-phase-roles-no-defend-support-attack",
        "combinedFitFormula": "IP 60% + OOP 40%",
        "inPossessionRoles": ROLE_DEFINITIONS
            .iter()
            .map(|role| role_json(role))
            .collect::<Vec<_>>(),
        "outOfPossessionRoles": OUT_OF_POSSESSION_ROLE_DEFINITIONS
            .iter()
            .map(|role| oop_role_json(role))
            .collect::<Vec<_>>(),
        "source": "FM26 split in-possession/out-of-possession role model from project role notes",
        "status": "fm26-phase-catalogue-mapped"
    })
}

fn sort_fits(fits: &mut [PlayerRoleFit]) {
    fits.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| right.position_fit.cmp(&left.position_fit))
            .then_with(|| left.role.cmp(&right.role))
    });
}

fn score_in_possession_role(
    role: &'static RoleDefinition,
    position_bytes: &[u8],
    attributes: &HashMap<String, u8>,
) -> Option<PlayerRoleFit> {
    let (position_fit, top_position, top_familiarity) =
        role_position_fit(role.positions, position_bytes)?;
    if position_fit < 5 {
        return None;
    }
    let attribute_fit = weighted_attribute_fit(
        role.primary_attributes,
        role.secondary_attributes,
        attributes,
    );
    let score = phase_score(position_fit, attribute_fit);
    let mut evidence = role_evidence(
        role.primary_attributes,
        role.secondary_attributes,
        attributes,
        top_position,
        top_familiarity,
    );
    evidence.insert(0, "In possession role model".to_string());
    Some(PlayerRoleFit {
        role_key: role.key.to_string(),
        role: role.label.to_string(),
        short_role: role.short_label.to_string(),
        role_id_mask: format!("0x{:X}", role.mask),
        positions: role.positions,
        score,
        position_fit,
        attribute_fit,
        evidence,
        phase: "in-possession",
        in_possession_role: Some(role.label),
        out_of_possession_role: None,
        in_possession_fit: Some(score),
        out_of_possession_fit: None,
        red_flags: role_red_flags(
            role.label,
            role.primary_attributes,
            role.secondary_attributes,
            attributes,
        ),
    })
}

fn score_out_of_possession_role(
    role: &'static OutOfPossessionRoleDefinition,
    position_bytes: &[u8],
    attributes: &HashMap<String, u8>,
) -> Option<PlayerRoleFit> {
    let (position_fit, top_position, top_familiarity) =
        role_position_fit(role.positions, position_bytes)?;
    if position_fit < 5 {
        return None;
    }
    let attribute_fit = weighted_attribute_fit(
        role.primary_attributes,
        role.secondary_attributes,
        attributes,
    );
    let score = phase_score(position_fit, attribute_fit);
    let mut evidence = role_evidence(
        role.primary_attributes,
        role.secondary_attributes,
        attributes,
        top_position,
        top_familiarity,
    );
    evidence.insert(0, "Out of possession role model".to_string());
    Some(PlayerRoleFit {
        role_key: role.key.to_string(),
        role: role.label.to_string(),
        short_role: role.short_label.to_string(),
        role_id_mask: format!("OOP:0x{:X}", role.mask),
        positions: role.positions,
        score,
        position_fit,
        attribute_fit,
        evidence,
        phase: "out-of-possession",
        in_possession_role: None,
        out_of_possession_role: Some(role.label),
        in_possession_fit: None,
        out_of_possession_fit: Some(score),
        red_flags: role_red_flags(
            role.label,
            role.primary_attributes,
            role.secondary_attributes,
            attributes,
        ),
    })
}

fn combine_phase_roles(
    ip_fits: &[PlayerRoleFit],
    oop_fits: &[PlayerRoleFit],
    attributes: &HashMap<String, u8>,
) -> Vec<PlayerRoleFit> {
    let mut combined = Vec::new();
    for ip in ip_fits.iter().take(14) {
        for oop in oop_fits.iter().take(14) {
            if !phase_positions_compatible(ip.positions, oop.positions) {
                continue;
            }
            let score = ((f32::from(ip.score) * 0.60) + (f32::from(oop.score) * 0.40))
                .round()
                .clamp(0.0, 100.0) as u8;
            let position_fit =
                ((u16::from(ip.position_fit) + u16::from(oop.position_fit)) / 2) as u8;
            let attribute_fit = match (ip.attribute_fit, oop.attribute_fit) {
                (Some(left), Some(right)) => Some(((u16::from(left) + u16::from(right)) / 2) as u8),
                (Some(value), None) | (None, Some(value)) => Some(value),
                (None, None) => None,
            };
            let mut red_flags = ip.red_flags.clone();
            red_flags.extend(oop.red_flags.clone());
            red_flags.sort();
            red_flags.dedup();
            if red_flags.is_empty() {
                red_flags.extend(combined_red_flags(ip, oop, attributes));
            }
            let mut evidence = vec![
                format!("IP {} fit {}", ip.role, ip.score),
                format!("OOP {} fit {}", oop.role, oop.score),
                "Combined tactical fit uses IP 60% and OOP 40%.".to_string(),
            ];
            evidence.extend(ip.evidence.iter().skip(1).take(2).cloned());
            evidence.extend(oop.evidence.iter().skip(1).take(2).cloned());
            combined.push(PlayerRoleFit {
                role_key: format!("{}_plus_{}", ip.role_key, oop.role_key),
                role: format!("{} + {}", ip.role, oop.role),
                short_role: format!("{}+{}", ip.short_role, oop.short_role),
                role_id_mask: format!("IP:{};OOP:{}", ip.role_id_mask, oop.role_id_mask),
                positions: ip.positions,
                score,
                position_fit,
                attribute_fit,
                evidence,
                phase: "combined",
                in_possession_role: ip.in_possession_role,
                out_of_possession_role: oop.out_of_possession_role,
                in_possession_fit: ip.in_possession_fit,
                out_of_possession_fit: oop.out_of_possession_fit,
                red_flags,
            });
        }
    }
    combined
}

fn phase_score(position_fit: u8, attribute_fit: Option<u8>) -> u8 {
    match attribute_fit {
        Some(attribute_fit) => {
            (f32::from(position_fit) * 0.34 + f32::from(attribute_fit) * 0.66).round() as u8
        }
        None => position_fit,
    }
    .min(position_fit.saturating_add(25))
    .clamp(0, 100)
}

fn role_position_fit(
    positions: &'static [&'static str],
    position_bytes: &[u8],
) -> Option<(u8, &'static str, u8)> {
    let mut best: Option<(&'static str, u8)> = None;
    for position in positions {
        let index = POSITION_NAMES.iter().position(|name| name == position)?;
        let familiarity = position_bytes.get(index).copied().unwrap_or_default();
        if best.is_none_or(|(_, current)| familiarity > current) {
            best = Some((*position, familiarity));
        }
    }
    let (position, familiarity) = best?;
    let fit = ((f32::from(familiarity) / 20.0) * 100.0)
        .round()
        .clamp(0.0, 100.0) as u8;
    Some((fit, position, familiarity))
}

fn weighted_attribute_fit(
    primary_attributes: &[&str],
    secondary_attributes: &[&str],
    attributes: &HashMap<String, u8>,
) -> Option<u8> {
    let mut total = 0_u32;
    let mut weight = 0_u32;
    let mut primary_total = 0_u32;
    let mut primary_count = 0_u32;
    let mut weak_primary = 0_u32;
    for key in primary_attributes {
        if let Some(value) = attributes.get(*key) {
            total += u32::from(*value) * 3;
            weight += 3;
            primary_total += u32::from(*value);
            primary_count += 1;
            if *value <= 8 {
                weak_primary += 1;
            }
        }
    }
    for key in secondary_attributes {
        if let Some(value) = attributes.get(*key) {
            total += u32::from(*value);
            weight += 1;
        }
    }
    (weight >= 6).then(|| {
        let average = total as f32 / weight as f32;
        let mut score = ((average / 20.0) * 100.0).round().clamp(0.0, 100.0) as i16;
        if primary_count >= 3 {
            let primary_average = primary_total as f32 / primary_count as f32;
            if primary_average < 10.0 {
                score -= 15;
            }
        }
        if weak_primary >= 2 {
            score -= 10;
        }
        score.clamp(0, 100) as u8
    })
}

fn role_evidence(
    primary_attributes: &[&str],
    secondary_attributes: &[&str],
    attributes: &HashMap<String, u8>,
    top_position: &'static str,
    top_familiarity: u8,
) -> Vec<String> {
    let mut evidence = vec![format!("{top_position} familiarity {top_familiarity}/20")];
    let mut values = primary_attributes
        .iter()
        .chain(secondary_attributes.iter())
        .filter_map(|key| {
            attributes
                .get(*key)
                .map(|value| ((*key).to_string(), *value))
        })
        .collect::<Vec<_>>();
    values.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    evidence.extend(
        values
            .into_iter()
            .take(3)
            .map(|(attribute, value)| format!("{attribute} {value}")),
    );
    evidence
}

fn role_red_flags(
    label: &str,
    primary_attributes: &[&str],
    secondary_attributes: &[&str],
    attributes: &HashMap<String, u8>,
) -> Vec<String> {
    let mut flags = Vec::new();
    let weak = |attribute: &str| attributes.get(attribute).is_some_and(|value| *value <= 9);
    let any_weak_primary = primary_attributes.iter().any(|attribute| weak(attribute));
    if label.contains("Pressing")
        && ["Aggression", "Work Rate", "Anticipation"]
            .iter()
            .any(|attribute| weak(attribute))
    {
        flags.push(
            "Pressing fraud: pressing role has weak aggression, work rate or anticipation."
                .to_string(),
        );
    }
    if label.contains("Covering")
        && ["Pace", "Anticipation"]
            .iter()
            .any(|attribute| weak(attribute))
    {
        flags.push("High-line risk: covering role lacks pace or anticipation.".to_string());
    }
    if (label.contains("Playmaker")
        || label.contains("Playmaking")
        || label.contains("Free Role")
        || label.contains("False Nine"))
        && ["Passing", "Decisions", "Vision", "Technique"]
            .iter()
            .any(|attribute| weak(attribute))
    {
        flags.push(
            "Creative fraud: creative role has weak passing, decisions, vision or technique."
                .to_string(),
        );
    }
    if (label.contains("Centre-Back") || label.contains("Target"))
        && ["Strength", "Jumping Reach", "Heading"]
            .iter()
            .any(|attribute| weak(attribute))
    {
        flags.push(
            "Physical mismatch: aerial/physical role has weak strength, jumping or heading."
                .to_string(),
        );
    }
    if ["Marking", "Work Rate", "Stamina"].iter().all(|attribute| {
        primary_attributes.contains(attribute) || secondary_attributes.contains(attribute)
    }) && ["Marking", "Work Rate", "Stamina"]
        .iter()
        .any(|attribute| weak(attribute))
    {
        flags.push(
            "Bad defensive match: tracking role lacks marking, work rate or stamina.".to_string(),
        );
    }
    if any_weak_primary {
        flags.push("Weak VI attribute: one or more very important role attributes are below the safe threshold.".to_string());
    }
    flags
}

fn combined_red_flags(
    ip: &PlayerRoleFit,
    oop: &PlayerRoleFit,
    attributes: &HashMap<String, u8>,
) -> Vec<String> {
    let mut flags = Vec::new();
    if oop.role.contains("Tracking")
        && ["Marking", "Work Rate", "Stamina"]
            .iter()
            .any(|attribute| attributes.get(*attribute).is_some_and(|value| *value <= 9))
    {
        flags.push(format!("Bad defensive match for {}", oop.role));
    }
    if ip.role.contains("Playmaker")
        && ["Passing", "Composure", "Decisions"]
            .iter()
            .any(|attribute| attributes.get(*attribute).is_some_and(|value| *value <= 9))
    {
        flags.push(format!("Bad build-up match for {}", ip.role));
    }
    flags
}

fn phase_positions_compatible(
    left: &'static [&'static str],
    right: &'static [&'static str],
) -> bool {
    left.iter().any(|left_position| {
        right.iter().any(|right_position| {
            canonical_slot(left_position) == canonical_slot(right_position)
                || position_family(left_position) == position_family(right_position)
        })
    })
}

fn role_json(role: &RoleDefinition) -> serde_json::Value {
    json!({
        "key": role.key,
        "role": role.label,
        "shortRole": role.short_label,
        "roleIdMask": format!("0x{:X}", role.mask),
        "positions": role.positions,
        "phase": "in-possession",
        "veryImportant": role.primary_attributes,
        "important": role.secondary_attributes,
    })
}

fn oop_role_json(role: &OutOfPossessionRoleDefinition) -> serde_json::Value {
    json!({
        "key": role.key,
        "role": role.label,
        "shortRole": role.short_label,
        "roleIdMask": format!("OOP:0x{:X}", role.mask),
        "positions": role.positions,
        "phase": "out-of-possession",
        "veryImportant": role.primary_attributes,
        "important": role.secondary_attributes,
    })
}

fn position_family(slot: &str) -> Option<&'static str> {
    Some(match canonical_slot(slot)? {
        "GK" => "GK",
        "SW" | "DC" => "DC",
        "DL" | "DR" | "WBL" | "WBR" => "FB",
        "DM" => "DM",
        "MC" => "MC",
        "ML" | "MR" | "AML" | "AMR" => "WIDE",
        "AMC" => "AMC",
        "ST" => "ST",
        _ => return None,
    })
}

fn canonical_slot(slot: &str) -> Option<&'static str> {
    Some(match slot {
        "GK" => "GK",
        "SW" => "SW",
        "DC" | "DCR" | "DCL" => "DC",
        "DL" => "DL",
        "DR" => "DR",
        "WBL" => "WBL",
        "WBR" => "WBR",
        "DM" | "DML" | "DMR" => "DM",
        "MC" | "MCL" | "MCR" => "MC",
        "ML" => "ML",
        "MR" => "MR",
        "AMC" | "AMCL" | "AMCR" => "AMC",
        "AML" => "AML",
        "AMR" => "AMR",
        "ST" | "STC" | "STL" | "STR" | "STCL" | "STCR" => "ST",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fm26_role_catalogue_is_phase_split_without_legacy_duties() {
        assert!(role_definition_for_mask(16_777_216)
            .is_some_and(|role| role.label == "Ball-Playing Centre-Back"));
        assert_eq!(DUTY_DEFINITIONS.len(), 0);
        assert!(duty_definition_for_mask(4_194_304).is_none());
        assert!(OUT_OF_POSSESSION_ROLE_DEFINITIONS.len() >= 28);
        let ip_labels = ROLE_DEFINITIONS
            .iter()
            .map(|role| role.label)
            .collect::<Vec<_>>();
        assert!(ip_labels.contains(&"Inside Wing-Back"));
        assert!(ip_labels.contains(&"Free Role"));
        assert!(!ip_labels.contains(&"Mezzala"));
        assert!(!ip_labels.contains(&"Enganche"));
        assert!(!ip_labels.contains(&"Trequartista"));
        assert!(!ip_labels.contains(&"Complete Forward"));
    }

    #[test]
    fn centre_back_scores_phase_split_centre_back_roles() {
        let mut positions = vec![1_u8; POSITION_NAMES.len()];
        positions[POSITION_NAMES
            .iter()
            .position(|name| *name == "DC")
            .unwrap()] = 20;
        let attrs = HashMap::from([
            ("Marking".to_string(), 14),
            ("Tackling".to_string(), 13),
            ("Heading".to_string(), 15),
            ("Positioning".to_string(), 12),
            ("Anticipation".to_string(), 13),
            ("Jumping Reach".to_string(), 14),
            ("Strength".to_string(), 13),
            ("Passing".to_string(), 7),
            ("Technique".to_string(), 6),
            ("Pace".to_string(), 12),
        ]);
        let evaluation = evaluate_player_roles(&positions, &attrs);
        let best = evaluation.best.expect("best role");
        assert!(best.phase == "combined");
        assert!(best.in_possession_role.is_some());
        assert!(best.out_of_possession_role.is_some());
        assert!(best.role.contains("Centre-Back"));
        assert!(best.score >= 60);
    }

    #[test]
    fn weak_tracking_attributes_create_red_flag() {
        let mut positions = vec![1_u8; POSITION_NAMES.len()];
        positions[POSITION_NAMES
            .iter()
            .position(|name| *name == "AMR")
            .unwrap()] = 20;
        let attrs = HashMap::from([
            ("Crossing".to_string(), 16),
            ("Dribbling".to_string(), 16),
            ("Technique".to_string(), 15),
            ("Teamwork".to_string(), 14),
            ("Acceleration".to_string(), 15),
            ("Agility".to_string(), 15),
            ("Pace".to_string(), 15),
            ("Marking".to_string(), 5),
            ("Work Rate".to_string(), 6),
            ("Stamina".to_string(), 7),
            ("Anticipation".to_string(), 12),
            ("Off the Ball".to_string(), 14),
            ("Decisions".to_string(), 12),
        ]);
        let tracking_winger = OUT_OF_POSSESSION_ROLE_DEFINITIONS
            .iter()
            .find(|role| role.label == "Tracking Winger")
            .expect("tracking winger role");
        let scored = score_out_of_possession_role(tracking_winger, &positions, &attrs)
            .expect("scored tracking winger");
        assert!(scored
            .red_flags
            .iter()
            .any(|flag| flag.contains("Bad defensive match")));
    }
}
