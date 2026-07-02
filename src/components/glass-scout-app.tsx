"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { AppSidebar, type Screen } from "@/components/app-sidebar";
import { Topbar } from "@/components/topbar";
import { DashboardScreen } from "@/components/dashboard-screen";
import { MyTeamScreen } from "@/components/my-team-screen";
import { TacticsScreen } from "@/components/tactics-screen";
import { ScoutRoomScreen } from "@/components/recruitment-screen";
import { FavoritedPlayersScreen } from "@/components/favorited-players-screen";
import { PlayerProfileScreen } from "@/components/player-profile-screen";
import { StartupScreen } from "@/components/startup-screen";
import { SettingsScreen } from "@/components/settings-screen";
import { ClubProfileScreen } from "@/components/club-profile-screen";
import { TooltipProvider } from "@/components/ui/tooltip";
import {
  fm26LiveAdapter,
  indexedPlayerProfile,
  searchIndexedPlayers,
  type IndexedPlayerSearchResult,
  type LiveConnectorStatus,
  type LiveFootballSnapshot,
  type LivePlayer,
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
  pointerValidation: "not_run",
  handleAccessFlags: "Not checked",
  entityRoot: null,
  savePointer: null,
  managedClubPointer: null,
  playerCollectionPointer: null,
  liveMemoryTacticRead: "not_run",
  tacticManagerPointer: null,
  failureStage: null,
  lastSuccessfulRead: null,
  windowsErrorCode: null,
  readPipeline: [],
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
  dataError: "Diagnostics have not run yet.",
  dataSource: "none",
  dataWarnings: [],
};

function meaningfulEntries(player: LivePlayer) {
  return Object.fromEntries(
    Object.entries(player).filter(([, value]) => {
      if (value == null) return false;
      if (Array.isArray(value)) return value.length > 0;
      if (typeof value === "object") return Object.keys(value).length > 0;
      return true;
    }),
  ) as Partial<LivePlayer>;
}

function mergeIndexedProfile(existing: LivePlayer | undefined, profile: LivePlayer): LivePlayer {
  if (!existing) return profile;
  return {
    ...existing,
    ...meaningfulEntries(profile),
    attributes: Object.keys(profile.attributes ?? {}).length ? profile.attributes : existing.attributes,
    per90: Object.keys(profile.per90 ?? {}).length ? profile.per90 : existing.per90,
    strengths: profile.strengths?.length ? profile.strengths : existing.strengths,
    weaknesses: profile.weaknesses?.length ? profile.weaknesses : existing.weaknesses,
    playableRoles: profile.playableRoles?.length ? profile.playableRoles : existing.playableRoles,
    otherRoles: profile.otherRoles?.length ? profile.otherRoles : existing.otherRoles,
  };
}

