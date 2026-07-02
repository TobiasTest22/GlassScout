"use client";

import { useMemo, useState } from "react";
import { Database, ExternalLink, Search, ShieldCheck, Star } from "lucide-react";
import type { LiveFootballSnapshot, LivePlayer } from "@/domain/adapters";
import type { FavoriteRecord } from "@/domain/live-data";
import { LiveDataState } from "@/components/live-data-state";
import { PlayerFace } from "@/components/player-face";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

function interestLabel(player: LivePlayer) {
  return player.transferInterest ?? player.loanInterest ?? "Unknown";
}

function fitLabel(player: LivePlayer) {
  if (player.tacticalFit == null) return "Tactic unavailable";
  if (player.tacticalFit >= 70) return "Strong";
  if (player.tacticalFit >= 50) return "Medium";
  return "Low";
}

function knowledgeLabel(player: LivePlayer) {
  return (player.scoutKnowledge ?? "unknown").replaceAll("_", " ");
}

function availabilityLabel(player: LivePlayer) {
  if (player.transferAvailable === true) return "Transfer";
  if (player.loanAvailable === true) return "Loan";
  if (player.transferAvailable === false && player.loanAvailable === false) return "Not available";
  return "Unknown";
}

function Filter({
  label,
  value,
  options,
  onChange,
}: {
  label: string;
  value: string;
  options: string[];
  onChange: (value: string) => void;
}) {
  return (
    <label className="scout-filter">
      <span>{label}</span>
      <select value={value} onChange={(event) => onChange(event.target.value)}>
        {options.map((option) => <option key={option}>{option}</option>)}
      </select>
    </label>
  );
}

function RatingRing({ value }: { value: number | null }) {
  const safe = value == null ? 0 : Math.max(0, Math.min(100, value));
  const tone = value == null ? "unknown" : value >= 70 ? "strong" : value >= 50 ? "medium" : "poor";
  return (
    <span
      className={`fit-score-ring fit-score-${tone}`}
      style={{ "--fit-score": `${safe * 3.6}deg` } as React.CSSProperties}
    >
      <strong>{value ?? "—"}</strong>
    </span>
  );
}

