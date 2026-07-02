use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

use rusqlite::{params, Connection, OpenFlags, OptionalExtension, Row};
use serde_json::{json, Value};

use crate::fm26::{
    roles::evaluate_player_roles,
    structs::{PLAYER_ATTRIBUTE_NAMES, POSITION_NAMES},
};

#[derive(Clone, Debug)]
pub(crate) struct DossierReference {
    pub(crate) player_count: u32,
    pub(crate) path: PathBuf,
}

#[derive(Clone, Debug)]
struct DossierPlayer {
    uid: String,
    name: String,
    age: Option<u8>,
    dob_year: Option<u16>,
    dob_doy: Option<u16>,
    foot_left: Option<i64>,
    foot_right: Option<i64>,
    height_cm: Option<i64>,
    nationality: Option<String>,
    club_id: Option<String>,
    club_name: Option<String>,
    positions: Vec<String>,
    position_bytes: Vec<u8>,
    attributes: HashMap<String, u8>,
    scout_knowledge: &'static str,
    scout_confidence: u8,
    is_ours: bool,
    best_position: Option<String>,
    contract_type: Option<String>,
    contract_expiry_packed: Option<i64>,
    contract_start_packed: Option<i64>,
    sign_date_packed: Option<i64>,
    contract_remaining_text: Option<String>,
    wage: Option<i64>,
    value: Option<i64>,
    condition: Option<i64>,
    match_sharpness: Option<i64>,
    fatigue: Option<i64>,
    injury_type: Option<String>,
    is_injured: Option<i64>,
    squad_number: Option<i64>,
    personality: Option<String>,
    traits_json: Option<String>,
    form_json: Option<String>,
    apps: Option<i64>,
    minutes: Option<i64>,
    goals: Option<i64>,
    assists: Option<i64>,
    avg_rating: Option<f64>,
    xg: Option<f64>,
    xa: Option<f64>,
    shots: Option<i64>,
    key_passes: Option<i64>,
    chances_created: Option<i64>,
    passes_att: Option<i64>,
    passes_comp: Option<i64>,
    tackles_att: Option<i64>,
    tackles_won: Option<i64>,
    interceptions: Option<i64>,
    headers_att: Option<i64>,
    headers_won: Option<i64>,
    crosses_att: Option<i64>,
    crosses_comp: Option<i64>,
    dribbles: Option<i64>,
    progressive_passes: Option<i64>,
    clean_sheets: Option<i64>,
    goals_conceded: Option<i64>,
    career_apps: Option<i64>,
    career_goals: Option<i64>,
    career_league_apps: Option<i64>,
    career_league_goals: Option<i64>,
    transfer_listed: Option<i64>,
    loan_listed: Option<i64>,
    not_for_sale: Option<i64>,
    top_ip_pct: Option<f64>,
    top_oop_pct: Option<f64>,
    top_ip_proj_pct: Option<f64>,
    top_oop_proj_pct: Option<f64>,
    eff_score: Option<i64>,
}

pub(crate) fn augment_live_players(players: &mut [Value]) -> Option<DossierReference> {
    let path = latest_save_database()?;
    let connection = open_readonly(&path).ok()?;
    let player_count = connection
        .query_row("select count(*) from player", [], |row| {
            row.get::<_, i64>(0)
        })
        .ok()
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or_default();
    for player in players.iter_mut() {
        let Some(uid) = player.get("id").and_then(Value::as_str) else {
            continue;
        };
        let Some(dossier) = read_player(&connection, uid).ok().flatten() else {
            continue;
        };
        merge_player(player, &dossier);
    }
    Some(DossierReference { player_count, path })
}

pub(crate) fn search_players(query: &str, limit: usize) -> Vec<Value> {
    let Some(path) = latest_save_database() else {
        return Vec::new();
    };
    let Ok(connection) = open_readonly(&path) else {
        return Vec::new();
    };
    let normalized = query.trim();
    let sql = if normalized.is_empty() {
        "
        select p.uid
        from player p
        left join player_managed pm on pm.uid = p.uid
        order by p.is_ours desc, coalesce(p.top_ip_pct, 0) desc, coalesce(p.common_name, p.native_full, '') asc
        limit ?1
        "
    } else {
        "
        select p.uid
        from player p
        left join club c on c.id = p.native_club_id
        left join nation n on n.id = p.native_nation_id
        where coalesce(p.common_name, '') like ?1
           or coalesce(p.native_full, '') like ?1
           or coalesce(c.name, '') like ?1
           or coalesce(n.name, '') like ?1
        order by p.is_ours desc, coalesce(p.knowledge_level, 0) desc, coalesce(p.top_ip_pct, 0) desc, coalesce(p.common_name, p.native_full, '') asc
        limit ?2
        "
    };
    let mut statement = match connection.prepare(sql) {
        Ok(statement) => statement,
        Err(_) => return Vec::new(),
    };
    let uids = if normalized.is_empty() {
        statement
            .query_map([limit as i64], |row| row.get::<_, i64>(0))
            .map(|rows| rows.filter_map(Result::ok).collect::<Vec<_>>())
            .unwrap_or_default()
    } else {
        let like = format!("%{normalized}%");
        statement
            .query_map(params![like, limit as i64], |row| row.get::<_, i64>(0))
            .map(|rows| rows.filter_map(Result::ok).collect::<Vec<_>>())
            .unwrap_or_default()
    };

    uids.into_iter()
        .filter_map(|uid| read_player(&connection, &uid.to_string()).ok().flatten())
        .map(|player| indexed_player_json(&player))
        .collect()
}

