"use client";

import { Bell, Command, HelpCircle, RefreshCw, Search } from "lucide-react";
import type { Screen } from "@/components/app-sidebar";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import type { LiveFootballSnapshot } from "@/domain/adapters";

function syncLabel(value: string | null) {
  if (!value) return "Not synced";
  const elapsed = Math.max(0, Date.now() - Number(value));
  if (elapsed < 60_000) return "Synced just now";
  return `Synced ${Math.max(1, Math.round(elapsed / 60_000))}m ago`;
}

export function Topbar({
  search,
  onSearch,
  snapshot,
  screen,
  checking,
  onRefresh,
}: {
  search: string;
  onSearch: (value: string) => void;
  snapshot: LiveFootballSnapshot;
  screen: Screen;
  checking: boolean;
  onRefresh: () => Promise<unknown>;
}) {
  const connected = snapshot.status.state === "connected";
  const club = snapshot.clubs.find((item) => item.id === snapshot.managedClubId);
  const dashboard = screen === "Dashboard";

  return (
    <header className={dashboard ? "topbar topbar-dashboard" : "topbar"}>
      {dashboard ? (
        <div className="dashboard-greeting">
          <strong>Morning, {snapshot.managerName?.split(" ")[0] ?? "Manager"}</strong>
          <span>{club?.name ?? "Active club"} <i /> {snapshot.season ?? "Active save"}</span>
        </div>
      ) : null}
      <div className="search-wrap">
        <Search aria-hidden="true" />
        <Input
          aria-label="Search players, clubs and reports"
          placeholder={connected ? "Search players, clubs, reports…" : "Load the active FM26 save to search"}
          value={search}
          onChange={(event) => onSearch(event.target.value)}
          disabled={!connected}
        />
        <span className="keyboard-hint"><Command /> K</span>
      </div>
      <div className="topbar-actions">
        <div className="sync-badge">
          <span className={connected ? "live-dot" : "neutral-dot"} />
          <span>{connected ? "Live memory" : "Disconnected"}</span>
          <i />
          <span>{syncLabel(snapshot.status.lastSync)}</span>
        </div>
        <Button variant="outline" size="icon" aria-label="Reload active save" onClick={onRefresh} disabled={checking}>
          <RefreshCw className={checking ? "spin" : undefined} />
        </Button>
        {!dashboard ? (
          <>
            <Button variant="ghost" size="icon" aria-label="Notifications"><Bell /></Button>
            <Button variant="ghost" size="icon" aria-label="Help"><HelpCircle /></Button>
            <div className="user-avatar">TB</div>
          </>
        ) : null}
      </div>
    </header>
  );
}
