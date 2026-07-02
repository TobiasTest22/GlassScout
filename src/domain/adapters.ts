export type ConnectionState =
  | "not_checked"
  | "process_not_found"
  | "access_denied"
  | "process_ready"
  | "parser_unverified"
  | "connected";

export type LiveConnectorStatus = {
  processDetected: boolean;
  processId: number | null;
  processPath: string | null;
  saveDetected: boolean | null;
  memoryAccess: "not_checked" | "denied" | "read_only_handle_open";
  parserStatus: "unverified" | "ready" | "error";
  state: ConnectionState;
  playersLoaded: number;
  managedSquadPlayers: number;
  databasePlayersIndexed: number;
  backgroundPlayersIndexed: number;
  visiblePlayersLoaded: number;
  fullyScoutedPlayers: number;
  partialScoutReports: number;
  databaseIndexStatus: "not_run" | "ready" | "partial" | "failed";
  databaseScope: "none" | "managed-squad" | "full-save-index";
  clubsLoaded: number;
  lastSync: string | null;
  bytesRead: number;
  executableHeaderValid: boolean;
  gameBuild?: string | null;
  productVersion?: string | null;
  executableSha256?: string | null;
  architecture?: string | null;
  moduleBase?: string | null;
  entityMapStatus?: "missing" | "matched" | "invalid" | "not_checked";
  entityMapProfileId?: string | null;
  mappingSchemaVersion?: number;
  mappingCoverage?: Array<{ section: string; validated: number; candidate: number; unmapped: number }>;
  pointerValidation?: "not_run" | "passed" | "failed";
  handleAccessFlags?: string;
  entityRoot?: string | null;
  savePointer?: string | null;
  managedClubPointer?: string | null;
  playerCollectionPointer?: string | null;
  liveMemoryTacticRead?: "not_run" | "object_not_found" | "object_detected_unmapped" | "ready";
  tacticManagerPointer?: string | null;
  failureStage?: string | null;
  lastSuccessfulRead?: string | null;
  windowsErrorCode?: number | null;
  canWriteMemory: false;
  message: string;
  warnings: string[];
};

export type LivePlayer = {
  id: string;
  name: string;
  age: number | null;
  dateOfBirth?: string | null;
  nationality: string | null;
  secondNationality?: string | null;
  positions: string[];
  bestRole: string | null;
  playableRoles?: PlayerRoleFit[];
  otherRoles?: PlayerRoleFit[];
  currentAbility: number | null;
  potentialAbility: number | null;
  abilityScore?: number | null;
  form: string | null;
  averageRating: number | null;
  minutesPlayed: number | null;
  goals: number | null;
  assists: number | null;
  contractStatus: string | null;
  value: string | null;
  wage: string | null;
  squadImportance: string | null;
  developmentTrend: "improving" | "stable" | "declining" | null;
  tacticalFit: number | null;
  roleFit: number | null;
  preferredFoot?: string | null;
  strengths: string[];
  weaknesses: string[];
  clubId: string | null;
  clubName?: string | null;
  transferInterest: string | null;
  loanInterest: string | null;
  transferAvailable: boolean | null;
  loanAvailable: boolean | null;
  notForSale?: boolean | null;
  attributes?: Record<string, number | null>;
  per90?: Record<string, number | null>;
  inPossessionFit?: number | null;
  outOfPossessionFit?: number | null;
  projectedInPossessionFit?: number | null;
  projectedOutOfPossessionFit?: number | null;
  matchSharpness?: number | null;
  fatigue?: number | null;
  efficiencyScore?: number | null;
  dossierReference?: string | null;
  scoutKnowledge?: "fully_known" | "partly_known" | "unknown" | "needs_scouting" | "missing_data";
  scoutConfidence?: number | null;
  lastScoutedDate?: string | null;
  reportReliability?: string | null;
  bestCalculatedPosition?: string | null;
  truePrice?: number | null;
  fairPriceRange?: [number, number] | null;
  valuationLabel?: "undervalued" | "fair" | "overpriced" | "unavailable";
  valuationReasoning?: string[];
  retrainingSuggestion?: string | null;
  roleReasoning?: string[];
  riskLevel?: "low" | "medium" | "high" | "unknown";
  marketValueAmount?: number | null;
  personality?: string | null;
  condition?: string | null;
  heightCm?: number | null;
  rawStats?: Record<string, number | null>;
  careerTotals?: Record<string, number | null>;
  formHistory?: string | null;
  traits?: string | null;
  contractStartDate?: string | null;
  signDate?: string | null;
  contractRemaining?: string | null;
  recommendation?: RecommendationEvidence;
  knowledge?: Record<string, KnowledgeField<unknown>>;
};

