"use client";

import { useMemo, useState } from "react";
import {
  ChevronDown,
  Database,
  ExternalLink,
  Search,
  ShieldCheck,
  Star,
} from "lucide-react";
import type { LiveFootballSnapshot, LivePlayer } from "@/domain/adapters";
import type { FavoriteRecord } from "@/domain/live-data";
import { LiveDataState } from "@/components/live-data-state";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

const knownInterest = new Set([
  "Very interested",
  "Interested",
  "Slightly interested",
  "Only loan possible",
  "Only if promoted",
  "Only if wages are improved",
  "Only if guaranteed playing time",
]);

function interestLabel(player: LivePlayer) {
  return player.transferInterest ?? player.loanInterest ?? "Unknown";
}

function availabilityLabel(player: LivePlayer) {
  if (player.transferAvailable === true) return "Transfer";
  if (player.loanAvailable === true) return "Loan";
  if (player.transferAvailable === false && player.loanAvailable === false) return "Not available";
  return "Unknown";
}

function realismLabel(player: LivePlayer) {
  const interest = interestLabel(player);
  if (interest === "Not interested") return "Not interested";
  if (interest === "Only loan possible") return "Loan only";
  if (!knownInterest.has(interest)) return "Unknown";
  if (!player.value || !player.wage) return "Needs financial evidence";
  return "Interest recorded";
}