export function GlassScoutApp() {
  const [mode, setMode] = useState<"fm26" | null>(null);
  const [screen, setScreenState] = useState<Screen>("Dashboard");
  const [screenHistory, setScreenHistory] = useState<Screen[]>(["Dashboard"]);
  const [historyIndex, setHistoryIndex] = useState(0);
  const [search, setSearch] = useState("");
  const [snapshot, setSnapshot] = useState<LiveFootballSnapshot>(initialSnapshot);
  const [selectedPlayerId, setSelectedPlayerId] = useState<string | null>(null);
  const [selectedClubId, setSelectedClubId] = useState<string | null>(null);
  const [returnScreen, setReturnScreen] = useState<Screen>("Scout Room");
  const [globalIndexed, setGlobalIndexed] = useState<IndexedPlayerSearchResult[]>([]);
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

  useEffect(() => {
    const timer = window.setTimeout(() => {
      if (!search.trim()) {
        setGlobalIndexed([]);
        return;
      }
      searchIndexedPlayers(search)
        .then((items) => setGlobalIndexed(items.slice(0, 8)))
        .catch(() => setGlobalIndexed([]));
    }, search.trim() ? 160 : 0);
    return () => window.clearTimeout(timer);
  }, [search]);

  const navigate = useCallback((nextScreen: Screen) => {
    if (nextScreen === screen) return;
    const nextIndex = historyIndex + 1;
    setScreenHistory((current) => [...current.slice(0, nextIndex), nextScreen]);
    setHistoryIndex(nextIndex);
    setScreenState(nextScreen);
  }, [historyIndex, screen]);

  const goBack = useCallback(() => {
    if (historyIndex <= 0) return;
    const nextIndex = historyIndex - 1;
    setHistoryIndex(nextIndex);
    setScreenState(screenHistory[nextIndex]);
  }, [historyIndex, screenHistory]);

  const goForward = useCallback(() => {
    if (historyIndex >= screenHistory.length - 1) return;
    const nextIndex = historyIndex + 1;
    setHistoryIndex(nextIndex);
    setScreenState(screenHistory[nextIndex]);
  }, [historyIndex, screenHistory]);

  const checkConnection = useCallback(async () => {
    setChecking(true);
    try {
      const nextSnapshot = await fm26LiveAdapter.getSnapshot();
      setSnapshot(nextSnapshot);
      return nextSnapshot.status;
    } finally {
      setChecking(false);
    }
  }, []);

  const enterWorkspace = useCallback(() => {
    setScreenState("Dashboard");
    setScreenHistory(["Dashboard"]);
    setHistoryIndex(0);
    setMode("fm26");
  }, []);

  const togglePlayerFavorite = (playerId: string) => setFavorites((current) => toggleFavorite(current, playerId));
  const updatePlayerNote = (playerId: string, note: string) => setFavorites((current) => updateFavoriteNote(current, playerId, note));

  const loadIndexedPlayerProfile = useCallback(async (playerId: string) => {
    try {
      const profile = await indexedPlayerProfile(playerId);
      if (!profile) return;
      setSnapshot((current) => {
        const existingIndex = current.players.findIndex((player) => player.id === profile.id);
        if (existingIndex === -1) return { ...current, players: [...current.players, profile] };
        const players = [...current.players];
        players[existingIndex] = mergeIndexedProfile(players[existingIndex], profile);
        return { ...current, players };
      });
    } catch {
      // The profile screen will keep its clean unavailable state if the local save index cannot resolve this player.
    }
  }, []);

  const openPlayer = useCallback((playerId: string) => {
    setReturnScreen((current) => screen === "Player Profile" || screen === "Club Profile" ? current : screen);
    setSelectedPlayerId(playerId);
    navigate("Player Profile");
    void loadIndexedPlayerProfile(playerId);
  }, [loadIndexedPlayerProfile, navigate, screen]);

  const openClub = useCallback((clubId: string) => {
    setReturnScreen((current) => screen === "Player Profile" || screen === "Club Profile" ? current : screen);
    setSelectedClubId(clubId);
    navigate("Club Profile");
  }, [navigate, screen]);

  const goBackOrReturn = useCallback(() => {
    if (historyIndex > 0) {
      goBack();
      return;
    }
    navigate(returnScreen);
  }, [goBack, historyIndex, navigate, returnScreen]);

  const globalClubs = useMemo(
    () => search.trim()
      ? snapshot.clubs.filter((club) => club.name.toLowerCase().includes(search.trim().toLowerCase())).slice(0, 4)
      : [],
    [search, snapshot.clubs],
  );

  if (mode === null) {
    return (
      <TooltipProvider>
        <StartupScreen onConnect={checkConnection} onEnter={enterWorkspace} />
      </TooltipProvider>
    );
  }

  const content =
    screen === "Dashboard" ? <DashboardScreen snapshot={snapshot} checking={checking} onRefresh={checkConnection} onNavigate={navigate} /> :
    screen === "Squad" ? <MyTeamScreen snapshot={snapshot} checking={checking} onRefresh={checkConnection} onOpenPlayer={openPlayer} /> :
    screen === "Tactical Board" ? <TacticsScreen snapshot={snapshot} onOpenPlayer={openPlayer} /> :
    screen === "Scout Room" ? <ScoutRoomScreen snapshot={snapshot} favorites={favorites} checking={checking} onRefresh={checkConnection} onToggleFavorite={togglePlayerFavorite} onOpenPlayer={openPlayer} /> :
    screen === "Shortlist" ? <FavoritedPlayersScreen snapshot={snapshot} favorites={favorites} checking={checking} onRefresh={checkConnection} onToggleFavorite={togglePlayerFavorite} onUpdateNote={updatePlayerNote} onOpenPlayer={openPlayer} /> :
    screen === "Player Profile" ? (
      <PlayerProfileScreen
        player={snapshot.players.find((player) => player.id === selectedPlayerId) ?? null}
        snapshot={snapshot}
        favorite={selectedPlayerId ? favorites.some((record) => record.playerId === selectedPlayerId) : false}
        onToggleFavorite={() => selectedPlayerId && togglePlayerFavorite(selectedPlayerId)}
        onBack={goBackOrReturn}
        onOpenClub={openClub}
      />
    ) : screen === "Club Profile" ? (
      <ClubProfileScreen clubId={selectedClubId} snapshot={snapshot} onBack={goBackOrReturn} onOpenPlayer={openPlayer} />
    ) : (
      <SettingsScreen snapshot={snapshot} checking={checking} onRefresh={checkConnection} />
    );

  return (
    <TooltipProvider>
      <div className="app-canvas">
        <div className="app-top-shell">
          <AppSidebar screen={screen} onNavigate={navigate} />
          <Topbar
            search={search}
            onSearch={setSearch}
            snapshot={snapshot}
            screen={screen}
            checking={checking}
            onRefresh={checkConnection}
            canGoBack={historyIndex > 0}
            canGoForward={historyIndex < screenHistory.length - 1}
            onGoBack={goBack}
            onGoForward={goForward}
          />
          {search ? (
            <motion.div className="global-search-results" initial={{ opacity: 0, y: -6 }} animate={{ opacity: 1, y: 0 }}>
              <header><strong>Search results</strong><button onClick={() => setSearch("")}>Clear</button></header>
              {globalClubs.map((club) => (
                <button key={club.id} onClick={() => { openClub(club.id); setSearch(""); }}>
                  <span>Team</span>
                  <strong>{club.name}</strong>
                  <small>{club.league ?? "Competition unknown"}</small>
                </button>
              ))}
              {globalIndexed.map((result) => (
                <button key={result.id} onClick={() => { openPlayer(result.id); setSearch(""); }}>
                  <span>Player</span>
                  <strong>{result.name}</strong>
                  <small>{result.positions.join(" / ") || "Position unknown"} · {result.visibility}</small>
                </button>
              ))}
              {!globalClubs.length && !globalIndexed.length ? <p>No indexed player or mapped team matches “{search}”.</p> : null}
            </motion.div>
          ) : null}
        </div>
        <section className="app-main">
          <AnimatePresence mode="wait">
            <div key={screen} className="screen-slot">{content}</div>
          </AnimatePresence>
        </section>
      </div>
    </TooltipProvider>
  );
}
