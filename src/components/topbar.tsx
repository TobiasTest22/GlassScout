"use client";

import { Bell, Command, Search, Wifi, WifiOff } from "lucide-react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import type { LiveConnectorStatus } from "@/domain/adapters";

export function Topbar({ search, onSearch, connection }: { search: string; onSearch: (value: string) => void; connection: LiveConnectorStatus }) {
  const connected = connection.state === "connected";
  return (
    <header className="topbar">
      <div className="search-wrap">
        <Search aria-hidden="true" />
        <Input
          aria-label="Search players and reports"
          placeholder={connected ? "Search players and reports…" : "Waiting for FM26 live data"}
          value={search}
          onChange={(event) => onSearch(event.target.value)}
          disabled={!connected}
        />
        <span className="keyboard-hint"><Command /> K</span>
      </div>
      <div className="topbar-actions">
        <Badge variant="outline" className="sync-badge">
          <span className={connected ? "live-dot" : "neutral-dot"} />
          {connected ? "FM26 live" : connection.processDetected ? "FM26 detected · connection blocked" : "Waiting for FM26"}
        </Badge>
        <Button variant="outline" size="icon" aria-label={connected ? "Football data connected" : "FM26 data requires attention"}>
          {connected ? <Wifi /> : <WifiOff />}
        </Button>
        <Button variant="ghost" size="icon" aria-label="Notifications"><Bell /></Button>
        <div className="user-avatar">TB</div>
      </div>
    </header>
  );
}
