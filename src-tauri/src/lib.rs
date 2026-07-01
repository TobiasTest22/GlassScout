mod connector;
mod visibility;

use tauri_plugin_sql::{Migration, MigrationKind};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let migrations = vec![Migration {
        version: 1,
        description: "initial_revealed_data_schema",
        sql: include_str!("../migrations/001_initial.sql"),
        kind: MigrationKind::Up,
    }];

    tauri::Builder::default()
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:glassscout.db", migrations)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            connector::connector_status,
            connector::connector_snapshot,
            visibility::filter_observations
        ])
        .run(tauri::generate_context!())
        .expect("error while running GlassScout FM26");
}
