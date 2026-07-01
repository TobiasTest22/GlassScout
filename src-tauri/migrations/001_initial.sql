PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS saves (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  manager_club TEXT,
  season TEXT,
  tactical_style TEXT,
  ip_formation TEXT,
  oop_formation TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS players (
  id TEXT NOT NULL,
  save_id TEXT NOT NULL,
  fm_uid TEXT,
  name TEXT NOT NULL,
  date_of_birth TEXT,
  age INTEGER,
  club TEXT,
  nationality TEXT,
  position TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  PRIMARY KEY (id, save_id),
  FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_players_save_uid
  ON players(save_id, fm_uid)
  WHERE fm_uid IS NOT NULL;

CREATE TABLE IF NOT EXISTS observations (
  id TEXT PRIMARY KEY,
  save_id TEXT NOT NULL,
  player_id TEXT NOT NULL,
  field_name TEXT NOT NULL,
  value_json TEXT,
  raw_value_json TEXT,
  source TEXT NOT NULL,
  source_type TEXT NOT NULL,
  confidence INTEGER NOT NULL CHECK(confidence BETWEEN 0 AND 100),
  visibility_status TEXT NOT NULL CHECK(visibility_status IN (
    'unknown', 'rumoured', 'estimated', 'ranged', 'scout_confirmed',
    'coach_confirmed', 'analyst_confirmed', 'fully_visible', 'blocked_hidden'
  )),
  date_seen TEXT NOT NULL,
  revealed_by TEXT,
  scout_id TEXT,
  is_hidden_blocked INTEGER NOT NULL DEFAULT 0 CHECK(is_hidden_blocked IN (0, 1)),
  is_estimated INTEGER NOT NULL DEFAULT 0 CHECK(is_estimated IN (0, 1)),
  is_range INTEGER NOT NULL DEFAULT 0 CHECK(is_range IN (0, 1)),
  is_exact_visible INTEGER NOT NULL DEFAULT 0 CHECK(is_exact_visible IN (0, 1)),
  FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
  FOREIGN KEY (player_id, save_id) REFERENCES players(id, save_id) ON DELETE CASCADE,
  CHECK(
    visibility_status != 'blocked_hidden'
    OR (is_hidden_blocked = 1 AND value_json IS NULL)
  )
);

CREATE INDEX IF NOT EXISTS idx_observations_player_field
  ON observations(save_id, player_id, field_name, date_seen DESC);

CREATE TABLE IF NOT EXISTS role_registry (
  id TEXT NOT NULL,
  registry_version TEXT NOT NULL,
  name TEXT NOT NULL,
  position_group TEXT NOT NULL,
  phase TEXT NOT NULL CHECK(phase IN ('in_possession', 'out_of_possession', 'both')),
  definition_json TEXT NOT NULL,
  confidence_basis TEXT NOT NULL,
  verified_against_build TEXT,
  PRIMARY KEY (id, registry_version)
);

CREATE TABLE IF NOT EXISTS scout_reports (
  id TEXT PRIMARY KEY,
  save_id TEXT NOT NULL,
  player_id TEXT NOT NULL,
  scout_id TEXT,
  report_date TEXT NOT NULL,
  confidence INTEGER NOT NULL CHECK(confidence BETWEEN 0 AND 100),
  recommendation TEXT,
  notes TEXT,
  FOREIGN KEY (player_id, save_id) REFERENCES players(id, save_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS recruitment_history (
  id TEXT PRIMARY KEY,
  save_id TEXT NOT NULL,
  player_id TEXT NOT NULL,
  stage TEXT NOT NULL,
  changed_at TEXT NOT NULL,
  reason TEXT,
  FOREIGN KEY (player_id, save_id) REFERENCES players(id, save_id) ON DELETE CASCADE
);