pub(crate) fn player_profile(player_id: &str) -> Option<Value> {
    let path = latest_save_database()?;
    let connection = open_readonly(&path).ok()?;
    read_player(&connection, player_id)
        .ok()
        .flatten()
        .map(|player| player_json(&player))
}

pub(crate) fn players_by_ids(player_ids: &[String]) -> Vec<Value> {
    let Some(path) = latest_save_database() else {
        return Vec::new();
    };
    let Ok(connection) = open_readonly(&path) else {
        return Vec::new();
    };
    player_ids
        .iter()
        .filter_map(|id| read_player(&connection, id).ok().flatten())
        .map(|player| indexed_player_json(&player))
        .collect()
}

fn latest_save_database() -> Option<PathBuf> {
    let appdata = env::var_os("APPDATA")?;
    let saves = PathBuf::from(appdata)
        .join("es.solbizz.fmdossier")
        .join("saves");
    let mut candidates = fs::read_dir(saves)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("sqlite"))
        .filter_map(|path| {
            let modified = fs::metadata(&path)
                .and_then(|metadata| metadata.modified())
                .ok()?;
            Some((modified, path))
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| right.0.cmp(&left.0));
    candidates.into_iter().map(|(_, path)| path).next()
}

fn open_readonly(path: &Path) -> rusqlite::Result<Connection> {
    Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
}

fn read_player(connection: &Connection, uid: &str) -> rusqlite::Result<Option<DossierPlayer>> {
    let uid_i64 = match uid.parse::<i64>() {
        Ok(uid) => uid,
        Err(_) => return Ok(None),
    };
    let mut statement = connection.prepare(
        "
        select
          p.uid,
          coalesce(nullif(p.common_name, ''), nullif(p.native_full, ''), trim(coalesce(p.native_first, '') || ' ' || coalesce(p.native_surname, ''))) as name,
          p.age, p.dob_year, p.dob_doy, p.foot_left, p.foot_right, p.height_cm,
          p.native_club_id,
          c.name as club_name,
          n.name as nation_name,
          p.knowledge_level,
          p.is_ours,
          p.contract_type,
          p.transfer_listed,
          p.loan_listed,
          p.not_for_sale,
          p.value_native,
          p.wage_native,
          p.top_ip_pct,
          p.top_oop_pct,
          p.top_ip_proj_pct,
          p.top_oop_proj_pct,
          p.eff_score,
          pm.contract_expiry_packed,
          pm.contract_start_packed,
          pm.sign_date_packed,
          pm.contract_remaining_text,
          pm.wage_managed,
          pm.best_position_short,
          pm.condition,
          pm.is_injured,
          pm.squad_number,
          pm.personality,
          pm.traits_json,
          pm.match_sharpness,
          pm.fatigue,
          pm.injury_type,
          coalesce(ps.apps, pm.overall_apps) as apps,
          coalesce(ps.minutes, pm.overall_minutes) as minutes,
          coalesce(ps.goals, pm.overall_goals) as goals,
          coalesce(ps.assists, pm.overall_assists) as assists,
          coalesce(ps.avg_rating, pm.overall_avg_rating) as avg_rating,
          coalesce(ps.xg, pm.xg) as xg,
          coalesce(ps.xa, pm.xa) as xa,
          coalesce(ps.form_json, pm.form_json) as form_json,
          ps.shots,
          coalesce(ps.key_passes, pm.overall_key_passes) as key_passes,
          coalesce(ps.chances_created, pm.overall_chances_created) as chances_created,
          ps.passes_att,
          ps.passes_comp,
          ps.tackles_att,
          ps.tackles_won,
          ps.interceptions,
          ps.headers_att,
          ps.headers_won,
          ps.crosses_att,
          ps.crosses_comp,
          ps.dribbles,
          ps.progressive_passes,
          ps.clean_sheets,
          ps.goals_conceded,
          ps.career_apps,
          ps.career_goals,
          ps.career_league_apps,
          ps.career_league_goals
        from player p
        left join player_managed pm on pm.uid = p.uid
        left join player_stats ps on ps.uid = p.uid
        left join club c on c.id = p.native_club_id
        left join nation n on n.id = p.native_nation_id
        where p.uid = ?1
        ",
    )?;
    let mut player = statement
        .query_row([uid_i64], |row| row_to_player(row))
        .optional()?;
    if let Some(player) = player.as_mut() {
        player.positions = read_positions(connection, uid_i64)?;
        player.position_bytes = position_bytes(&player.positions);
        if player.is_ours || player.scout_confidence > 0 {
            player.attributes = read_attributes(connection, uid_i64)?;
        }
    }
    Ok(player)
}