export function RecruitmentScreen({
  snapshot,
  favorites,
  checking,
  onRefresh,
  onToggleFavorite,
  onOpenPlayer,
}: {
  snapshot: LiveFootballSnapshot;
  favorites: FavoriteRecord[];
  checking: boolean;
  onRefresh: () => Promise<unknown>;
  onToggleFavorite: (playerId: string) => void;
  onOpenPlayer: (playerId: string) => void;
}) {
  const [query, setQuery] = useState("");
  const [position, setPosition] = useState("All positions");
  const [interest, setInterest] = useState("All interest");
  const [realism, setRealism] = useState("All realism");
  const [transferType, setTransferType] = useState("All types");

  const clubById = useMemo(() => new Map(snapshot.clubs.map((club) => [club.id, club])), [snapshot.clubs]);
  const favoriteIds = useMemo(() => new Set(favorites.map((record) => record.playerId)), [favorites]);
  const visibleTargets = useMemo(
    () => snapshot.players.filter((player) => player.clubId !== snapshot.managedClubId),
    [snapshot.managedClubId, snapshot.players],
  );
  const positions = useMemo(
    () => ["All positions", ...new Set(visibleTargets.flatMap((player) => player.positions))],
    [visibleTargets],
  );
  const players = useMemo(() => {
    const normalizedQuery = query.trim().toLowerCase();
    return visibleTargets
      .filter((player) => {
        const club = player.clubId ? clubById.get(player.clubId) : null;
        const matchesQuery = !normalizedQuery || [player.name, club?.name, player.nationality, ...player.positions]
          .filter(Boolean)
          .some((value) => value?.toLowerCase().includes(normalizedQuery));
        const matchesPosition = position === "All positions" || player.positions.includes(position);
        const playerInterest = interestLabel(player);
        const playerRealism = realismLabel(player);
        const playerType = availabilityLabel(player);
        return matchesQuery
          && matchesPosition
          && (interest === "All interest" || playerInterest === interest)
          && (realism === "All realism" || playerRealism === realism)
          && (transferType === "All types" || playerType === transferType);
      })
      .toSorted((a, b) => {
        const aKnown = knownInterest.has(interestLabel(a)) ? 1 : 0;
        const bKnown = knownInterest.has(interestLabel(b)) ? 1 : 0;
        if (aKnown !== bKnown) return bKnown - aKnown;
        return (b.tacticalFit ?? -1) - (a.tacticalFit ?? -1);
      });
  }, [clubById, interest, position, query, realism, transferType, visibleTargets]);

  if (snapshot.status.state !== "connected") {
    return <main className="screen"><LiveDataState snapshot={snapshot} title="Recruitment" checking={checking} onRefresh={onRefresh} /></main>;
  }

  return (
    <main className="screen recruitment-hub">
      <div className="planner-heading">
        <div>
          <h1>Recruitment Hub</h1>
          <p>Only club-visible players can enter this decision workflow.</p>
        </div>
        <div className="database-scope-badge"><Database />{snapshot.status.databasePlayersIndexed} indexed · {snapshot.status.visiblePlayersLoaded} visible</div>
      </div>

      <section className="recruitment-command-bar" aria-label="Recruitment filters">
        <div className="recruitment-search"><Search /><Input aria-label="Search recruitment players" placeholder="Search players, clubs, nations…" value={query} onChange={(event) => setQuery(event.target.value)} /></div>
        <label><span>Position</span><select value={position} onChange={(event) => setPosition(event.target.value)}>{positions.map((value) => <option key={value}>{value}</option>)}</select><ChevronDown /></label>
        <label><span>Interest</span><select value={interest} onChange={(event) => setInterest(event.target.value)}><option>All interest</option><option>Very interested</option><option>Interested</option><option>Slightly interested</option><option>Doubtful</option><option>Not interested</option><option>Unknown</option></select><ChevronDown /></label>
        <label><span>Realism</span><select value={realism} onChange={(event) => setRealism(event.target.value)}><option>All realism</option><option>Interest recorded</option><option>Needs financial evidence</option><option>Loan only</option><option>Not interested</option><option>Unknown</option></select><ChevronDown /></label>
        <label><span>Transfer type</span><select value={transferType} onChange={(event) => setTransferType(event.target.value)}><option>All types</option><option>Transfer</option><option>Loan</option><option>Not available</option><option>Unknown</option></select><ChevronDown /></label>
      </section>

      <section className="recruitment-board">
        <header>
          <span>Player</span><span>Role & tactical fit</span><span>Knowledge</span><span>Interest & realism</span><span>Financials</span><span>Availability</span><span>Projection / risk</span><span>Next action</span>
        </header>
        {players.map((player) => {
          const club = player.clubId ? clubById.get(player.clubId) : null;
          const favorite = favoriteIds.has(player.id);
          return (
            <article key={player.id}>
              <div className="target-identity">
                <span>{player.name.split(" ").map((part) => part[0]).join("").slice(0, 2)}</span>
                <div><strong>{player.name}</strong><small>{player.age ?? "Age unknown"} · {player.nationality ?? "Nation unknown"} · {player.positions.join(" / ") || "Position unknown"}</small><b>{club?.name ?? "Club unknown"}</b></div>
              </div>
              <div><strong>{player.bestRole ?? "Unknown"}</strong><small>Tactical fit {player.tacticalFit == null ? "Unknown" : `${player.tacticalFit}%`}</small></div>
              <div><strong>{player.scoutKnowledge?.replaceAll("_", " ") ?? "Unknown"}</strong><small>Confidence Unknown</small></div>
              <div><strong>{interestLabel(player)}</strong><small>{realismLabel(player)}</small></div>
              <div><strong>{player.value ?? "Value unknown"}</strong><small>{player.wage ?? "Wage unknown"}</small></div>
              <div><strong>{availabilityLabel(player)}</strong><small>{player.contractStatus ?? "Contract unknown"}</small></div>
              <div><strong>Projection unknown</strong><small>{player.riskLevel === "unknown" || !player.riskLevel ? "Risk unknown" : `${player.riskLevel} risk`}</small></div>
              <div className="target-actions">
                <Button variant="outline" size="icon-sm" aria-label={`Open ${player.name}`} onClick={() => onOpenPlayer(player.id)}><ExternalLink /></Button>
                <Button variant={favorite ? "secondary" : "outline"} size="icon-sm" aria-label={`${favorite ? "Remove" : "Add"} ${player.name} shortlist`} onClick={() => onToggleFavorite(player.id)}><Star /></Button>
              </div>
            </article>
          );
        })}
        {!players.length ? (
          <div className="recruitment-zero-state">
            <ShieldCheck />
            <h2>No visibility-safe recruitment targets</h2>
            <p>
              GlassScout indexed {snapshot.status.backgroundPlayersIndexed} wider-save player records, but none can be shown until FM26 scout-knowledge visibility is mapped and validated.
            </p>
          </div>
        ) : null}
      </section>
    </main>
  );
}