export type PlayerRoleFit = {
  roleKey: string;
  role: string;
  shortRole: string;
  roleIdMask: string;
  positions: string[];
  score: number;
  positionFit: number;
  attributeFit: number | null;
  evidence: string[];
  phase?: "in-possession" | "out-of-possession" | "combined";
  inPossessionRole?: string | null;
  outOfPossessionRole?: string | null;
  inPossessionFit?: number | null;
  outOfPossessionFit?: number | null;
  redFlags?: string[];
};

export type FieldVisibility = "known" | "estimated" | "range" | "unknown";

export type KnowledgeField<T> = {
  value: T | null;
  visibility: FieldVisibility;
  source: "own-squad" | "scout-report" | "player-profile" | "data-hub" | "memory-raw";
  confidence: number;
  lastValidated: string | null;
};

export type RecommendationEvidence = {
  minimum: number | null;
  maximum: number | null;
  completeness: number;
  label: string;
};

export type LiveClub = {
  id: string;
  name: string;
  nation: string | null;
  league: string | null;
};

export type LiveTacticSlot = {
  playerId: string | null;
  position: string;
  role: string | null;
  roleShort?: string | null;
  roleMask?: string | null;
  duty: string | null;
  dutyShort?: string | null;
  dutyMask?: string | null;
  decoderStatus?: string;
};

export type LiveTactic = {
  name: string | null;
  formation: string;
  formationEnum?: string;
  slots: LiveTacticSlot[];
  teamInstructions: string[];
  playerInstructionsReadable: boolean;
  decoderStatus?: string;
  formationCode?: number;
  layoutStatus?: "exact-template" | "formation-name-only";
  roleDutyDecoderStatus?: string;
  rolePacketPointer?: string | null;
  rolePacketStride?: number | null;
  rolePacketWidth?: number | null;
  rolesResolved?: number;
  dutiesResolved?: number;
  warnings?: string[];
};

export type TacticSource = "none" | "live-memory";

export type LiveFootballSnapshot = {
  status: LiveConnectorStatus;
  managedClubId: string | null;
  managerName: string | null;
  season: string | null;
  clubs: LiveClub[];
  players: LivePlayer[];
  tactic: LiveTactic | null;
  tacticSource: TacticSource;
  dataError: string | null;
  dataSource?: "none" | "live-memory";
  dataWarnings?: string[];
};

export interface FootballDataAdapter {
  readonly kind: "fm26-live";
  getStatus(): Promise<LiveConnectorStatus>;
  getSnapshot(): Promise<LiveFootballSnapshot>;
}

export interface FutureRealLifeAdapter {
  readonly kind: "real-life-future";
  readonly available: false;
  readonly providerRequired: "licensed-provider";
}

const desktopRequiredStatus: LiveConnectorStatus = {
  processDetected: false,
  processId: null,
  processPath: null,
  saveDetected: null,
  memoryAccess: "not_checked",
  parserStatus: "unverified",
  state: "parser_unverified",
  playersLoaded: 0,
  managedSquadPlayers: 0,
  databasePlayersIndexed: 0,
  backgroundPlayersIndexed: 0,
  visiblePlayersLoaded: 0,
  fullyScoutedPlayers: 0,
  partialScoutReports: 0,
  databaseIndexStatus: "not_run",
  databaseScope: "none",
  clubsLoaded: 0,
  lastSync: null,
  bytesRead: 0,
  executableHeaderValid: false,
  gameBuild: null,
  productVersion: null,
  executableSha256: null,
  architecture: null,
  moduleBase: null,
  entityMapStatus: "not_checked",
  entityMapProfileId: null,
  mappingSchemaVersion: 2,
  mappingCoverage: [],
  pointerValidation: "not_run",
  handleAccessFlags: "Not available in browser",
  entityRoot: null,
  savePointer: null,
  managedClubPointer: null,
  playerCollectionPointer: null,
  liveMemoryTacticRead: "not_run",
  tacticManagerPointer: null,
  failureStage: null,
  lastSuccessfulRead: null,
  windowsErrorCode: null,
  canWriteMemory: false,
  message: "GlassScout requires the installed Windows app to connect to the active FM26 game.",
  warnings: ["No live data is being simulated."],
};