fn row_to_player(row: &Row<'_>) -> rusqlite::Result<DossierPlayer> {
    let uid: i64 = row.get("uid")?;
    let knowledge_level = row
        .get::<_, Option<i64>>("knowledge_level")?
        .unwrap_or_default();
    let is_ours = row.get::<_, Option<i64>>("is_ours")?.unwrap_or_default() == 1;
    let (scout_knowledge, scout_confidence) = if is_ours {
        ("fully_known", 100)
    } else if knowledge_level >= 5 {
        ("fully_known", 100)
    } else if knowledge_level >= 3 {
        ("partly_known", 65)
    } else if knowledge_level > 0 {
        ("partly_known", 35)
    } else {
        ("unknown", 0)
    };
    Ok(DossierPlayer {
        uid: uid.to_string(),
        name: row
            .get::<_, Option<String>>("name")?
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| format!("FM player {uid}")),
        age: row
            .get::<_, Option<i64>>("age")?
            .and_then(|value| u8::try_from(value).ok()),
        dob_year: row
            .get::<_, Option<i64>>("dob_year")?
            .and_then(|value| u16::try_from(value).ok()),
        dob_doy: row
            .get::<_, Option<i64>>("dob_doy")?
            .and_then(|value| u16::try_from(value).ok()),
        foot_left: row.get("foot_left")?,
        foot_right: row.get("foot_right")?,
        height_cm: row.get("height_cm")?,
        nationality: row.get("nation_name")?,
        club_id: row
            .get::<_, Option<i64>>("native_club_id")?
            .map(|value| value.to_string()),
        club_name: row.get("club_name")?,
        positions: Vec::new(),
        position_bytes: vec![0; POSITION_NAMES.len()],
        attributes: HashMap::new(),
        scout_knowledge,
        scout_confidence,
        is_ours,
        best_position: row.get("best_position_short")?,
        contract_type: row.get("contract_type")?,
        contract_expiry_packed: row.get("contract_expiry_packed")?,
        contract_start_packed: row.get("contract_start_packed")?,
        sign_date_packed: row.get("sign_date_packed")?,
        contract_remaining_text: row.get("contract_remaining_text")?,
        wage: row
            .get::<_, Option<i64>>("wage_managed")?
            .or(row.get("wage_native")?),
        value: row.get("value_native")?,
        condition: row.get("condition")?,
        match_sharpness: row.get("match_sharpness")?,
        fatigue: row.get("fatigue")?,
        injury_type: row.get("injury_type")?,
        is_injured: row.get("is_injured")?,
        squad_number: row.get("squad_number")?,
        personality: row.get("personality")?,
        traits_json: row.get("traits_json")?,
        form_json: row.get("form_json")?,
        apps: row.get("apps")?,
        minutes: row.get("minutes")?,
        goals: row.get("goals")?,
        assists: row.get("assists")?,
        avg_rating: row.get("avg_rating")?,
        xg: row.get("xg")?,
        xa: row.get("xa")?,
        shots: row.get("shots")?,
        key_passes: row.get("key_passes")?,
        chances_created: row.get("chances_created")?,
        passes_att: row.get("passes_att")?,
        passes_comp: row.get("passes_comp")?,
        tackles_att: row.get("tackles_att")?,
        tackles_won: row.get("tackles_won")?,
        interceptions: row.get("interceptions")?,
        headers_att: row.get("headers_att")?,
        headers_won: row.get("headers_won")?,
        crosses_att: row.get("crosses_att")?,
        crosses_comp: row.get("crosses_comp")?,
        dribbles: row.get("dribbles")?,
        progressive_passes: row.get("progressive_passes")?,
        clean_sheets: row.get("clean_sheets")?,
        goals_conceded: row.get("goals_conceded")?,
        career_apps: row.get("career_apps")?,
        career_goals: row.get("career_goals")?,
        career_league_apps: row.get("career_league_apps")?,
        career_league_goals: row.get("career_league_goals")?,
        transfer_listed: row.get("transfer_listed")?,
        loan_listed: row.get("loan_listed")?,
        not_for_sale: row.get("not_for_sale")?,
        top_ip_pct: row.get("top_ip_pct")?,
        top_oop_pct: row.get("top_oop_pct")?,
        top_ip_proj_pct: row.get("top_ip_proj_pct")?,
        top_oop_proj_pct: row.get("top_oop_proj_pct")?,
        eff_score: row.get("eff_score")?,
    })
}

