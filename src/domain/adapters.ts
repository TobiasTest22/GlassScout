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

export type LiveFootballSnapshot = {
  status: LiveConnectorStatus;
  managedClubId: string | null;
  managerName: string | null;
  season: string | null;
  clubs: LiveClub[];
  players: LivePlayer[];
  tactic: LiveTactic | null;
  dataError: string | null;
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

const browserPreviewStatus: LiveConnectorStatus = {
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
  canWriteMemory: false,
  message: "Desktop connector unavailable in browser preview. Run the Tauri app to inspect FM26.",
  warnings: ["No live data is being simulated."],
};

export const fm26LiveAdapter: FootballDataAdapter = {
  kind: "fm26-live",
  async getStatus() {
    if (typeof window === "undefined" || !("__TAURI_INTERNALS__" in window)) {
      return browserPreviewStatus;
    }

    try {
      const { invoke } = await import("@tauri-apps/api/core");
      return await invoke<LiveConnectorStatus>("connector_status");
    } catch (error) {
      return {
        ...browserPreviewStatus,
        state: "access_denied",
        memoryAccess: "denied",
        message: error instanceof Error ? error.message : "The desktop connector could not be reached.",
      };
    }
  },
  async getSnapshot() {
    if (typeof window === "undefined" || !("__TAURI_INTERNALS__" in window)) {
      return {
        status: browserPreviewStatus,
        managedClubId: null,
        managerName: null,
        season: null,
        clubs: [],
        players: [],
        tactic: null,
        dataError: browserPreviewStatus.message,
      };
    }

    try {
      const { invoke } = await import("@tauri-apps/api/core");
      return await invoke<LiveFootballSnapshot>("connector_snapshot");
    } catch (error) {
      const message = error instanceof Error ? error.message : "The desktop connector could not be reached.";
      return {
        status: { ...browserPreviewStatus, state: "access_denied", memoryAccess: "denied", message },
        managedClubId: null,
        managerName: null,
        season: null,
        clubs: [],
        players: [],
        tactic: null,
        dataError: message,
      };
    }
  },
};

export const realLifeAdapter: FutureRealLifeAdapter = {
  kind: "real-life-future",
  available: false,
  providerRequired: "licensed-provider",
};
