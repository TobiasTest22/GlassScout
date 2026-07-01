use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RawObservation {
    field: String,
    value: Option<serde_json::Value>,
    raw_value: Option<serde_json::Value>,
    visibility_status: String,
    is_hidden_blocked: bool,
    confidence: u8,
    source: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SafeObservation {
    field: String,
    value: Option<serde_json::Value>,
    visibility_status: String,
    confidence: u8,
    source: String,
}

fn is_usable(status: &str) -> bool {
    matches!(
        status,
        "rumoured"
            | "estimated"
            | "ranged"
            | "scout_confirmed"
            | "coach_confirmed"
            | "analyst_confirmed"
            | "fully_visible"
    )
}

#[tauri::command]
pub fn filter_observations(observations: Vec<RawObservation>) -> Vec<SafeObservation> {
    observations
        .into_iter()
        .filter(|item| !item.is_hidden_blocked && is_usable(&item.visibility_status))
        .map(|item| SafeObservation {
            field: item.field,
            value: item.value,
            visibility_status: item.visibility_status,
            confidence: item.confidence,
            source: item.source,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocked_hidden_values_never_leave_the_filter() {
        let result = filter_observations(vec![
            RawObservation {
                field: "passing".into(),
                value: Some(serde_json::json!(15)),
                raw_value: Some(serde_json::json!(18)),
                visibility_status: "scout_confirmed".into(),
                is_hidden_blocked: false,
                confidence: 80,
                source: "scout".into(),
            },
            RawObservation {
                field: "potentialAbility".into(),
                value: Some(serde_json::json!(190)),
                raw_value: Some(serde_json::json!(190)),
                visibility_status: "blocked_hidden".into(),
                is_hidden_blocked: true,
                confidence: 100,
                source: "memory".into(),
            },
        ]);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].field, "passing");
    }
}