fn read_positions(connection: &Connection, uid: i64) -> rusqlite::Result<Vec<String>> {
    let mut statement = connection.prepare(
        "select slot, familiarity from player_position where uid = ?1 order by familiarity desc, slot asc",
    )?;
    let mut positions = Vec::new();
    let rows = statement.query_map([uid], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    })?;
    for row in rows.filter_map(Result::ok) {
        if row.1 >= 15 {
            positions.push(row.0);
        }
    }
    if positions.is_empty() {
        let mut fallback = connection.prepare(
            "select slot from player_position where uid = ?1 order by familiarity desc, slot asc limit 1",
        )?;
        if let Some(slot) = fallback
            .query_row([uid], |row| row.get::<_, String>(0))
            .optional()?
        {
            positions.push(slot);
        }
    }
    Ok(positions)
}

fn position_bytes(positions: &[String]) -> Vec<u8> {
    let mut bytes = vec![1_u8; POSITION_NAMES.len()];
    for position in positions {
        if let Some(index) = POSITION_NAMES
            .iter()
            .position(|name| *name == position.as_str())
        {
            bytes[index] = 20;
        }
    }
    bytes
}

fn read_attributes(connection: &Connection, uid: i64) -> rusqlite::Result<HashMap<String, u8>> {
    let mut statement =
        connection.prepare("select attr_key, true_value from player_attribute where uid = ?1")?;
    let rows = statement.query_map([uid], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    })?;
    Ok(rows
        .filter_map(Result::ok)
        .filter_map(|(key, value)| {
            let name = normalize_attribute_key(&key)?;
            let value = u8::try_from(value).ok()?.clamp(1, 20);
            if hidden_attribute_name(name) {
                return None;
            }
            Some((name.to_string(), value))
        })
        .collect())
}

fn normalize_attribute_key(key: &str) -> Option<&'static str> {
    Some(match key {
        "AerialReach" => "Aerial Reach",
        "CommandOfArea" => "Command of Area",
        "FirstTouch" => "First Touch",
        "FreeKicks" => "Free Kick Taking",
        "InjuryProneness" => "Injury Proneness",
        "JumpingReach" => "Jumping Reach",
        "LongShots" => "Long Shots",
        "LongThrows" => "Long Throws",
        "NaturalFitness" => "Natural Fitness",
        "OffTheBall" => "Off the Ball",
        "OneOnOnes" => "One on Ones",
        "PenaltyTaking" => "Penalty Taking",
        "RushingOut" => "Rushing Out",
        "TendencyToPunch" => "Punching",
        "WorkRate" => "Work Rate",
        "LeftFoot" | "RightFoot" => return None,
        other => PLAYER_ATTRIBUTE_NAMES
            .iter()
            .find(|name| name.replace(' ', "") == other || **name == other)
            .copied()?,
    })
}

fn hidden_attribute_name(name: &str) -> bool {
    matches!(
        name,
        "Dirtiness" | "Consistency" | "Important Matches" | "Injury Proneness" | "Versatility"
    )
}

fn indexed_player_json(player: &DossierPlayer) -> Value {
    let role = role_summary(player);
    json!({
        "id": player.uid,
        "name": player.name,
        "age": player.age,
        "nationality": player.nationality,
        "clubId": player.club_id,
        "clubName": player.club_name,
        "positions": player.positions,
        "managedSquad": player.is_ours,
        "visibility": if player.scout_confidence > 0 { "known" } else { "unknown" },
        "scoutKnowledge": player.scout_knowledge,
        "scoutConfidence": player.scout_confidence,
        "bestRole": role.0,
        "roleFit": role.1,
        "value": money_label(player.value, false),
        "wage": money_label(player.wage, true),
        "contractStatus": contract_label(player),
        "contractRemaining": player.contract_remaining_text,
        "averageRating": player.avg_rating,
        "minutesPlayed": player.minutes,
        "goals": player.goals,
        "assists": player.assists,
        "transferInterest": Option::<String>::None,
        "loanInterest": Option::<String>::None,
        "transferAvailable": player.transfer_listed.map(|value| value == 1),
        "loanAvailable": player.loan_listed.map(|value| value == 1),
        "notForSale": player.not_for_sale.map(|value| value == 1),
        "per90": per90(player),
        "rawStats": raw_stats(player),
        "inPossessionFit": player.top_ip_pct.map(round_percent),
        "outOfPossessionFit": player.top_oop_pct.map(round_percent),
        "projectedInPossessionFit": player.top_ip_proj_pct.map(round_percent),
        "projectedOutOfPossessionFit": player.top_oop_proj_pct.map(round_percent),
        "efficiencyScore": player.eff_score,
        "marketValueAmount": player.value,
    })
}

