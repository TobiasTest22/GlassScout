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
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PlayerRoleFit {
    pub(crate) role_key: &'static str,
    pub(crate) role: &'static str,
    pub(crate) short_role: &'static str,
    pub(crate) role_id_mask: String,
    pub(crate) positions: &'static [&'static str],
    pub(crate) score: u8,
    pub(crate) position_fit: u8,
    pub(crate) attribute_fit: Option<u8>,
    pub(crate) evidence: Vec<String>,
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

pub(crate) const ROLE_DEFINITIONS: &[RoleDefinition] = &[
    RoleDefinition {
        key: "goalkeeper",
        label: "Goalkeeper",
        short_label: "GK",
        mask: 1,
        positions: &["GK"],
        primary_attributes: &[
            "Handling",
            "Reflexes",
            "Aerial Reach",
            "Command of Area",
            "Communication",
            "Positioning",
        ],
        secondary_attributes: &["Decisions", "One on Ones", "Kicking"],
    },
    RoleDefinition {
        key: "sweeper_keeper",
        label: "Sweeper Keeper",
        short_label: "SK",
        mask: 4_096,
        positions: &["GK"],
        primary_attributes: &[
            "Reflexes",
            "One on Ones",
            "Rushing Out",
            "Kicking",
            "Passing",
            "First Touch",
        ],
        secondary_attributes: &["Decisions", "Composure", "Acceleration"],
    },
    RoleDefinition {
        key: "no_nonsense_goalkeeper",
        label: "No-Nonsense Goalkeeper",
        short_label: "NNGK",
        mask: 9_007_199_254_740_992,
        positions: &["GK"],
        primary_attributes: &[
            "Handling",
            "Reflexes",
            "Aerial Reach",
            "Command of Area",
            "Communication",
        ],
        secondary_attributes: &["Kicking", "Decisions", "Positioning"],
    },
    RoleDefinition {
        key: "central_defender",
        label: "Central Defender",
        short_label: "CD",
        mask: 2,
        positions: &["DC"],
        primary_attributes: &[
            "Marking",
            "Tackling",
            "Heading",
            "Positioning",
            "Jumping Reach",
            "Strength",
        ],
        secondary_attributes: &["Anticipation", "Concentration", "Decisions"],
    },
    RoleDefinition {
        key: "ball_playing_defender",
        label: "Ball Playing Defender",
        short_label: "BPD",
        mask: 16_777_216,
        positions: &["DC"],
        primary_attributes: &[
            "Marking",
            "Tackling",
            "Heading",
            "Passing",
            "First Touch",
            "Technique",
        ],
        secondary_attributes: &["Composure", "Vision", "Positioning", "Decisions"],
    },
    RoleDefinition {
        key: "no_nonsense_centre_back",
        label: "No-Nonsense Centre-Back",
        short_label: "NCB",
        mask: 536_870_912,
        positions: &["DC"],
        primary_attributes: &[
            "Heading",
            "Tackling",
            "Marking",
            "Positioning",
            "Jumping Reach",
            "Strength",
        ],
        secondary_attributes: &["Bravery", "Concentration", "Aggression"],
    },
    RoleDefinition {
        key: "wide_centre_back",
        label: "Wide Centre-Back",
        short_label: "WCB",
        mask: 2_251_799_813_685_248,
        positions: &["DC"],
        primary_attributes: &[
            "Marking",
            "Tackling",
            "Positioning",
            "Pace",
            "Stamina",
            "Passing",
        ],
        secondary_attributes: &["Crossing", "Dribbling", "Work Rate", "Decisions"],
    },
    RoleDefinition {
        key: "overlapping_centre_back",
        label: "Overlapping Centre-Back",
        short_label: "OCB",
        mask: 18_014_398_509_481_984,
        positions: &["DC"],
        primary_attributes: &[
            "Marking",
            "Tackling",
            "Passing",
            "Stamina",
            "Work Rate",
            "Dribbling",
        ],
        secondary_attributes: &["Crossing", "Pace", "Technique", "Decisions"],
    },
    RoleDefinition {
        key: "libero",
        label: "Libero",
        short_label: "LIB",
        mask: 16_384,
        positions: &["DC", "SW"],
        primary_attributes: &[
            "Passing",
            "First Touch",
            "Technique",
            "Decisions",
            "Positioning",
            "Anticipation",
        ],
        secondary_attributes: &["Marking", "Tackling", "Vision", "Composure"],
    },
    RoleDefinition {
        key: "full_back",
        label: "Full-Back",
        short_label: "FB",
        mask: 4,
        positions: &["DL", "DR"],
        primary_attributes: &[
            "Marking",
            "Tackling",
            "Positioning",
            "Crossing",
            "Stamina",
            "Work Rate",
        ],
        secondary_attributes: &["Pace", "Acceleration", "Teamwork"],
    },
    RoleDefinition {
        key: "wing_back",
        label: "Wing-Back",
        short_label: "WB",
        mask: 8,
        positions: &["DL", "DR", "WBL", "WBR"],
        primary_attributes: &[
            "Crossing",
            "Dribbling",
            "Stamina",
            "Work Rate",
            "Pace",
            "Acceleration",
        ],
        secondary_attributes: &["Tackling", "Positioning", "Teamwork", "Off the Ball"],
    },
    RoleDefinition {
        key: "advanced_wing_back",
        label: "Advanced Wing-Back",
        short_label: "AWB",
        mask: 274_877_906_944,
        positions: &["WBL", "WBR", "DL", "DR"],
        primary_attributes: &[
            "Crossing",
            "Dribbling",
            "Acceleration",
            "Pace",
            "Stamina",
            "Off the Ball",
        ],
        secondary_attributes: &["Technique", "Work Rate", "Decisions"],
    },
    RoleDefinition {
        key: "inverted_wing_back",
        label: "Inverted Wing-Back",
        short_label: "IWB",
        mask: 17_592_186_044_416,
        positions: &["DL", "DR", "WBL", "WBR"],
        primary_attributes: &[
            "Passing",
            "First Touch",
            "Technique",
            "Decisions",
            "Teamwork",
            "Positioning",
        ],
        secondary_attributes: &["Tackling", "Work Rate", "Stamina"],
    },
    RoleDefinition {
        key: "inverted_full_back",
        label: "Inverted Full-Back",
        short_label: "IFB",
        mask: 4_503_599_627_370_496,
        positions: &["DL", "DR"],
        primary_attributes: &[
            "Positioning",
            "Tackling",
            "Marking",
            "Passing",
            "Decisions",
            "Teamwork",
        ],
        secondary_attributes: &["Concentration", "Strength", "Work Rate"],
    },
    RoleDefinition {
        key: "playmaking_wing_back",
        label: "Playmaking Wing-Back",
        short_label: "PWB",
        mask: 36_028_797_018_963_968,
        positions: &["DL", "DR", "WBL", "WBR"],
        primary_attributes: &[
            "Passing",
            "Vision",
            "Technique",
            "First Touch",
            "Crossing",
            "Decisions",
        ],
        secondary_attributes: &["Dribbling", "Teamwork", "Stamina"],
    },
    RoleDefinition {
        key: "no_nonsense_full_back",
        label: "No-Nonsense Full-Back",
        short_label: "NFB",
        mask: 68_719_476_736,
        positions: &["DL", "DR"],
        primary_attributes: &[
            "Tackling",
            "Marking",
            "Positioning",
            "Strength",
            "Heading",
            "Work Rate",
        ],
        secondary_attributes: &["Bravery", "Concentration", "Teamwork"],
    },
    RoleDefinition {
        key: "defensive_midfielder",
        label: "Defensive Midfielder",
        short_label: "DM",
        mask: 16,
        positions: &["DM"],
        primary_attributes: &[
            "Tackling",
            "Positioning",
            "Anticipation",
            "Decisions",
            "Teamwork",
            "Work Rate",
        ],
        secondary_attributes: &["Passing", "Strength", "Stamina"],
    },
    RoleDefinition {
        key: "anchor",
        label: "Anchor",
        short_label: "A",
        mask: 8_589_934_592,
        positions: &["DM"],
        primary_attributes: &[
            "Tackling",
            "Positioning",
            "Marking",
            "Concentration",
            "Strength",
            "Decisions",
        ],
        secondary_attributes: &["Anticipation", "Teamwork", "Work Rate"],
    },
    RoleDefinition {
        key: "half_back",
        label: "Half Back",
        short_label: "HB",
        mask: 34_359_738_368,
        positions: &["DM"],
        primary_attributes: &[
            "Positioning",
            "Marking",
            "Tackling",
            "Passing",
            "Teamwork",
            "Decisions",
        ],
        secondary_attributes: &["Composure", "Anticipation", "Strength"],
    },
    RoleDefinition {
        key: "regista",
        label: "Regista",
        short_label: "REG",
        mask: 549_755_813_888,
        positions: &["DM"],
        primary_attributes: &[
            "Passing",
            "Vision",
            "Technique",
            "First Touch",
            "Decisions",
            "Flair",
        ],
        secondary_attributes: &["Composure", "Teamwork", "Anticipation"],
    },
    RoleDefinition {
        key: "segundo_volante",
        label: "Segundo Volante",
        short_label: "SV",
        mask: 1_125_899_906_842_624,
        positions: &["DM"],
        primary_attributes: &[
            "Passing",
            "Off the Ball",
            "Stamina",
            "Work Rate",
            "Tackling",
            "Decisions",
        ],
        secondary_attributes: &["Finishing", "Long Shots", "Technique", "Teamwork"],
    },
    RoleDefinition {
        key: "central_midfielder",
        label: "Central Midfielder",
        short_label: "CM",
        mask: 32,
        positions: &["MC"],
        primary_attributes: &[
            "Passing",
            "First Touch",
            "Decisions",
            "Teamwork",
            "Work Rate",
            "Stamina",
        ],
        secondary_attributes: &["Tackling", "Positioning", "Technique"],
    },
    RoleDefinition {
        key: "deep_lying_playmaker",
        label: "Deep-Lying Playmaker",
        short_label: "DLP",
        mask: 32_768,
        positions: &["MC", "DM"],
        primary_attributes: &[
            "Passing",
            "Vision",
            "Technique",
            "First Touch",
            "Decisions",
            "Composure",
        ],
        secondary_attributes: &["Teamwork", "Positioning", "Anticipation"],
    },
    RoleDefinition {
        key: "box_to_box_midfielder",
        label: "Box-to-Box Midfielder",
        short_label: "BBM",
        mask: 65_536,
        positions: &["MC"],
        primary_attributes: &[
            "Stamina",
            "Work Rate",
            "Teamwork",
            "Passing",
            "Tackling",
            "Off the Ball",
        ],
        secondary_attributes: &["Finishing", "Long Shots", "Strength", "Pace"],
    },
    RoleDefinition {
        key: "ball_winning_midfielder",
        label: "Ball-Winning Midfielder",
        short_label: "BWM",
        mask: 268_435_456,
        positions: &["MC", "DM"],
        primary_attributes: &[
            "Tackling",
            "Aggression",
            "Work Rate",
            "Teamwork",
            "Stamina",
            "Positioning",
        ],
        secondary_attributes: &["Strength", "Bravery", "Anticipation"],
    },
    RoleDefinition {
        key: "mezzala",
        label: "Mezzala",
        short_label: "MEZ",
        mask: 140_737_488_355_328,
        positions: &["MC"],
        primary_attributes: &[
            "Dribbling",
            "Passing",
            "Technique",
            "Off the Ball",
            "Vision",
            "Stamina",
        ],
        secondary_attributes: &["First Touch", "Decisions", "Work Rate"],
    },
    RoleDefinition {
        key: "advanced_playmaker",
        label: "Advanced Playmaker",
        short_label: "AP",
        mask: 131_072,
        positions: &["MC", "AMC", "AML", "AMR"],
        primary_attributes: &[
            "Passing",
            "Vision",
            "Technique",
            "First Touch",
            "Decisions",
            "Flair",
        ],
        secondary_attributes: &["Composure", "Off the Ball", "Teamwork"],
    },
    RoleDefinition {
        key: "midfield_playmaker",
        label: "Midfield Playmaker",
        short_label: "MP",
        mask: 144_115_188_075_855_872,
        positions: &["MC"],
        primary_attributes: &[
            "Passing",
            "Vision",
            "Technique",
            "First Touch",
            "Decisions",
            "Teamwork",
        ],
        secondary_attributes: &["Composure", "Positioning", "Anticipation"],
    },
    RoleDefinition {
        key: "box_to_box_playmaker",
        label: "Box-to-Box Playmaker",
        short_label: "BBP",
        mask: 70_368_744_177_664,
        positions: &["MC"],
        primary_attributes: &[
            "Passing",
            "Vision",
            "Stamina",
            "Work Rate",
            "Technique",
            "Decisions",
        ],
        secondary_attributes: &["Tackling", "First Touch", "Teamwork"],
    },
    RoleDefinition {
        key: "attacking_midfielder",
        label: "Attacking Midfielder",
        short_label: "AM",
        mask: 512,
        positions: &["AMC"],
        primary_attributes: &[
            "First Touch",
            "Passing",
            "Technique",
            "Off the Ball",
            "Decisions",
            "Vision",
        ],
        secondary_attributes: &["Dribbling", "Flair", "Composure"],
    },
    RoleDefinition {
        key: "enganche",
        label: "Enganche",
        short_label: "ENG",
        mask: 137_438_953_472,
        positions: &["AMC"],
        primary_attributes: &[
            "Passing",
            "Vision",
            "Technique",
            "First Touch",
            "Composure",
            "Flair",
        ],
        secondary_attributes: &["Decisions", "Off the Ball", "Teamwork"],
    },
    RoleDefinition {
        key: "trequartista",
        label: "Trequartista",
        short_label: "TQ",
        mask: 4_294_967_296,
        positions: &["AMC", "ST"],
        primary_attributes: &[
            "Technique",
            "Flair",
            "First Touch",
            "Passing",
            "Vision",
            "Off the Ball",
        ],
        secondary_attributes: &["Composure", "Finishing", "Dribbling"],
    },
    RoleDefinition {
        key: "shadow_striker",
        label: "Shadow Striker",
        short_label: "SS",
        mask: 2_199_023_255_552,
        positions: &["AMC"],
        primary_attributes: &[
            "Finishing",
            "Off the Ball",
            "Anticipation",
            "Composure",
            "Acceleration",
            "Pace",
        ],
        secondary_attributes: &["Technique", "First Touch", "Work Rate"],
    },
    RoleDefinition {
        key: "channel_midfielder",
        label: "Channel Midfielder",
        short_label: "CHM",
        mask: 72_057_594_037_927_936,
        positions: &["MC", "AMC"],
        primary_attributes: &[
            "Off the Ball",
            "Work Rate",
            "Stamina",
            "Passing",
            "Dribbling",
            "Decisions",
        ],
        secondary_attributes: &["Pace", "Technique", "Teamwork"],
    },
    RoleDefinition {
        key: "wide_central_midfielder",
        label: "Wide Central Midfielder",
        short_label: "WCM",
        mask: 281_474_976_710_656,
        positions: &["MC"],
        primary_attributes: &[
            "Crossing",
            "Passing",
            "Work Rate",
            "Stamina",
            "Teamwork",
            "Decisions",
        ],
        secondary_attributes: &["Dribbling", "Tackling", "Positioning"],
    },
    RoleDefinition {
        key: "wide_midfielder",
        label: "Wide Midfielder",
        short_label: "WM",
        mask: 64,
        positions: &["ML", "MR"],
        primary_attributes: &[
            "Crossing",
            "Passing",
            "Work Rate",
            "Stamina",
            "Teamwork",
            "Positioning",
        ],
        secondary_attributes: &["Tackling", "Dribbling", "Pace"],
    },
    RoleDefinition {
        key: "winger",
        label: "Winger",
        short_label: "W",
        mask: 128,
        positions: &["ML", "MR", "AML", "AMR"],
        primary_attributes: &[
            "Crossing",
            "Dribbling",
            "Acceleration",
            "Pace",
            "Technique",
            "Off the Ball",
        ],
        secondary_attributes: &["Flair", "First Touch", "Work Rate"],
    },
    RoleDefinition {
        key: "inside_forward",
        label: "Inside Forward",
        short_label: "IF",
        mask: 134_217_728,
        positions: &["AML", "AMR"],
        primary_attributes: &[
            "Dribbling",
            "Finishing",
            "Off the Ball",
            "Acceleration",
            "Pace",
            "Technique",
        ],
        secondary_attributes: &["First Touch", "Composure", "Flair"],
    },
    RoleDefinition {
        key: "inverted_winger",
        label: "Inverted Winger",
        short_label: "IW",
        mask: 562_949_953_421_312,
        positions: &["AML", "AMR"],
        primary_attributes: &[
            "Dribbling",
            "Passing",
            "Technique",
            "First Touch",
            "Vision",
            "Acceleration",
        ],
        secondary_attributes: &["Off the Ball", "Flair", "Decisions"],
    },
    RoleDefinition {
        key: "defensive_winger",
        label: "Defensive Winger",
        short_label: "DW",
        mask: 1_073_741_824,
        positions: &["ML", "MR", "AML", "AMR"],
        primary_attributes: &[
            "Work Rate",
            "Teamwork",
            "Tackling",
            "Positioning",
            "Stamina",
            "Acceleration",
        ],
        secondary_attributes: &["Crossing", "Pace", "Marking"],
    },
    RoleDefinition {
        key: "wide_target_forward",
        label: "Wide Target Forward",
        short_label: "WTF",
        mask: 4_398_046_511_104,
        positions: &["AML", "AMR", "ML", "MR"],
        primary_attributes: &[
            "Heading",
            "Jumping Reach",
            "Strength",
            "First Touch",
            "Off the Ball",
            "Teamwork",
        ],
        secondary_attributes: &["Finishing", "Balance", "Bravery"],
    },
    RoleDefinition {
        key: "wide_playmaker",
        label: "Wide Playmaker",
        short_label: "WP",
        mask: 8_796_093_022_208,
        positions: &["ML", "MR", "AML", "AMR"],
        primary_attributes: &[
            "Passing",
            "Vision",
            "Technique",
            "First Touch",
            "Decisions",
            "Flair",
        ],
        secondary_attributes: &["Dribbling", "Teamwork", "Off the Ball"],
    },
    RoleDefinition {
        key: "wide_forward",
        label: "Wide Forward",
        short_label: "WF",
        mask: 35_184_372_088_832,
        positions: &["AML", "AMR"],
        primary_attributes: &[
            "Finishing",
            "Off the Ball",
            "Acceleration",
            "Pace",
            "Dribbling",
            "Composure",
        ],
        secondary_attributes: &["Technique", "First Touch", "Anticipation"],
    },
    RoleDefinition {
        key: "deep_lying_forward",
        label: "Deep-Lying Forward",
        short_label: "DLF",
        mask: 1_024,
        positions: &["ST"],
        primary_attributes: &[
            "First Touch",
            "Passing",
            "Technique",
            "Teamwork",
            "Composure",
            "Off the Ball",
        ],
        secondary_attributes: &["Strength", "Finishing", "Vision"],
    },
    RoleDefinition {
        key: "centre_forward",
        label: "Centre Forward",
        short_label: "CF",
        mask: 2_048,
        positions: &["ST"],
        primary_attributes: &[
            "Finishing",
            "First Touch",
            "Off the Ball",
            "Composure",
            "Anticipation",
            "Technique",
        ],
        secondary_attributes: &["Pace", "Strength", "Heading"],
    },
    RoleDefinition {
        key: "target_forward",
        label: "Target Forward",
        short_label: "TF",
        mask: 262_144,
        positions: &["ST"],
        primary_attributes: &[
            "Heading",
            "Jumping Reach",
            "Strength",
            "First Touch",
            "Teamwork",
            "Bravery",
        ],
        secondary_attributes: &["Finishing", "Composure", "Balance"],
    },
    RoleDefinition {
        key: "poacher",
        label: "Poacher",
        short_label: "P",
        mask: 524_288,
        positions: &["ST"],
        primary_attributes: &[
            "Finishing",
            "Off the Ball",
            "Anticipation",
            "Composure",
            "Acceleration",
            "First Touch",
        ],
        secondary_attributes: &["Pace", "Agility", "Technique"],
    },
    RoleDefinition {
        key: "complete_forward",
        label: "Complete Forward",
        short_label: "CF",
        mask: 1_048_576,
        positions: &["ST"],
        primary_attributes: &[
            "Finishing",
            "First Touch",
            "Technique",
            "Off the Ball",
            "Composure",
            "Strength",
        ],
        secondary_attributes: &["Passing", "Heading", "Pace", "Vision"],
    },
    RoleDefinition {
        key: "false_nine",
        label: "False Nine",
        short_label: "F9",
        mask: 1_099_511_627_776,
        positions: &["ST"],
        primary_attributes: &[
            "First Touch",
            "Passing",
            "Vision",
            "Technique",
            "Off the Ball",
            "Composure",
        ],
        secondary_attributes: &["Dribbling", "Flair", "Decisions"],
    },
    RoleDefinition {
        key: "channel_forward",
        label: "Channel Forward",
        short_label: "CHF",
        mask: 2_147_483_648,
        positions: &["ST"],
        primary_attributes: &[
            "Off the Ball",
            "Pace",
            "Acceleration",
            "Work Rate",
            "Finishing",
            "Dribbling",
        ],
        secondary_attributes: &["Strength", "Technique", "Composure"],
    },
];