export const fm26LiveAdapter: FootballDataAdapter = {
  kind: "fm26-live",
  async getStatus() {
    if (typeof window === "undefined" || !("__TAURI_INTERNALS__" in window)) {
      return desktopRequiredStatus;
    }

    try {
      const { invoke } = await import("@tauri-apps/api/core");
      return await invoke<LiveConnectorStatus>("connector_status");
    } catch (error) {
      return {
        ...desktopRequiredStatus,
        state: "access_denied",
        memoryAccess: "denied",
        message: error instanceof Error ? error.message : "The desktop connector could not be reached.",
      };
    }
  },
  async getSnapshot() {
    if (typeof window === "undefined" || !("__TAURI_INTERNALS__" in window)) {
      return {
        status: desktopRequiredStatus,
        managedClubId: null,
        managerName: null,
        season: null,
        clubs: [],
        players: [],
        tactic: null,
        tacticSource: "none",
        dataError: desktopRequiredStatus.message,
        dataSource: "none",
        dataWarnings: desktopRequiredStatus.warnings,
      };
    }

    try {
      const { invoke } = await import("@tauri-apps/api/core");
      return await invoke<LiveFootballSnapshot>("load_active_save");
    } catch (error) {
      const message = error instanceof Error ? error.message : "The desktop connector could not be reached.";
      return {
        status: { ...desktopRequiredStatus, state: "access_denied", memoryAccess: "denied", message },
        managedClubId: null,
        managerName: null,
        season: null,
        clubs: [],
        players: [],
        tactic: null,
        tacticSource: "none",
        dataError: message,
        dataSource: "none",
        dataWarnings: [message],
      };
    }
  },
};

export type IndexedPlayerSearchResult = {
  id: string;
  name: string;
  age?: number | null;
  nationality?: string | null;
  clubId?: string | null;
  clubName?: string | null;
  positions: string[];
  managedSquad: boolean;
  visibility: "known" | "unknown";
  scoutKnowledge: "fully_known" | "partly_known" | "unknown";
  scoutConfidence: number;
  bestRole?: string | null;
  roleFit?: number | null;
  value?: string | null;
  wage?: string | null;
  contractStatus?: string | null;
  contractRemaining?: string | null;
  averageRating?: number | null;
  minutesPlayed?: number | null;
  goals?: number | null;
  assists?: number | null;
  transferInterest?: string | null;
  loanInterest?: string | null;
  transferAvailable?: boolean | null;
  loanAvailable?: boolean | null;
  notForSale?: boolean | null;
  per90?: Record<string, number | null>;
  rawStats?: Record<string, number | null>;
  inPossessionFit?: number | null;
  outOfPossessionFit?: number | null;
  projectedInPossessionFit?: number | null;
  projectedOutOfPossessionFit?: number | null;
  efficiencyScore?: number | null;
  marketValueAmount?: number | null;
};

export async function searchIndexedPlayers(query: string): Promise<IndexedPlayerSearchResult[]> {
  if (typeof window === "undefined" || !("__TAURI_INTERNALS__" in window)) return [];
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<IndexedPlayerSearchResult[]>("search_indexed_players", { query });
}

export async function indexedPlayersByIds(playerIds: string[]): Promise<IndexedPlayerSearchResult[]> {
  if (typeof window === "undefined" || !("__TAURI_INTERNALS__" in window) || playerIds.length === 0) return [];
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<IndexedPlayerSearchResult[]>("indexed_players_by_ids", { playerIds });
}

export async function indexedPlayerProfile(playerId: string): Promise<LivePlayer | null> {
  if (typeof window === "undefined" || !("__TAURI_INTERNALS__" in window)) return null;
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<LivePlayer | null>("indexed_player_profile", { playerId });
}

export type MappingLabStatus = {
  enabled: boolean;
  readOnly: boolean;
  maximumWindowBytes: number;
  evidenceDirectory: string | null;
  message: string;
};

export type MappingLabCaptureResult = {
  success: boolean;
  snapshotId: string;
  evidenceFile: string;
  playerId: string;
  playerName: string;
  windowsCaptured: number;
  bytesCaptured: number;
};

export async function getMappingLabStatus(): Promise<MappingLabStatus | null> {
  if (typeof window === "undefined" || !("__TAURI_INTERNALS__" in window)) return null;
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<MappingLabStatus>("mapping_lab_status");
}

export async function captureMappingEvidence(playerId: string, label: string): Promise<MappingLabCaptureResult> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<MappingLabCaptureResult>("mapping_lab_capture", { playerId, label, windowSize: 1024 });
}

export type MappingLabComparisonResult = {
  success: boolean;
  firstSnapshotId: string;
  secondSnapshotId: string;
  changedBytes: number;
  unchangedBytes: number;
  evidenceFile: string;
};

export async function compareMappingEvidence(firstSnapshotId: string, secondSnapshotId: string): Promise<MappingLabComparisonResult> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<MappingLabComparisonResult>("mapping_lab_compare", { firstSnapshotId, secondSnapshotId });
}

export const realLifeAdapter: FutureRealLifeAdapter = {
  kind: "real-life-future",
  available: false,
  providerRequired: "licensed-provider",
};