fn player_json(player: &DossierPlayer) -> Value {
    let role = role_summary(player);
    let playable = if player.attributes.is_empty() {
        Vec::new()
    } else {
        evaluate_player_roles(&player.position_bytes, &player.attributes).playable
    };
    let other_roles = if player.attributes.is_empty() {
        Vec::new()
    } else {
        evaluate_player_roles(&player.position_bytes, &player.attributes).secondary
    };
    let role_reasoning = if player.attributes.is_empty() {
        vec![
            "Profile is available from the FM Dossier save index, but attributes are hidden because scouting knowledge is insufficient.".to_string(),
        ]
    } else {
        evaluate_player_roles(&player.position_bytes, &player.attributes).reasoning
    };
    let condition = condition_label(player);
    json!({
        "id": player.uid,
        "name": player.name,
        "age": player.age,
        "dateOfBirth": fm_date_label(player.dob_year, player.dob_doy),
        "nationality": player.nationality,
        "secondNationality": null,
        "positions": player.positions,
        "bestRole": role.0,
        "currentAbility": null,
        "potentialAbility": null,
        "abilityScore": role.1,
        "form": form_label(player.avg_rating),
        "averageRating": player.avg_rating,
        "minutesPlayed": player.minutes,
        "goals": player.goals,
        "assists": player.assists,
        "contractStatus": contract_label(player),
        "value": money_label(player.value, false),
        "wage": money_label(player.wage, true),
        "squadImportance": player.squad_number.map(|number| format!("#{number}")),
        "developmentTrend": null,
        "tacticalFit": null,
        "roleFit": role.1,
        "playableRoles": playable,
        "otherRoles": other_roles,
        "preferredFoot": preferred_foot_label(player),
        "heightCm": player.height_cm,
        "strengths": attribute_extremes(&player.attributes, true),
        "weaknesses": attribute_extremes(&player.attributes, false),
        "clubId": player.club_id,
        "clubName": player.club_name,
        "transferInterest": Option::<String>::None,
        "loanInterest": Option::<String>::None,
        "transferAvailable": player.transfer_listed.map(|value| value == 1),
        "loanAvailable": player.loan_listed.map(|value| value == 1),
        "notForSale": player.not_for_sale.map(|value| value == 1),
        "attributes": player.attributes,
        "per90": per90(player),
        "rawStats": raw_stats(player),
        "careerTotals": career_totals(player),
        "formHistory": player.form_json,
        "traits": player.traits_json,
        "inPossessionFit": player.top_ip_pct.map(round_percent),
        "outOfPossessionFit": player.top_oop_pct.map(round_percent),
        "projectedInPossessionFit": player.top_ip_proj_pct.map(round_percent),
        "projectedOutOfPossessionFit": player.top_oop_proj_pct.map(round_percent),
        "efficiencyScore": player.eff_score,
        "scoutKnowledge": player.scout_knowledge,
        "scoutConfidence": player.scout_confidence,
        "lastScoutedDate": null,
        "reportReliability": if player.scout_confidence >= 60 { "FM Dossier knowledge index" } else { "Unknown" },
        "bestCalculatedPosition": player.best_position,
        "truePrice": null,
        "fairPriceRange": null,
        "valuationLabel": "unavailable",
        "valuationReasoning": ["Value and wage are read from FM Dossier's local save index when available; no hidden CA/PA is used."],
        "retrainingSuggestion": null,
        "roleReasoning": role_reasoning,
        "riskLevel": if player.is_injured == Some(1) { "high" } else { "unknown" },
        "marketValueAmount": player.value,
        "personality": player.personality,
        "condition": condition,
        "matchSharpness": player.match_sharpness,
        "fatigue": player.fatigue,
        "contractStartDate": player.contract_start_packed.and_then(unpack_fm_dossier_date),
        "signDate": player.sign_date_packed.and_then(unpack_fm_dossier_date),
        "contractRemaining": player.contract_remaining_text,
        "recommendation": {
            "minimum": role.1,
            "maximum": role.1,
            "completeness": player.scout_confidence,
            "label": if player.scout_confidence >= 60 { "FM Dossier local evidence" } else { "identity and public database evidence only" }
        },
        "knowledge": {}
    })
}