pub(crate) const DUTY_DEFINITIONS: &[DutyDefinition] = &[
    DutyDefinition {
        key: "defend",
        label: "Defend",
        short_label: "D",
        mask: 2_097_152,
    },
    DutyDefinition {
        key: "stopper",
        label: "Stopper",
        short_label: "St",
        mask: 33_554_432,
    },
    DutyDefinition {
        key: "cover",
        label: "Cover",
        short_label: "Co",
        mask: 67_108_864,
    },
    DutyDefinition {
        key: "support",
        label: "Support",
        short_label: "S",
        mask: 4_194_304,
    },
    DutyDefinition {
        key: "attack",
        label: "Attack",
        short_label: "A",
        mask: 8_388_608,
    },
    DutyDefinition {
        key: "float",
        label: "Float",
        short_label: "F",
        mask: 17_179_869_184,
    },
];

pub(crate) const OUT_OF_POSSESSION_ROLE_DEFINITIONS: &[OutOfPossessionRoleDefinition] = &[
    OutOfPossessionRoleDefinition {
        key: "op_goalkeeper",
        label: "Goalkeeper",
        short_label: "GK",
        mask: 1,
        positions: &["GK"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_sweeper_keeper",
        label: "Sweeper Keeper",
        short_label: "SK",
        mask: 2,
        positions: &["GK"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_line_keeper",
        label: "Line Keeper",
        short_label: "LK",
        mask: 4,
        positions: &["GK"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_centre_back",
        label: "Centre Back",
        short_label: "CB",
        mask: 8,
        positions: &["DC"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_stopping_centre_back",
        label: "Stopping Centre Back",
        short_label: "SCB",
        mask: 16,
        positions: &["DC"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_covering_centre_back",
        label: "Covering Centre Back",
        short_label: "CCB",
        mask: 32,
        positions: &["DC"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_wide_centre_back",
        label: "Wide Centre Back",
        short_label: "WCB",
        mask: 64,
        positions: &["DC"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_stopping_wide_centre_back",
        label: "Stopping Wide Centre Back",
        short_label: "SWCB",
        mask: 128,
        positions: &["DC"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_covering_wide_centre_back",
        label: "Covering Wide Centre Back",
        short_label: "CWCB",
        mask: 256,
        positions: &["DC"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_full_back",
        label: "Full Back",
        short_label: "FB",
        mask: 512,
        positions: &["DL", "DR"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_pressing_full_back",
        label: "Pressing Full Back",
        short_label: "PFB",
        mask: 1_024,
        positions: &["DL", "DR"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_holding_full_back",
        label: "Holding Full Back",
        short_label: "HFB",
        mask: 2_048,
        positions: &["DL", "DR"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_wing_back",
        label: "Wing Back",
        short_label: "WB",
        mask: 4_096,
        positions: &["WBL", "WBR", "DL", "DR"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_pressing_wing_back",
        label: "Pressing Wing Back",
        short_label: "PWB",
        mask: 8_192,
        positions: &["WBL", "WBR", "DL", "DR"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_holding_wing_back",
        label: "Holding Wing Back",
        short_label: "HWB",
        mask: 16_384,
        positions: &["WBL", "WBR", "DL", "DR"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_defensive_midfielder",
        label: "Defensive Midfielder",
        short_label: "DM",
        mask: 32_768,
        positions: &["DM"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_dropping_defensive_midfielder",
        label: "Dropping Defensive Midfielder",
        short_label: "DDM",
        mask: 65_536,
        positions: &["DM"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_pressing_defensive_midfielder",
        label: "Pressing Defensive Midfielder",
        short_label: "PDM",
        mask: 131_072,
        positions: &["DM"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_screening_defensive_midfielder",
        label: "Screening Defensive Midfielder",
        short_label: "SDM",
        mask: 262_144,
        positions: &["DM"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_wide_cover_defensive_midfielder",
        label: "Wide Cover Defensive Midfielder",
        short_label: "WCDM",
        mask: 524_288,
        positions: &["DM"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_central_midfielder",
        label: "Central Midfielder",
        short_label: "CM",
        mask: 1_048_576,
        positions: &["MC"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_pressing_central_midfielder",
        label: "Pressing Central Midfielder",
        short_label: "PCM",
        mask: 134_217_728,
        positions: &["MC"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_screening_central_midfielder",
        label: "Screening Central Midfielder",
        short_label: "SCM",
        mask: 268_435_456,
        positions: &["MC"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_wide_cover_central_midfielder",
        label: "Wide Cover Central Midfielder",
        short_label: "WCCM",
        mask: 536_870_912,
        positions: &["MC"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_attacking_midfielder",
        label: "Attacking Midfielder",
        short_label: "AM",
        mask: 1_073_741_824,
        positions: &["AMC"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_tracking_attacking_midfielder",
        label: "Tracking Attacking Midfielder",
        short_label: "TAM",
        mask: 2_147_483_648,
        positions: &["AMC"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_central_outlet_midfielder",
        label: "Central Outlet Midfielder",
        short_label: "COM",
        mask: 4_294_967_296,
        positions: &["AMC"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_splitting_outlet_attacking_midfielder",
        label: "Splitting Outlet Attacking Midfielder",
        short_label: "SOAM",
        mask: 8_589_934_592,
        positions: &["AMC"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_wide_midfielder",
        label: "Wide Midfielder",
        short_label: "WM",
        mask: 34_359_738_368,
        positions: &["ML", "MR"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_tracking_wide_midfielder",
        label: "Tracking Wide Midfielder",
        short_label: "TWM",
        mask: 68_719_476_736,
        positions: &["ML", "MR"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_wide_outlet_wide_midfielder",
        label: "Wide Outlet Wide Midfielder",
        short_label: "WOWM",
        mask: 137_438_953_472,
        positions: &["ML", "MR"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_winger",
        label: "Winger",
        short_label: "W",
        mask: 274_877_906_944,
        positions: &["AML", "AMR", "ML", "MR"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_tracking_winger",
        label: "Tracking Winger",
        short_label: "TW",
        mask: 549_755_813_888,
        positions: &["AML", "AMR", "ML", "MR"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_inverting_outlet_winger",
        label: "Inverting Outlet Winger",
        short_label: "IOW",
        mask: 1_099_511_627_776,
        positions: &["AML", "AMR"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_wide_outlet_winger",
        label: "Wide Outlet Winger",
        short_label: "WOW",
        mask: 2_199_023_255_552,
        positions: &["AML", "AMR"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_centre_forward",
        label: "Centre Forward",
        short_label: "CF",
        mask: 4_398_046_511_104,
        positions: &["ST"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_tracking_centre_forward",
        label: "Tracking Centre Forward",
        short_label: "TCF",
        mask: 8_796_093_022_208,
        positions: &["ST"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_central_outlet_centre_forward",
        label: "Central Outlet Centre Forward",
        short_label: "COCF",
        mask: 17_592_186_044_416,
        positions: &["ST"],
    },
    OutOfPossessionRoleDefinition {
        key: "op_splitting_outlet_centre_forward",
        label: "Splitting Outlet Centre Forward",
        short_label: "SOCF",
        mask: 35_184_372_088_832,
        positions: &["ST"],
    },
];

pub(crate) fn evaluate_player_roles(
    position_bytes: &[u8],
    attributes: &HashMap<String, u8>,
) -> PlayerRoleEvaluation {
    let mut fits = ROLE_DEFINITIONS
        .iter()
        .filter_map(|role| score_role(role, position_bytes, attributes))
        .collect::<Vec<_>>();
    fits.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| right.position_fit.cmp(&left.position_fit))
            .then_with(|| left.role.cmp(right.role))
    });

    let best = fits.first().cloned();
    let playable = fits
        .iter()
        .filter(|fit| fit.position_fit >= 55 && fit.score >= 45)
        .take(8)
        .cloned()
        .collect::<Vec<_>>();
    let secondary = fits
        .iter()
        .filter(|fit| fit.position_fit >= 30 && fit.score >= 35)
        .skip(playable.len())
        .take(8)
        .cloned()
        .collect::<Vec<_>>();
    let reasoning = match &best {
        Some(fit) => vec![
            format!(
                "{} is the highest FM26 role fit from mapped position familiarity and visible attributes.",
                fit.role
            ),
            format!(
                "Position fit {}%, attribute fit {}%.",
                fit.position_fit,
                fit.attribute_fit
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            ),
            "Role catalogue comes from FM26 metadata masks; scores use GlassScout's own visible-evidence model.".to_string(),
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
    let duty = DUTY_DEFINITIONS
        .iter()
        .find(|definition| value & definition.mask == definition.mask);
    let known_role_bits = role.map(|definition| definition.mask).unwrap_or_default();
    let known_duty_bits = duty.map(|definition| definition.mask).unwrap_or_default();
    DecodedRoleDuty {
        role,
        duty,
        unknown_bits: value & !(known_role_bits | known_duty_bits),
    }
}

pub(crate) fn role_definition_for_mask(mask: u64) -> Option<&'static RoleDefinition> {
    ROLE_DEFINITIONS
        .iter()
        .find(|definition| definition.mask == mask)
}

pub(crate) fn duty_definition_for_mask(mask: u64) -> Option<&'static DutyDefinition> {
    DUTY_DEFINITIONS
        .iter()
        .find(|definition| definition.mask == mask)
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
        "outOfPossessionRoles": OUT_OF_POSSESSION_ROLE_DEFINITIONS
            .iter()
            .map(|role| json!({
                "key": role.key,
                "role": role.label,
                "shortRole": role.short_label,
                "roleIdMask": format!("0x{:X}", role.mask),
                "positions": role.positions
            }))
            .collect::<Vec<_>>(),
        "source": "FM26 global metadata role/duty bitmask fields",
        "status": "metadata-catalogue-mapped"
    })
}

fn score_role(
    role: &'static RoleDefinition,
    position_bytes: &[u8],
    attributes: &HashMap<String, u8>,
) -> Option<PlayerRoleFit> {
    let (position_fit, top_position, top_familiarity) = role_position_fit(role, position_bytes)?;
    if position_fit < 5 {
        return None;
    }
    let attribute_fit = weighted_attribute_fit(role, attributes);
    let score = match attribute_fit {
        Some(attribute_fit) => {
            (f32::from(position_fit) * 0.38 + f32::from(attribute_fit) * 0.62).round() as u8
        }
        None => position_fit,
    }
    .min(position_fit.saturating_add(25))
    .clamp(0, 100);

    let evidence = role_evidence(role, attributes, top_position, top_familiarity);
    Some(PlayerRoleFit {
        role_key: role.key,
        role: role.label,
        short_role: role.short_label,
        role_id_mask: format!("0x{:X}", role.mask),
        positions: role.positions,
        score,
        position_fit,
        attribute_fit,
        evidence,
    })
}

fn role_position_fit(
    role: &RoleDefinition,
    position_bytes: &[u8],
) -> Option<(u8, &'static str, u8)> {
    let mut best: Option<(&'static str, u8)> = None;
    for position in role.positions {
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

fn weighted_attribute_fit(role: &RoleDefinition, attributes: &HashMap<String, u8>) -> Option<u8> {
    let mut total = 0_u32;
    let mut weight = 0_u32;
    for key in role.primary_attributes {
        if let Some(value) = attributes.get(*key) {
            total += u32::from(*value) * 2;
            weight += 2;
        }
    }
    for key in role.secondary_attributes {
        if let Some(value) = attributes.get(*key) {
            total += u32::from(*value);
            weight += 1;
        }
    }
    (weight >= 4).then(|| {
        let average = total as f32 / weight as f32;
        ((average / 20.0) * 100.0).round().clamp(0.0, 100.0) as u8
    })
}

fn role_evidence(
    role: &RoleDefinition,
    attributes: &HashMap<String, u8>,
    top_position: &'static str,
    top_familiarity: u8,
) -> Vec<String> {
    let mut evidence = vec![format!("{top_position} familiarity {top_familiarity}/20")];
    let mut values = role
        .primary_attributes
        .iter()
        .chain(role.secondary_attributes.iter())
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
    fn fm26_role_catalogue_contains_metadata_masks() {
        assert!(role_definition_for_mask(16_777_216).is_some());
        assert!(duty_definition_for_mask(4_194_304).is_some());
        assert!(OUT_OF_POSSESSION_ROLE_DEFINITIONS.len() >= 39);
    }

    #[test]
    fn centre_back_scores_defender_roles_from_position_and_attributes() {
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
            ("Jumping Reach".to_string(), 14),
            ("Strength".to_string(), 13),
            ("Passing".to_string(), 7),
            ("Technique".to_string(), 6),
        ]);
        let evaluation = evaluate_player_roles(&positions, &attrs);
        let best = evaluation.best.expect("best role");
        assert!(best.role.contains("Defender") || best.role.contains("Centre-Back"));
        assert!(best.score >= 60);
    }
}
