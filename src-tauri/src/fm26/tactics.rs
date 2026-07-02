pub(crate) struct FormationTemplate {
    pub(crate) enum_name: &'static str,
    pub(crate) label: &'static str,
    pub(crate) positions: &'static [&'static str],
    pub(crate) exact_shape: bool,
}

const GENERIC_XI: &[&str] = &[
    "GK", "Slot 2", "Slot 3", "Slot 4", "Slot 5", "Slot 6", "Slot 7", "Slot 8", "Slot 9",
    "Slot 10", "Slot 11",
];
const P442: &[&str] = &[
    "GK", "DR", "DCR", "DCL", "DL", "MR", "MCR", "MCL", "ML", "STCR", "STCL",
];
const P433_DM_WIDE: &[&str] = &[
    "GK", "DR", "DCR", "DCL", "DL", "DM", "MCR", "MCL", "AMR", "AML", "ST",
];
const P424_WIDE: &[&str] = &[
    "GK", "DR", "DCR", "DCL", "DL", "MCR", "MCL", "AMR", "AML", "STCR", "STCL",
];
const P352: &[&str] = &[
    "GK", "DCR", "DC", "DCL", "MR", "MCR", "MC", "MCL", "ML", "STCR", "STCL",
];
const P41212: &[&str] = &[
    "GK", "DR", "DCR", "DCL", "DL", "DM", "MCR", "MCL", "AMC", "STCR", "STCL",
];
const P451: &[&str] = &[
    "GK", "DR", "DCR", "DCL", "DL", "MR", "MCR", "MC", "MCL", "ML", "ST",
];
const P343: &[&str] = &[
    "GK", "DCR", "DC", "DCL", "MR", "MCR", "MCL", "ML", "AMR", "AML", "ST",
];
const P3412: &[&str] = &[
    "GK", "DCR", "DC", "DCL", "MR", "MCR", "MCL", "ML", "AMC", "STCR", "STCL",
];
const P3421: &[&str] = &[
    "GK", "DCR", "DC", "DCL", "MR", "MCR", "MCL", "ML", "AMCR", "AMCL", "ST",
];
const P4321_NARROW: &[&str] = &[
    "GK", "DR", "DCR", "DCL", "DL", "MCR", "MC", "MCL", "AMCR", "AMCL", "ST",
];
const P4312: &[&str] = &[
    "GK", "DR", "DCR", "DCL", "DL", "MCR", "MC", "MCL", "AMC", "STCR", "STCL",
];
const P4411: &[&str] = &[
    "GK", "DR", "DCR", "DCL", "DL", "MR", "MCR", "MCL", "ML", "AMC", "ST",
];
const P4231_WIDE: &[&str] = &[
    "GK", "DR", "DCR", "DCL", "DL", "MCR", "MCL", "AMR", "AMC", "AML", "ST",
];
const P4231_NARROW: &[&str] = &[
    "GK", "DR", "DCR", "DCL", "DL", "MCR", "MCL", "AMCR", "AMC", "AMCL", "ST",
];
const P4222_DM_WIDE: &[&str] = &[
    "GK", "DR", "DCR", "DCL", "DL", "DMR", "DML", "AMR", "AML", "STCR", "STCL",
];
const P4222_DM_NARROW: &[&str] = &[
    "GK", "DR", "DCR", "DCL", "DL", "DMR", "DML", "AMCR", "AMCL", "STCR", "STCL",
];
const P433_DM_NARROW: &[&str] = &[
    "GK", "DR", "DCR", "DCL", "DL", "DM", "MCR", "MCL", "STR", "STC", "STL",
];
const P4321_DM_NARROW: &[&str] = &[
    "GK", "DR", "DCR", "DCL", "DL", "DM", "MCR", "MCL", "AMCR", "AMCL", "ST",
];
const P4231_DM_WIDE: &[&str] = &[
    "GK", "DR", "DCR", "DCL", "DL", "DMR", "DML", "AMR", "AMC", "AML", "ST",
];
const P4141_DM: &[&str] = &[
    "GK", "DR", "DCR", "DCL", "DL", "DM", "MR", "MCR", "MCL", "ML", "ST",
];
const P433_NARROW: &[&str] = &[
    "GK", "DR", "DCR", "DCL", "DL", "MCR", "MC", "MCL", "STR", "STC", "STL",
];
const P424: &[&str] = &[
    "GK", "DR", "DCR", "DCL", "DL", "MCR", "MCL", "AMR", "AML", "STCR", "STCL",
];
const P541: &[&str] = &[
    "GK", "DR", "DCR", "DC", "DCL", "DL", "MR", "MCR", "MCL", "ML", "ST",
];
const P5212_WB: &[&str] = &[
    "GK", "WBR", "DCR", "DC", "DCL", "WBL", "MCR", "MCL", "AMC", "STCR", "STCL",
];
const P5221_WB: &[&str] = &[
    "GK", "WBR", "DCR", "DC", "DCL", "WBL", "MCR", "MCL", "AMR", "AML", "ST",
];