fn merge_player(player: &mut Value, dossier: &DossierPlayer) {
    let Some(object) = player.as_object_mut() else {
        return;
    };
    let role = role_summary(dossier);
    if object.get("age").is_none_or(Value::is_null) {
        object.insert("age".to_string(), json!(dossier.age));
    }
    if object.get("dateOfBirth").is_none_or(Value::is_null) {
        object.insert(
            "dateOfBirth".to_string(),
            json!(fm_date_label(dossier.dob_year, dossier.dob_doy)),
        );
    }
    if object.get("nationality").is_none_or(Value::is_null) {
        object.insert("nationality".to_string(), json!(dossier.nationality));
    }
    if object
        .get("positions")
        .and_then(Value::as_array)
        .is_none_or(Vec::is_empty)
    {
        object.insert("positions".to_string(), json!(dossier.positions));
    }
    if object
        .get("attributes")
        .and_then(Value::as_object)
        .is_none_or(serde_json::Map::is_empty)
        && !dossier.attributes.is_empty()
    {
        object.insert("attributes".to_string(), json!(dossier.attributes));
    }
    if object.get("scoutKnowledge").and_then(Value::as_str) != Some("fully_known") {
        object.insert("scoutKnowledge".to_string(), json!(dossier.scout_knowledge));
    }
    let existing_confidence = object
        .get("scoutConfidence")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    if existing_confidence < u64::from(dossier.scout_confidence) {
        object.insert(
            "scoutConfidence".to_string(),
            json!(dossier.scout_confidence),
        );
    }
    if object.get("contractStatus").is_none_or(Value::is_null) {
        object.insert("contractStatus".to_string(), json!(contract_label(dossier)));
    }
    if object.get("wage").is_none_or(Value::is_null) {
        object.insert("wage".to_string(), json!(money_label(dossier.wage, true)));
    }
    if object.get("value").is_none_or(Value::is_null) {
        object.insert(
            "value".to_string(),
            json!(money_label(dossier.value, false)),
        );
    }
    object.insert("condition".to_string(), json!(condition_label(dossier)));
    object.insert("averageRating".to_string(), json!(dossier.avg_rating));
    object.insert("minutesPlayed".to_string(), json!(dossier.minutes));
    object.insert("goals".to_string(), json!(dossier.goals));
    object.insert("assists".to_string(), json!(dossier.assists));
    object.insert("form".to_string(), json!(form_label(dossier.avg_rating)));
    object.insert(
        "preferredFoot".to_string(),
        json!(preferred_foot_label(dossier)),
    );
    object.insert("heightCm".to_string(), json!(dossier.height_cm));
    object.insert("matchSharpness".to_string(), json!(dossier.match_sharpness));
    object.insert("fatigue".to_string(), json!(dossier.fatigue));
    object.insert(
        "squadImportance".to_string(),
        json!(dossier.squad_number.map(|number| format!("#{number}"))),
    );
    object.insert(
        "transferAvailable".to_string(),
        json!(dossier.transfer_listed.map(|value| value == 1)),
    );
    object.insert(
        "loanAvailable".to_string(),
        json!(dossier.loan_listed.map(|value| value == 1)),
    );
    object.insert(
        "notForSale".to_string(),
        json!(dossier.not_for_sale.map(|value| value == 1)),
    );
    object.insert(
        "transferInterest".to_string(),
        json!(Option::<String>::None),
    );
    object.insert("loanInterest".to_string(), json!(Option::<String>::None));
    object.insert("clubName".to_string(), json!(dossier.club_name));
    object.insert(
        "dossierReference".to_string(),
        json!("fm-dossier-local-save-index"),
    );
    if !dossier.attributes.is_empty() {
        let evaluation = evaluate_player_roles(&dossier.position_bytes, &dossier.attributes);
        if object
            .get("playableRoles")
            .and_then(Value::as_array)
            .is_none_or(Vec::is_empty)
        {
            object.insert("playableRoles".to_string(), json!(evaluation.playable));
        }
        if object
            .get("otherRoles")
            .and_then(Value::as_array)
            .is_none_or(Vec::is_empty)
        {
            object.insert("otherRoles".to_string(), json!(evaluation.secondary));
        }
        if object
            .get("strengths")
            .and_then(Value::as_array)
            .is_none_or(Vec::is_empty)
        {
            object.insert(
                "strengths".to_string(),
                json!(attribute_extremes(&dossier.attributes, true)),
            );
        }
        if object
            .get("weaknesses")
            .and_then(Value::as_array)
            .is_none_or(Vec::is_empty)
        {
            object.insert(
                "weaknesses".to_string(),
                json!(attribute_extremes(&dossier.attributes, false)),
            );
        }
    }
    if object.get("roleFit").and_then(Value::as_u64).is_none() {
        object.insert("roleFit".to_string(), json!(role.1));
    }
    if role.0.is_some() || object.get("bestRole").is_none_or(Value::is_null) {
        object.insert("bestRole".to_string(), json!(role.0));
    }
    object.insert(
        "inPossessionFit".to_string(),
        json!(dossier.top_ip_pct.map(round_percent)),
    );
    object.insert(
        "outOfPossessionFit".to_string(),
        json!(dossier.top_oop_pct.map(round_percent)),
    );
    object.insert(
        "projectedInPossessionFit".to_string(),
        json!(dossier.top_ip_proj_pct.map(round_percent)),
    );
    object.insert(
        "projectedOutOfPossessionFit".to_string(),
        json!(dossier.top_oop_proj_pct.map(round_percent)),
    );
    object.insert("efficiencyScore".to_string(), json!(dossier.eff_score));
    object.insert("per90".to_string(), per90(dossier));
    object.insert("rawStats".to_string(), raw_stats(dossier));
    object.insert("careerTotals".to_string(), career_totals(dossier));
    object.insert("formHistory".to_string(), json!(dossier.form_json));
    object.insert("traits".to_string(), json!(dossier.traits_json));
    object.insert(
        "contractStartDate".to_string(),
        json!(dossier
            .contract_start_packed
            .and_then(unpack_fm_dossier_date)),
    );
    object.insert(
        "signDate".to_string(),
        json!(dossier.sign_date_packed.and_then(unpack_fm_dossier_date)),
    );
    object.insert(
        "contractRemaining".to_string(),
        json!(dossier.contract_remaining_text),
    );
}

