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
  pointerValidation?: "not_run" | "passed" | "failed";
  handleAccessFlags?: string;
  entityRoot?: string | null;
  savePointer?: string | null;
  managedClubPointer?: string | null;
  playerCollectionPointer?: string | null;
  liveMemoryTacticRead?: "disabled";
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
  nationality: string | null;
  positions: string[];
  bestRole: string | null;
  currentAbility: number | null;
  potentialAbility: number | null;
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
  strengths: string[];
  weaknesses: string[];
  clubId: string | null;
  transferInterest: string | null;
  loanInterest: string | null;
  transferAvailable: boolean | null;
  loanAvailable: boolean | null;
  attributes?: Record<string, number | null>;
  per90?: Record<string, number | null>;
  scoutKnowledge?: "fully_known" | "partly_known" | "unknown" | "needs_scouting" | "missing_data";
  bestCalculatedPosition?: string | null;
  truePrice?: number | null;
  fairPriceRange?: [number, number] | null;
  valuationLabel?: "undervalued" | "fair" | "overpriced" | "unavailable";
  valuationReasoning?: string[];
  retrainingSuggestion?: string | null;
  roleReasoning?: string[];
  riskLevel?: "low" | "medium" | "high" | "unknown";
  marketValueAmount?: number | null;
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
  duty: string | null;
};

export type LiveTactic = {
  name: string | null;
  formation: string;
  slots: LiveTacticSlot[];
  teamInstructions: string[];
  playerInstructionsReadable: boolean;
};

export type TacticSource = "none" | "fmf-file" | "live-memory";

export type TacticParserStatus =
  | "not_imported"
  | "imported_unparsed"
  | "partially_parsed"
  | "parsed"
  | "unsupported_format"
  | "invalid_file";

export type TacticFileResult = {
  success: boolean;
  fileName: string | null;
  fileSize: number | null;
  importedAt: string | null;
  parserStatus: TacticParserStatus;
  parsedFormation: string | null;
  parsedRoles: string[];
  parsedDuties: string[];
  detectedFormat: string | null;
  compressed: boolean | null;
  encoded: boolean | null;
  warnings: string[];
  errors: string[];
};

export type LiveFootballSnapshot = {
  status: LiveConnectorStatus;
  managedClubId: string | null;
  managerName: string | null;
  season: string | null;
  clubs: LiveClub[];
  players: LivePlayer[];
  tactic: LiveTactic | null;
  tacticSource: TacticSource;
  tacticFileStatus: TacticParserStatus;
  tacticFileName: string | null;
  tacticFileWarnings: string[];
  tacticFileSize?: number | null;
  tacticImportedAt?: string | null;
  tacticFileErrors?: string[];
  tacticDetectedFormat?: string | null;
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
  pointerValidation: "not_run",
  handleAccessFlags: "Not available in browser",
  entityRoot: null,
  savePointer: null,
  managedClubPointer: null,
  playerCollectionPointer: null,
  liveMemoryTacticRead: "disabled",
  failureStage: null,
  lastSuccessfulRead: null,
  windowsErrorCode: null,
  canWriteMemory: false,
  message: "GlassScout requires the installed Windows app to connect to the active FM26 game.",
  warnings: ["No live data is being simulated."],
};

const notImportedTactic: TacticFileResult = {
  success: false,
  fileName: null,
  fileSize: null,
  importedAt: null,
  parserStatus: "not_imported",
  parsedFormation: null,
  parsedRoles: [],
  parsedDuties: [],
  detectedFormat: null,
  compressed: null,
  encoded: null,
  warnings: [],
  errors: [],
};

export function mergeTacticFile(
  snapshot: LiveFootballSnapshot,
  file: TacticFileResult,
): LiveFootballSnapshot {
  const hasParsedTactic = (
    file.parserStatus === "parsed" || file.parserStatus === "partially_parsed"
  ) && file.parsedFormation != null;
  return {
    ...snapshot,
    tactic: hasParsedTactic ? {
      name: file.fileName,
      formation: file.parsedFormation ?? "Unavailable",
      slots: [],
      teamInstructions: [],
      playerInstructionsReadable: false,
    } : null,
    tacticSource: file.success ? "fmf-file" : "none",
    tacticFileStatus: file.parserStatus,
    tacticFileName: file.fileName,
    tacticFileWarnings: file.warnings,
    tacticFileSize: file.fileSize,
    tacticImportedAt: file.importedAt,
    tacticFileErrors: file.errors,
    tacticDetectedFormat: file.detectedFormat,
  };
}

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
        tacticFileStatus: "not_imported",
        tacticFileName: null,
        tacticFileWarnings: [],
        dataError: desktopRequiredStatus.message,
        dataSource: "none",
        dataWarnings: desktopRequiredStatus.warnings,
      };
    }

    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const [snapshot, tacticFile] = await Promise.all([
        invoke<LiveFootballSnapshot>("connector_snapshot"),
        invoke<TacticFileResult>("tactic_file_status").catch(() => notImportedTactic),
      ]);
      return mergeTacticFile(snapshot, tacticFile);
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
        tacticFileStatus: "not_imported",
        tacticFileName: null,
        tacticFileWarnings: [],
        dataError: message,
        dataSource: "none",
        dataWarnings: [message],
      };
    }
  },
};

export async function chooseAndImportFmfTactic(): Promise<TacticFileResult | null> {
  if (typeof window === "undefined" || !("__TAURI_INTERNALS__" in window)) {
    return {
      ...notImportedTactic,
      parserStatus: "invalid_file",
      errors: ["FMF tactic import is available only in the installed Windows app."],
    };
  }
  const [{ open }, { invoke }] = await Promise.all([
    import("@tauri-apps/plugin-dialog"),
    import("@tauri-apps/api/core"),
  ]);
  const selected = await open({
    title: "Choose an FM26 tactic",
    multiple: false,
    directory: false,
    filters: [{ name: "Football Manager tactic", extensions: ["fmf"] }],
  });
  if (selected == null || Array.isArray(selected)) {
    return null;
  }
  return await invoke<TacticFileResult>("import_tactic_file", { path: selected });
}

export const realLifeAdapter: FutureRealLifeAdapter = {
  kind: "real-life-future",
  available: false,
  providerRequired: "licensed-provider",
};
