"use client";

import { useCallback, useEffect, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { AppSidebar, type Screen } from "@/components/app-sidebar";
import { Topbar } from "@/components/topbar";
import { DashboardScreen } from "@/components/dashboard-screen";
import { MyTeamScreen } from "@/components/my-team-screen";
import { TacticsScreen } from "@/components/tactics-screen";
import { RoleDnaScreen } from "@/components/role-dna-screen";
import { RecruitmentScreen } from "@/components/recruitment-screen";
import { FavoritedPlayersScreen } from "@/components/favorited-players-screen";
import { PlayerProfileScreen } from "@/components/player-profile-screen";
import { StartupScreen } from "@/components/startup-screen";
import { SettingsScreen } from "@/components/settings-screen";
import { TooltipProvider } from "@/components/ui/tooltip";
import {
  chooseAndImportFmfTactic,
  fm26LiveAdapter,
  mergeTacticFile,
  type LiveConnectorStatus,
  type LiveFootballSnapshot,
} from "@/domain/adapters";
import { toggleFavorite, updateFavoriteNote, type FavoriteRecord } from "@/domain/live-data";

const initialStatus: LiveConnectorStatus = {
  processDetected: false,
  processId: null,
  processPath: null,
  saveDetected: null,
  memoryAccess: "not_checked",
  parserStatus: "unverified",
  state: "not_checked",
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
  handleAccessFlags: "Not checked",
  entityRoot: null,
  savePointer: null,
  managedClubPointer: null,
  playerCollectionPointer: null,
  liveMemoryTacticRead: "disabled",
  failureStage: null,
  lastSuccessfulRead: null,
  windowsErrorCode: null,
  canWriteMemory: false,
  message: "Diagnostics have not run yet.",
  warnings: [],
};

const initialSnapshot: LiveFootballSnapshot = {
  status: initialStatus,
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
  dataError: "Diagnostics have not run yet.",
  dataSource: "none",
  dataWarnings: [],
};

export function GlassScoutApp() {
  const [mode, setMode] = useState<"fm26" | null>(null);
  const [screen, setScreen] = useState<Screen>("Dashboard");
  const [search, setSearch] = useState("");
  const [connection, setConnection] = useState<LiveConnectorStatus>(initialStatus);
  const [snapshot, setSnapshot] = useState<LiveFootballSnapshot>(initialSnapshot);
  const [selectedPlayerId, setSelectedPlayerId] = useState<string | null>(null);
  const [favorites, setFavorites] = useState<FavoriteRecord[]>(() => {
    if (typeof window === "undefined") return [];
    const stored = window.localStorage.getItem("glassscout-favorites-v1");
    if (!stored) return [];
    try {
      return JSON.parse(stored) as FavoriteRecord[];
    } catch {
      window.localStorage.removeItem("glassscout-favorites-v1");
      return [];
    }
  });
  const [checking, setChecking] = useState(false);
  const [importingTactic, setImportingTactic] = useState(false);

  useEffect(() => {
    window.localStorage.setItem("glassscout-favorites-v1", JSON.stringify(favorites));
  }, [favorites]);

  const checkConnection = useCallback(async () => {
    setChecking(true);
    try {
      const nextSnapshot = await fm26LiveAdapter.getSnapshot();
      setSnapshot(nextSnapshot);
      setConnection(nextSnapshot.status);
      return nextSnapshot.status;
    } finally {
      setChecking(false);
    }
  }, []);
  const enterWorkspace = useCallback((status: LiveConnectorStatus) => {
    setConnection(status);
    setScreen("Dashboard");
    setMode("fm26");
  }, []);

  const togglePlayerFavorite = (playerId: string) => setFavorites((current) => toggleFavorite(current, playerId));
  const updatePlayerNote = (playerId: string, note: string) => setFavorites((current) => updateFavoriteNote(current, playerId, note));
  const importTactic = useCallback(async () => {
    setImportingTactic(true);
    try {
      const result = await chooseAndImportFmfTactic();
      if (result) {
        setSnapshot((current) => mergeTacticFile(current, result));
      }
    } finally {
      setImportingTactic(false);
    }
  }, []);
  const openPlayer = (playerId: string) => {
    setSelectedPlayerId(playerId);
    setScreen("Player Profile");
  };

  if (mode === null) {
    return (
      <TooltipProvider>
        <StartupScreen onConnect={checkConnection} onEnter={enterWorkspace} />
      </TooltipProvider>
    );
  }

  const content =
    screen === "Dashboard" ? <DashboardScreen snapshot={snapshot} checking={checking} onRefresh={checkConnection} onNavigate={setScreen} /> :
    screen === "My Team" ? <MyTeamScreen snapshot={snapshot} checking={checking} onRefresh={checkConnection} onOpenPlayer={openPlayer} /> :
    screen === "Tactic Evaluation" ? <TacticsScreen snapshot={snapshot} importing={importingTactic} onImportTactic={importTactic} /> :
    screen === "Role DNA" ? <RoleDnaScreen snapshot={snapshot} checking={checking} onRefresh={checkConnection} /> :
    screen === "Recruitment" ? <RecruitmentScreen snapshot={snapshot} favorites={favorites} checking={checking} onRefresh={checkConnection} onToggleFavorite={togglePlayerFavorite} onOpenPlayer={openPlayer} /> :
    screen === "Favorites / Shortlist" ? <FavoritedPlayersScreen snapshot={snapshot} favorites={favorites} checking={checking} onRefresh={checkConnection} onToggleFavorite={togglePlayerFavorite} onUpdateNote={updatePlayerNote} /> :
    screen === "Player Profile" ? <PlayerProfileScreen player={snapshot.players.find((player) => player.id === selectedPlayerId) ?? null} snapshot={snapshot} onBack={() => setScreen("Recruitment")} /> :
    <SettingsScreen snapshot={snapshot} checking={checking} onRefresh={checkConnection} />;

  return (
    <TooltipProvider>
      <div className="app-canvas">
        <AppSidebar screen={screen} onNavigate={setScreen} />
        <section className="app-main">
          <Topbar search={search} onSearch={setSearch} connection={connection} />
          {search ? (
            <motion.div className="search-result" initial={{ opacity: 0, y: -6 }} animate={{ opacity: 1, y: 0 }}>
              Searching the current dataset for <strong>“{search}”</strong>
              <button onClick={() => setSearch("")}>Clear</button>
            </motion.div>
          ) : null}
          <AnimatePresence mode="wait">
            <div key={screen} className="screen-slot">{content}</div>
          </AnimatePresence>
        </section>
      </div>
    </TooltipProvider>
  );
}