fn role_summary(player: &DossierPlayer) -> (Option<String>, Option<u8>) {
    if let Some(role) = player
        .best_position
        .as_ref()
        .map(|position| position_role_label(position).to_string())
    {
        return (Some(role), player.top_ip_pct.map(round_percent));
    }
    if !player.attributes.is_empty() {
        if let Some(best) = evaluate_player_roles(&player.position_bytes, &player.attributes).best {
            return (Some(best.role.to_string()), Some(best.score));
        }
    }
    (None, player.top_ip_pct.map(round_percent))
}

fn position_role_label(position: &str) -> &'static str {
    match position {
        "GK" => "Goalkeeper",
        "DC" => "Central Defender",
        "DL" | "DR" | "WBL" | "WBR" => "Full-Back",
        "DM" => "Defensive Midfielder",
        "MC" => "Central Midfielder",
        "ML" | "MR" | "AML" | "AMR" => "Winger",
        "AMC" => "Attacking Midfielder",
        "ST" => "Centre Forward",
        _ => "Natural Position",
    }
}

fn round_percent(value: f64) -> u8 {
    value.round().clamp(0.0, 100.0) as u8
}

fn money_label(value: Option<i64>, weekly: bool) -> Option<String> {
    let value = value.filter(|value| *value > 0)?;
    let suffix = if weekly { " p/w" } else { "" };
    Some(format!("€{}{suffix}", grouped_number(value)))
}

fn grouped_number(value: i64) -> String {
    let text = value.abs().to_string();
    let mut output = String::new();
    for (index, ch) in text.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            output.push(',');
        }
        output.push(ch);
    }
    let mut output = output.chars().rev().collect::<String>();
    if value < 0 {
        output.insert(0, '-');
    }
    output
}

fn contract_label(player: &DossierPlayer) -> Option<String> {
    let mut parts = Vec::new();
    if let Some(contract_type) = player
        .contract_type
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        parts.push(contract_type.clone());
    }
    if let Some(date) = player
        .contract_expiry_packed
        .and_then(unpack_fm_dossier_date)
    {
        parts.push(format!("expires {date}"));
    }
    (!parts.is_empty()).then(|| parts.join(" · "))
}

fn unpack_fm_dossier_date(value: i64) -> Option<String> {
    if value <= 0 {
        return None;
    }
    let year = value / 131_072;
    let day = (value % 131_072) / 256;
    if !(1900..=2100).contains(&year) || !(1..=366).contains(&day) {
        return None;
    }
    fm_date_label(Some(year as u16), Some(day as u16))
}

