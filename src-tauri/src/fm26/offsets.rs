use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::OnceLock};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MappingCoverage {
    pub(crate) section: String,
    pub(crate) validated: u32,
    pub(crate) candidate: u32,
    pub(crate) unmapped: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EntityMapIndex {
    pub(crate) schema_version: u32,
    pub(crate) profiles: Vec<EntityMapProfile>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EntityMapProfile {
    pub(crate) id: String,
    pub(crate) build_fingerprint: BuildFingerprint,
    pub(crate) file_version: String,
    pub(crate) product_version: String,
    pub(crate) executable_sha256: String,
    pub(crate) architecture: String,
    pub(crate) module: String,
    pub(crate) signatures: Vec<EntitySignature>,
    pub(crate) pointer_chains: Vec<PointerChain>,
    pub(crate) constants: MapConstants,
    pub(crate) sections: HashMap<String, HashMap<String, FieldDefinition>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BuildFingerprint {
    pub(crate) executable_sha256: String,
    pub(crate) file_version: String,
    pub(crate) product_version: String,
    pub(crate) architecture: String,
    pub(crate) module: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FieldDefinition {
    pub(crate) offset: Option<u64>,
    pub(crate) source: String,
    pub(crate) value_type: String,
    pub(crate) transform: Option<String>,
    pub(crate) status: String,
    pub(crate) confidence: f64,
    pub(crate) validations: Vec<String>,
}

#[derive(Deserialize)]
pub(crate) struct EntitySignature {
    pub(crate) name: String,
    pub(crate) pattern: String,
}

#[derive(Deserialize)]
pub(crate) struct PointerChain {
    pub(crate) name: String,
    pub(crate) root: String,
    pub(crate) offsets: Vec<u64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MapConstants {
    pub(crate) manager_registry_vector_offset: u64,
    pub(crate) human_person_offset: u64,
    pub(crate) person_first_name_offset: u64,
    pub(crate) person_second_name_offset: u64,
    pub(crate) person_common_name_offset: u64,
    pub(crate) person_contract_offset: u64,
    pub(crate) contract_team_offset: u64,
    pub(crate) team_vtable_rva: u64,
    pub(crate) team_club_offset: u64,
    pub(crate) team_players_start_offset: u64,
    pub(crate) team_players_end_offset: u64,
    pub(crate) club_vtable_rva: u64,
    pub(crate) club_name_offset: u64,
    pub(crate) entity_uid_offset: u64,
    pub(crate) player_person_offset: u64,
    pub(crate) player_positions_offset: u64,
    pub(crate) player_attributes_offset: u64,
    pub(crate) player_current_date_offset: u64,
    pub(crate) person_nationality_offset: u64,
    pub(crate) nation_name_offset: u64,
    pub(crate) person_birth_date_offset: u64,
    pub(crate) tactics_manager_vtable_rva: u64,
}

pub(crate) fn embedded_entity_map_index() -> &'static EntityMapIndex {
    static INDEX: OnceLock<EntityMapIndex> = OnceLock::new();
    INDEX.get_or_init(|| {
        serde_json::from_str(include_str!("../../entity-maps/index.json"))
            .expect("embedded entity-map index must be valid JSON")
    })
}

pub(crate) fn find_entity_map(
    file_version: Option<&str>,
    product_version: Option<&str>,
    executable_sha256: Option<&str>,
    architecture: Option<&str>,
) -> Option<&'static EntityMapProfile> {
    let index = embedded_entity_map_index();
    if index.schema_version != 2 {
        return None;
    }
    index.profiles.iter().find(|profile| {
        let declared_shape_is_valid = !profile.module.is_empty()
            && profile.build_fingerprint.executable_sha256 == profile.executable_sha256
            && profile.build_fingerprint.file_version == profile.file_version
            && profile.build_fingerprint.product_version == profile.product_version
            && profile.build_fingerprint.architecture == profile.architecture
            && profile.build_fingerprint.module == profile.module
            && profile
                .signatures
                .iter()
                .all(|signature| !signature.name.is_empty() && !signature.pattern.is_empty())
            && profile.pointer_chains.iter().all(|chain| {
                !chain.name.is_empty() && !chain.root.is_empty() && !chain.offsets.is_empty()
            });
        declared_shape_is_valid
            && file_version == Some(profile.file_version.as_str())
            && product_version == Some(profile.product_version.as_str())
            && executable_sha256 == Some(profile.executable_sha256.as_str())
            && architecture == Some(profile.architecture.as_str())
    })
}

pub(crate) fn mapping_coverage(profile: &EntityMapProfile) -> Vec<MappingCoverage> {
    let mut coverage: Vec<MappingCoverage> = profile
        .sections
        .iter()
        .map(|(section, fields)| {
            let mut item = MappingCoverage {
                section: section.clone(),
                validated: 0,
                candidate: 0,
                unmapped: 0,
            };
            for field in fields.values() {
                let structurally_valid = !field.source.is_empty()
                    && !field.value_type.is_empty()
                    && field.confidence.is_finite()
                    && (0.0..=1.0).contains(&field.confidence)
                    && (field.status != "validated" || field_is_publishable(field));
                let _transform_is_declared = field.transform.as_deref().unwrap_or("identity");
                if !structurally_valid {
                    item.unmapped += 1;
                } else {
                    match field.status.as_str() {
                        "validated" => item.validated += 1,
                        "candidate" => item.candidate += 1,
                        _ => item.unmapped += 1,
                    }
                }
            }
            item
        })
        .collect();
    coverage.sort_by(|left, right| left.section.cmp(&right.section));
    coverage
}

pub(crate) fn field_is_publishable(field: &FieldDefinition) -> bool {
    field.status == "validated"
        && field.offset.is_some()
        && field.confidence >= 0.95
        && !field.validations.is_empty()
}
