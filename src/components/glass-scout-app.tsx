"use client";

import { useEffect, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { AppSidebar, type Screen } from "@/components/app-sidebar";
import { Topbar } from "@/components/topbar";
import { DashboardScreen } from "@/components/dashboard-screen";
import { MyTeamScreen } from "@/components/my-team-screen";
import { TacticsScreen } from "@/components/tactics-screen";
import { RecruitmentScreen } from "@/components/recruitment-screen";
import { FavoritedPlayersScreen } from "@/components/favorited-players-screen";
import { MemoryScreen } from "@/components/memory-screen";
import { StartupScreen } from "@/components/startup-screen";
import { SettingsScreen } from "@/components/settings-screen";
import { TooltipProvider } from "@/components/ui/tooltip";
import { fm26LiveAdapter, type LiveConnectorStatus, type LiveFootballSnapshot } from "@/domain/adapters";
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
  dataError: "Diagnostics have not run yet.",
};

export function GlassScoutApp() {
  const [mode, setMode] = useState<"fm26" | null>(null);
  const [screen, setScreen] = useState<Screen>("Dashboard");
  const [search, setSearch] = useState("");
  const [connection, setConnection] = useState<LiveConnectorStatus>(initialStatus);
  const [snapshot, setSnapshot] = useState<LiveFootballSnapshot>(initialSnapshot);
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

  useEffect(() => {
    window.localStorage.setItem("glassscout-favorites-v1", JSON.stringify(favorites));
  }, [favorites]);

  const checkConnection = async () => {
    setChecking(true);
    const nextSnapshot = await fm26LiveAdapter.getSnapshot();
    setSnapshot(nextSnapshot);
    setConnection(nextSnapshot.status);
    setChecking(false);
    return nextSnapshot.status;
  };

  const togglePlayerFavorite = (playerId: string) => setFavorites((current) => toggleFavorite(current, playerId));
  const updatePlayerNote = (playerId: string, note: string) => setFavorites((current) => updateFavoriteNote(current, playerId, note));

  if (mode === null) {
    return (
      <TooltipProvider>
        <StartupScreen onConnect={checkConnection} onEnter={(status) => { setConnection(status); setMode("fm26"); }} />
      </TooltipProvider>
    );
  }

  const content =
    screen === "Dashboard" ? <DashboardScreen snapshot={snapshot} checking={checking} onRefresh={checkConnection} onNavigate={setScreen} /> :
    screen === "My Team" ? <MyTeamScreen snapshot={snapshot} checking={checking} onRefresh={checkConnection} /> :
    screen === "Tactics" ? <TacticsScreen snapshot={snapshot} checking={checking} onRefresh={checkConnection} /> :
    screen === "Recruitment" ? <RecruitmentScreen snapshot={snapshot} favorites={favorites} checking={checking} onRefresh={checkConnection} onToggleFavorite={togglePlayerFavorite} /> :
    screen === "Favorited Players" ? <FavoritedPlayersScreen snapshot={snapshot} favorites={favorites} checking={checking} onRefresh={checkConnection} onToggleFavorite={togglePlayerFavorite} onUpdateNote={updatePlayerNote} /> :
    screen === "Memory Center" ? <MemoryScreen status={connection} checking={checking} onCheck={checkConnection} /> :
    <SettingsScreen snapshot={snapshot} checking={checking} onRefresh={checkConnection} />;

  return (
    <TooltipProvider>
      <div className="app-canvas">
        <AppSidebar screen={screen} onNavigate={setScreen} />
        <section className="app-main">
          <Topbar search={search} onSearch={setSearch} connection={connection} />
          {search ? (
            <motion.div className="search-result" initial={{ opacity: 0, y: -6 }} animate={{ opacity: 1, y: 0 }}>
              Searching the current live snapshot for <strong>“{search}”</strong>
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