fn fm_date_label(year: Option<u16>, day_of_year: Option<u16>) -> Option<String> {
    let year = year?;
    let mut remaining = day_of_year?;
    if remaining == 0 {
        return None;
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let lengths: [u16; 12] = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    for (month, length) in lengths.into_iter().enumerate() {
        if remaining <= length {
            return Some(format!("{year:04}-{:02}-{:02}", month + 1, remaining));
        }
        remaining -= length;
    }
    None
}

fn condition_label(player: &DossierPlayer) -> Option<String> {
    if let Some(injury) = player
        .injury_type
        .as_ref()
        .filter(|value| !value.is_empty())
    {
        return Some(injury.clone());
    }
    if player.is_injured == Some(1) {
        return Some("Injured".to_string());
    }
    player
        .condition
        .map(|condition| format!("{}%", condition.clamp(0, 100)))
}

fn preferred_foot_label(player: &DossierPlayer) -> Option<&'static str> {
    let left = player.foot_left.unwrap_or_default();
    let right = player.foot_right.unwrap_or_default();
    if left <= 0 && right <= 0 {
        return None;
    }
    if (left - right).abs() <= 3 && left >= 10 && right >= 10 {
        Some("Both")
    } else if left > right {
        Some("Left")
    } else {
        Some("Right")
    }
}

fn form_label(avg_rating: Option<f64>) -> Option<String> {
    avg_rating.map(|rating| format!("{rating:.2}"))
}

fn per90(player: &DossierPlayer) -> Value {
    let minutes = player
        .minutes
        .filter(|minutes| *minutes > 0)
        .map(|minutes| minutes as f64);
    let per = |value: Option<f64>| -> Option<f64> {
        Some(((value? / minutes?) * 90.0 * 100.0).round() / 100.0)
    };
    json!({
        "xgPer90": per(player.xg),
        "xaPer90": per(player.xa),
        "goalsPer90": per(player.goals.map(|value| value as f64)),
        "assistsPer90": per(player.assists.map(|value| value as f64)),
        "shotsPer90": per(player.shots.map(|value| value as f64)),
        "keyPassesPer90": per(player.key_passes.map(|value| value as f64)),
        "chancesCreatedPer90": per(player.chances_created.map(|value| value as f64)),
        "passesAttemptedPer90": per(player.passes_att.map(|value| value as f64)),
        "passesCompletedPer90": per(player.passes_comp.map(|value| value as f64)),
        "tacklesAttemptedPer90": per(player.tackles_att.map(|value| value as f64)),
        "tacklesWonPer90": per(player.tackles_won.map(|value| value as f64)),
        "interceptionsPer90": per(player.interceptions.map(|value| value as f64)),
        "headersWonPer90": per(player.headers_won.map(|value| value as f64)),
        "crossesCompletedPer90": per(player.crosses_comp.map(|value| value as f64)),
        "dribblesPer90": per(player.dribbles.map(|value| value as f64)),
        "progressivePassesPer90": per(player.progressive_passes.map(|value| value as f64))
    })
}

fn raw_stats(player: &DossierPlayer) -> Value {
    json!({
        "apps": player.apps,
        "minutes": player.minutes,
        "goals": player.goals,
        "assists": player.assists,
        "xg": player.xg,
        "xa": player.xa,
        "shots": player.shots,
        "keyPasses": player.key_passes,
        "chancesCreated": player.chances_created,
        "passesAttempted": player.passes_att,
        "passesCompleted": player.passes_comp,
        "tacklesAttempted": player.tackles_att,
        "tacklesWon": player.tackles_won,
        "interceptions": player.interceptions,
        "headersAttempted": player.headers_att,
        "headersWon": player.headers_won,
        "crossesAttempted": player.crosses_att,
        "crossesCompleted": player.crosses_comp,
        "dribbles": player.dribbles,
        "progressivePasses": player.progressive_passes,
        "cleanSheets": player.clean_sheets,
        "goalsConceded": player.goals_conceded
    })
}

fn career_totals(player: &DossierPlayer) -> Value {
    json!({
        "careerApps": player.career_apps,
        "careerGoals": player.career_goals,
        "careerLeagueApps": player.career_league_apps,
        "careerLeagueGoals": player.career_league_goals
    })
}

fn attribute_extremes(attributes: &HashMap<String, u8>, strongest: bool) -> Vec<String> {
    let mut values = attributes
        .iter()
        .map(|(name, value)| (name.clone(), *value))
        .collect::<Vec<_>>();
    if strongest {
        values.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    } else {
        values.sort_by(|left, right| left.1.cmp(&right.1).then_with(|| left.0.cmp(&right.0)));
    }
    values
        .into_iter()
        .take(3)
        .map(|(name, value)| format!("{name} {value}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_fm_dossier_bridge_reads_current_save_when_available() {
        if latest_save_database().is_none() {
            return;
        }

        let results = search_players("", 10);
        assert!(
            !results.is_empty(),
            "FM Dossier save index should expose players"
        );

        let first_id = results[0]["id"].as_str().expect("player id");
        let profile = player_profile(first_id).expect("indexed profile");
        assert_eq!(profile["id"].as_str(), Some(first_id));
        assert!(profile["name"]
            .as_str()
            .is_some_and(|name| !name.is_empty()));
        assert!(profile["positions"].as_array().is_some());
    }
}