pub(crate) fn tactic_formation_template(code: u32) -> Option<FormationTemplate> {
    let (enum_name, label) = tactic_formation_name(code)?;
    let positions = match code {
        3 => P442,
        4 => P433_DM_WIDE,
        5 => P424_WIDE,
        6 => P352,
        7 => P41212,
        8 => P451,
        9 => P343,
        10 => P3412,
        11 => P3421,
        12 => P4321_NARROW,
        14 => P4312,
        16 => P4411,
        18 => P4231_NARROW,
        21 => P4231_WIDE,
        22 | 28 => P4222_DM_WIDE,
        24 => P4222_DM_NARROW,
        26 | 39 | 40 => P433_DM_NARROW,
        27 => P4321_DM_NARROW,
        29 | 54 => P4231_DM_WIDE,
        32 => P4141_DM,
        36 => P433_NARROW,
        37 => P424,
        38 | 57 | 58 => P541,
        41 | 64 => P5212_WB,
        59 => P5221_WB,
        _ => GENERIC_XI,
    };
    Some(FormationTemplate {
        enum_name,
        label,
        positions,
        exact_shape: positions != GENERIC_XI,
    })
}

pub(crate) fn tactic_formation_name(code: u32) -> Option<(&'static str, &'static str)> {
    Some(match code {
        0 => ("FormationStart", "Formation start"),
        1 => ("Formation532SWWBNotInUse", "5-3-2 SW WB"),
        2 => ("Formation532WB", "5-3-2 WB"),
        3 => ("Formation442", "4-4-2"),
        4 => ("Formation433DMWide", "4-1-2-3 DM Wide"),
        5 => ("Formation424Wide", "4-2-4 Wide"),
        6 => ("Formation352", "3-5-2"),
        7 => ("Formation442DiamondNarrow", "4-1-2-1-2 Diamond Narrow"),
        8 => ("Formation451", "4-5-1"),
        9 => ("Formation343", "3-4-3"),
        10 => ("Formation3412", "3-4-1-2"),
        11 => ("Formation3421", "3-4-2-1"),
        12 => ("Formation4321Narrow", "4-3-2-1 Narrow"),
        13 => ("Formation352SWNotInUse", "3-5-2 SW"),
        14 => ("Formation4312Narrow", "4-3-1-2 Narrow"),
        15 => ("Formation541DiamondWB", "5-4-1 Diamond WB"),
        16 => ("Formation4411", "4-4-1-1"),
        17 => ("Formation442SWNotInUse", "4-4-2 SW"),
        18 => ("Formation4231Narrow", "4-2-3-1 Narrow"),
        19 => ("Formation41311DMNarrow", "4-1-3-1-1 DM Narrow"),
        20 => ("Formation4132DMNarrow", "4-1-3-2 DM Narrow"),
        21 => ("Formation4231Wide", "4-2-3-1 Wide"),
        22 => ("Formation4222DM", "4-2-2-2 DM"),
        23 => ("Formation4332DMWide", "4-3-3-2 DM Wide"),
        24 => ("Formation4222DMNarrow", "4-2-2-2 DM Narrow"),
        25 => ("Formation31312DM", "3-1-3-1-2 DM"),
        26 => ("Formation433DMNarrow", "4-1-2-3 DM Narrow"),
        27 => ("Formation4321DMNarrow", "4-1-2-2-1 DM Narrow"),
        28 => ("Formation424DMWide", "4-2-4 DM Wide"),
        29 => ("Formation4231DMWide", "4-2-3-1 DM Wide"),
        30 => ("Formation5122DMWB", "5-1-2-2 DM WB"),
        31 => ("Formation3421DM", "3-4-2-1 DM"),
        32 => ("Formation4141DM", "4-1-4-1 DM"),
        33 => ("Formation44112DM", "4-4-1-1-2 DM"),
        34 => ("Formation3232DM", "3-2-3-2 DM"),
        35 => ("Formation343DMWide", "3-4-3 DM Wide"),
        36 => ("Formation433Narrow", "4-3-3 Narrow"),
        37 => ("Formation424", "4-2-4"),
        38 => ("Formation541", "5-4-1"),
        39 => ("Formation4123Narrow", "4-1-2-3 Narrow"),
        40 => ("Formation4231DMNarrow", "4-2-3-1 DM Narrow"),
        41 => ("Formation5212WB", "5-2-1-2 WB"),
        42 => ("Formation5122DMNarrow", "5-1-2-2 DM Narrow"),
        43 => ("Formation235", "2-3-5"),
        44 => ("Formation244", "2-4-4"),
        45 => ("Formation325", "3-2-5"),
        46 => ("Formation334", "3-3-4"),
        47 => ("Formation523Narrow", "5-2-3 Narrow"),
        48 => ("Formation5212SWWBNotInUse", "5-2-1-2 SW WB"),
        49 => ("Formation532SWDMWBNotInUse", "5-3-2 SW DM WB"),
        50 => ("Formation4141DMAsymmetricAMR", "4-1-4-1 DM Asymmetric AMR"),
        51 => ("Formation4240DMWide", "4-2-4-0 DM Wide"),
        52 => ("Formation4132DM", "4-1-3-2 DM"),
        53 => ("Formation4141DMAsymmetricAML", "4-1-4-1 DM Asymmetric AML"),
        54 => ("Formation4231DM", "4-2-3-1 DM"),
        55 => ("Formation42310DM", "4-2-3-1-0 DM"),
        56 => ("Formation5131DMWB", "5-1-3-1 DM WB"),
        57 => ("Formation541DM", "5-4-1 DM"),
        58 => ("Formation5410DM", "5-4-1-0 DM"),
        59 => ("Formation5221WB", "5-2-2-1 WB"),
        60 => ("Formation3142DM", "3-1-4-2 DM"),
        61 => ("Formation523WBWide", "5-2-3 WB Wide"),
        62 => ("Formation4222AMNarrow", "4-2-2-2 AM Narrow"),
        63 => ("Formation343DMNarrow", "3-4-3 DM Narrow"),
        64 => ("Formation5212DMWB", "5-2-1-2 DM WB"),
        65 => ("Formation3241", "3-2-4-1"),
        66 => ("Formation5212DMFBNotInUse", "5-2-1-2 DM FB"),
        67 => ("Formation5221DMFBNotInUse", "5-2-2-1 DM FB"),
        68 => ("Formation523DMFBNotInUse", "5-2-3 DM FB"),
        69 => ("Formation532FBNotInUse", "5-3-2 FB"),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fm26_metadata_catalogue_maps_current_live_formation_code() {
        let template = tactic_formation_template(4).expect("4-1-2-3 DM Wide");
        assert_eq!(template.enum_name, "Formation433DMWide");
        assert_eq!(template.label, "4-1-2-3 DM Wide");
        assert_eq!(template.positions.len(), 11);
        assert!(template.exact_shape);
    }

    #[test]
    fn fm26_metadata_catalogue_keeps_rare_formations_visible_without_fake_shape() {
        let template = tactic_formation_template(65).expect("3-2-4-1");
        assert_eq!(template.enum_name, "Formation3241");
        assert_eq!(template.positions, GENERIC_XI);
        assert!(!template.exact_shape);
    }
}