export function ScoutRoomScreen({
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
  const [position, setPosition] = useState("All");
  const [nation, setNation] = useState("All");
  const [role, setRole] = useState("All");
  const [age, setAge] = useState("All");
  const [fit, setFit] = useState("All");
  const [interest, setInterest] = useState("All");
  const [knowledge, setKnowledge] = useState("All");
  const [contract, setContract] = useState("All");
  const [wage, setWage] = useState("All");
  const [value, setValue] = useState("All");
  const [transferType, setTransferType] = useState("All");
  const [recommendation, setRecommendation] = useState("All");

  const clubById = useMemo(() => new Map(snapshot.clubs.map((club) => [club.id, club])), [snapshot.clubs]);
  const favoriteIds = useMemo(() => new Set(favorites.map((record) => record.playerId)), [favorites]);
  const positions = useMemo(() => ["All", ...new Set(snapshot.players.flatMap((player) => player.positions))], [snapshot.players]);
  const nations = useMemo(() => ["All", ...new Set(snapshot.players.flatMap((player) => player.nationality ? [player.nationality] : []))], [snapshot.players]);
  const roles = useMemo(() => ["All", ...new Set(snapshot.players.flatMap((player) => player.bestRole ? [player.bestRole] : []))], [snapshot.players]);

  const players = useMemo(() => {
    const normalized = query.trim().toLowerCase();
    return snapshot.players
      .filter((player) => {
        const club = player.clubId ? clubById.get(player.clubId) : null;
        const matchesQuery = !normalized || [player.name, club?.name, player.nationality, ...player.positions]
          .filter(Boolean)
          .some((item) => item?.toLowerCase().includes(normalized));
        const ageBand = player.age == null ? "Unknown" : player.age <= 21 ? "U21" : player.age <= 27 ? "22–27" : "28+";
        const playerRecommendation = player.clubId === snapshot.managedClubId ? "Current squad" : "Unknown";
        return matchesQuery
          && (position === "All" || player.positions.includes(position))
          && (nation === "All" || player.nationality === nation)
          && (role === "All" || player.bestRole === role)
          && (age === "All" || ageBand === age)
          && (fit === "All" || fitLabel(player) === fit)
          && (interest === "All" || interestLabel(player) === interest)
          && (knowledge === "All" || knowledgeLabel(player) === knowledge)
          && (contract === "All" || (contract === "Known") === Boolean(player.contractStatus))
          && (wage === "All" || (wage === "Known") === Boolean(player.wage))
          && (value === "All" || (value === "Known") === Boolean(player.value))
          && (transferType === "All" || availabilityLabel(player) === transferType)
          && (recommendation === "All" || playerRecommendation === recommendation);
      })
      .toSorted((left, right) => (right.roleFit ?? -1) - (left.roleFit ?? -1) || left.name.localeCompare(right.name));
  }, [age, clubById, contract, fit, interest, knowledge, nation, position, query, recommendation, role, snapshot.managedClubId, snapshot.players, transferType, value, wage]);

  if (snapshot.status.state !== "connected") {
    return <main className="screen"><LiveDataState snapshot={snapshot} title="Scout Room" checking={checking} onRefresh={onRefresh} /></main>;
  }

  return (
    <main className="screen recruitment-hub scout-room">
      <div className="planner-heading">
        <div><h1>Scout Room</h1><p>Player database, squad evidence, shortlist and recruitment decisions in one workspace.</p></div>
        <div className="database-scope-badge"><Database />{snapshot.status.databasePlayersIndexed} indexed · {snapshot.status.visiblePlayersLoaded} visible</div>
      </div>

      <section className="recruitment-command-bar scout-command-bar" aria-label="Scout Room filters">
        <div className="recruitment-search"><Search /><Input aria-label="Search players" placeholder="Search name, club, nation or position…" value={query} onChange={(event) => setQuery(event.target.value)} /></div>
        <Filter label="Position" value={position} options={positions} onChange={setPosition} />
        <Filter label="Nation" value={nation} options={nations} onChange={setNation} />
        <Filter label="Age" value={age} options={["All", "U21", "22–27", "28+", "Unknown"]} onChange={setAge} />
        <Filter label="Role" value={role} options={roles} onChange={setRole} />
        <Filter label="Tactical fit" value={fit} options={["All", "Strong", "Medium", "Low", "Tactic unavailable"]} onChange={setFit} />
        <Filter label="Interest" value={interest} options={["All", "Very interested", "Interested", "Slightly interested", "Doubtful", "Not interested", "Unknown"]} onChange={setInterest} />
        <Filter label="Knowledge" value={knowledge} options={["All", "fully known", "partly known", "unknown"]} onChange={setKnowledge} />
        <Filter label="Contract" value={contract} options={["All", "Known", "Unknown"]} onChange={setContract} />
        <Filter label="Wage" value={wage} options={["All", "Known", "Unknown"]} onChange={setWage} />
        <Filter label="Value" value={value} options={["All", "Known", "Unknown"]} onChange={setValue} />
        <Filter label="Transfer type" value={transferType} options={["All", "Transfer", "Loan", "Not available", "Unknown"]} onChange={setTransferType} />
        <Filter label="Recommendation" value={recommendation} options={["All", "Current squad", "Unknown"]} onChange={setRecommendation} />
      </section>

      <section className="recruitment-board scout-room-board">
        <header><span>Player</span><span>Role evidence</span><span>Knowledge</span><span>Interest</span><span>Financials</span><span>Availability</span><span>Recommendation</span><span>Actions</span></header>
        {players.map((player) => {
          const club = player.clubId ? clubById.get(player.clubId) : null;
          const favorite = favoriteIds.has(player.id);
          return (
            <article key={player.id}>
              <div className="target-identity">
                <PlayerFace playerId={player.id} name={player.name} size="sm" />
                <div><strong>{player.name}</strong><small>{player.age ?? "Age unknown"} · {player.nationality ?? "Nation unknown"} · {player.positions.join(" / ") || "Position unknown"}</small><b>{club?.name ?? "Club unknown"}</b></div>
              </div>
              <div className="scout-fit-cell"><RatingRing value={player.roleFit} /><span><strong>{player.bestRole ?? "Unknown"}</strong><small>{player.tacticalFit == null ? "Tactic fit unavailable" : `${player.tacticalFit}% tactic fit`}</small></span></div>
              <div><strong>{knowledgeLabel(player)}</strong><small>{player.scoutConfidence == null ? "Confidence unknown" : `${player.scoutConfidence}% confidence`}</small></div>
              <div><strong>{interestLabel(player)}</strong><small>Realism unknown</small></div>
              <div><strong>{player.value ?? "Value unknown"}</strong><small>{player.wage ?? "Wage unknown"}</small></div>
              <div><strong>{availabilityLabel(player)}</strong><small>{player.contractStatus ?? "Contract unknown"}</small></div>
              <div><strong>{player.clubId === snapshot.managedClubId ? "Current squad" : "No recommendation"}</strong><small>Visible evidence only</small></div>
              <div className="target-actions">
                <Button variant="outline" size="icon-sm" aria-label={`Open ${player.name}`} onClick={() => onOpenPlayer(player.id)}><ExternalLink /></Button>
                <Button variant={favorite ? "secondary" : "outline"} size="icon-sm" aria-label={`${favorite ? "Remove" : "Add"} ${player.name} shortlist`} onClick={() => onToggleFavorite(player.id)}><Star /></Button>
              </div>
            </article>
          );
        })}
        {!players.length ? <div className="recruitment-zero-state"><ShieldCheck /><h2>No players match these filters</h2><p>Clear one or more filters to return to the visibility-safe player set.</p></div> : null}
      </section>

      {snapshot.status.backgroundPlayersIndexed > 0 ? (
        <p className="knowledge-gate-note">{snapshot.status.backgroundPlayersIndexed} wider-save records are indexed but remain hidden until FM26 scouting visibility is validated.</p>
      ) : null}
    </main>
  );
}
