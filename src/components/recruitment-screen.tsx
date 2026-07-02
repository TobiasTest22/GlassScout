"use client";

import { useEffect, useMemo, useState } from "react";
import { ArrowDownUp, Database, Search, ShieldCheck, Star } from "lucide-react";
import type { IndexedPlayerSearchResult, LiveFootballSnapshot, LivePlayer } from "@/domain/adapters";
import { searchIndexedPlayers } from "@/domain/adapters";
import type { FavoriteRecord } from "@/domain/live-data";
import { LiveDataState } from "@/components/live-data-state";
import { PlayerFace } from "@/components/player-face";
import { ClubLogo } from "@/components/club-logo";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

type SortKey = "name" | "role" | "knowledge" | "interest" | "recommendation";
type SortDirection = "asc" | "desc";

const interestedLabels = new Set(["Very interested", "Interested", "Slightly interested", "Only loan possible", "Transfer listed", "Loan listed"]);

function indexedPlayer(result: IndexedPlayerSearchResult): LivePlayer {
  return {
    id: result.id, name: result.name, age: result.age ?? null, nationality: result.nationality ?? null, positions: result.positions,
    bestRole: result.bestRole ?? null, currentAbility: null, potentialAbility: null, form: result.averageRating == null ? null : result.averageRating.toFixed(2), averageRating: result.averageRating ?? null,
    minutesPlayed: result.minutesPlayed ?? null, goals: result.goals ?? null, assists: result.assists ?? null, contractStatus: result.contractStatus ?? null, value: result.value ?? null, wage: result.wage ?? null,
    squadImportance: null, developmentTrend: null, tacticalFit: null, roleFit: result.roleFit ?? null, strengths: [],
    weaknesses: [], clubId: result.clubId ?? null, clubName: result.clubName ?? null, transferInterest: null, loanInterest: null, transferAvailable: null,
    loanAvailable: null, attributes: {}, scoutKnowledge: result.scoutKnowledge,
    scoutConfidence: result.scoutConfidence, preferredFoot: null,
  };
}

function interestLabel(player: LivePlayer) {
  return player.transferInterest ?? player.loanInterest ?? "Unknown";
}

function knowledgeLabel(player: LivePlayer) {
  return (player.scoutKnowledge ?? "unknown").replaceAll("_", " ");
}

function recommendation(player: LivePlayer) {
  const score = player.tacticalFit ?? player.roleFit;
  if (score == null) return { label: "Not enough evidence", value: "—", sort: -1 };
  const confidence = player.scoutConfidence ?? 0;
  if (confidence >= 95) return { label: score >= 75 ? "Priority review" : score >= 55 ? "Scout further" : "Low priority", value: `${score}`, sort: score };
  const margin = Math.max(4, Math.round((100 - confidence) / 4));
  return { label: "Estimated interval", value: `${Math.max(0, score - margin)}–${Math.min(100, score + margin)}`, sort: score };
}

function RatingRing({ value }: { value: number | null }) {
  const safe = value == null ? 0 : Math.max(0, Math.min(100, value));
  const tone = value == null ? "unknown" : value >= 70 ? "strong" : value >= 50 ? "medium" : "poor";
  return <span className={`fit-score-ring fit-score-${tone}`} style={{ "--fit-score": `${safe * 3.6}deg` } as React.CSSProperties}><strong>{value ?? "—"}</strong></span>;
}

function SortButton({ label, value, current, direction, onSort }: {
  label: string; value: SortKey; current: SortKey; direction: SortDirection; onSort: (value: SortKey) => void;
}) {
  return <button className={current === value ? "sort-heading active" : "sort-heading"} onClick={() => onSort(value)}>{label}<ArrowDownUp />{current === value ? <small>{direction}</small> : null}</button>;
}

export function ScoutRoomScreen({ snapshot, favorites, checking, onRefresh, onToggleFavorite, onOpenPlayer }: {
  snapshot: LiveFootballSnapshot;
  favorites: FavoriteRecord[];
  checking: boolean;
  onRefresh: () => Promise<unknown>;
  onToggleFavorite: (playerId: string) => void;
  onOpenPlayer: (playerId: string) => void;
}) {
  const [query, setQuery] = useState("");
  const [interestedOnly, setInterestedOnly] = useState(false);
  const [position, setPosition] = useState("All");
  const [knowledge, setKnowledge] = useState("All");
  const [indexed, setIndexed] = useState<IndexedPlayerSearchResult[]>([]);
  const [sortKey, setSortKey] = useState<SortKey>("recommendation");
  const [sortDirection, setSortDirection] = useState<SortDirection>("desc");

  useEffect(() => {
    if (snapshot.status.state !== "connected") return;
    const timer = window.setTimeout(() => {
      searchIndexedPlayers(query).then(setIndexed).catch(() => setIndexed([]));
    }, query ? 180 : 0);
    return () => window.clearTimeout(timer);
  }, [query, snapshot.status.lastSync, snapshot.status.state]);

  const clubById = useMemo(() => new Map(snapshot.clubs.map((club) => [club.id, club])), [snapshot.clubs]);
  const liveById = useMemo(() => new Map(snapshot.players.map((player) => [player.id, player])), [snapshot.players]);
  const favoriteIds = useMemo(() => new Set(favorites.map((record) => record.playerId)), [favorites]);
  const allPlayers = useMemo(() => {
    const merged = new Map(snapshot.players.map((player) => [player.id, player]));
    for (const result of indexed) if (!merged.has(result.id)) merged.set(result.id, indexedPlayer(result));
    return [...merged.values()];
  }, [indexed, snapshot.players]);
  const positions = useMemo(() => ["All", ...new Set(allPlayers.flatMap((player) => player.positions))], [allPlayers]);

  const players = useMemo(() => {
    const normalized = query.trim().toLowerCase();
    const result = allPlayers.filter((player) => {
      const club = player.clubId ? clubById.get(player.clubId) : null;
      const clubName = club?.name ?? player.clubName;
      const matchesQuery = !normalized || [player.name, clubName, player.nationality, ...player.positions].filter(Boolean).some((item) => item?.toLowerCase().includes(normalized));
      return matchesQuery
        && (!interestedOnly || interestedLabels.has(interestLabel(player)))
        && (position === "All" || player.positions.includes(position))
        && (knowledge === "All" || knowledgeLabel(player) === knowledge);
    });
    const direction = sortDirection === "asc" ? 1 : -1;
    return result.toSorted((left, right) => {
      const values: Record<SortKey, [string | number, string | number]> = {
        name: [left.name, right.name],
        role: [left.bestRole ?? "", right.bestRole ?? ""],
        knowledge: [left.scoutConfidence ?? -1, right.scoutConfidence ?? -1],
        interest: [interestLabel(left), interestLabel(right)],
        recommendation: [recommendation(left).sort, recommendation(right).sort],
      };
      const [a, b] = values[sortKey];
      return (typeof a === "number" && typeof b === "number" ? a - b : String(a).localeCompare(String(b))) * direction || left.name.localeCompare(right.name);
    });
  }, [allPlayers, clubById, interestedOnly, knowledge, position, query, sortDirection, sortKey]);

  const onSort = (value: SortKey) => {
    if (sortKey === value) setSortDirection((current) => current === "asc" ? "desc" : "asc");
    else { setSortKey(value); setSortDirection(value === "name" ? "asc" : "desc"); }
  };

  if (snapshot.status.state !== "connected") {
    return <main className="screen"><LiveDataState snapshot={snapshot} title="Scout Room" checking={checking} onRefresh={onRefresh} /></main>;
  }

  return (
    <main className="screen recruitment-hub scout-room">
      <div className="planner-heading">
        <div><h1>Scout Room</h1><p>Search the indexed save, then judge only the evidence your club actually knows.</p></div>
        <div className="database-scope-badge"><Database />{snapshot.status.databasePlayersIndexed} indexed</div>
      </div>

      <section className="scout-room-toolbar">
        <div className="recruitment-search"><Search /><Input aria-label="Search players" placeholder="Search any indexed player…" value={query} onChange={(event) => setQuery(event.target.value)} /></div>
        <label className="interest-switch"><span>Interested in joining</span><button role="switch" aria-checked={interestedOnly} className={interestedOnly ? "on" : ""} onClick={() => setInterestedOnly((value) => !value)}><i /></button><small>{interestedOnly ? "On" : "Off"}</small></label>
        <label className="scout-filter"><span>Position</span><select value={position} onChange={(event) => setPosition(event.target.value)}>{positions.map((item) => <option key={item}>{item}</option>)}</select></label>
        <label className="scout-filter"><span>Knowledge</span><select value={knowledge} onChange={(event) => setKnowledge(event.target.value)}>{["All", "fully known", "partly known", "unknown"].map((item) => <option key={item}>{item}</option>)}</select></label>
      </section>

      <section className="recruitment-board scout-room-board">
        <header>
          <SortButton label="Player" value="name" current={sortKey} direction={sortDirection} onSort={onSort} />
          <SortButton label="Role evidence" value="role" current={sortKey} direction={sortDirection} onSort={onSort} />
          <SortButton label="Knowledge" value="knowledge" current={sortKey} direction={sortDirection} onSort={onSort} />
          <SortButton label="Interest" value="interest" current={sortKey} direction={sortDirection} onSort={onSort} />
          <span>Form & market</span><SortButton label="Recommendation" value="recommendation" current={sortKey} direction={sortDirection} onSort={onSort} /><span>Shortlist</span>
        </header>
        {players.map((player) => {
          const club = player.clubId ? clubById.get(player.clubId) : null;
          const favorite = favoriteIds.has(player.id);
          const rec = recommendation(player);
          const hasLiveDossier = liveById.has(player.id);
          const clubName = club?.name ?? player.clubName;
          return (
            <article key={player.id} className={hasLiveDossier ? "" : "indexed-player-row"} onClick={() => onOpenPlayer(player.id)}>
              <div className="target-identity"><PlayerFace playerId={player.id} name={player.name} size="sm" highResolution /><div><strong>{player.name}</strong><small>{player.age ?? "Age unknown"} · {player.nationality ?? "Nation unknown"} · {player.positions.join(" / ") || "Position unknown"} · FM ID {player.id}</small><b className="target-club">{club ? <ClubLogo clubId={club.id} name={club.name} size="sm" /> : null}{clubName ?? "Club not mapped"}</b></div></div>
              <div className="scout-fit-cell"><RatingRing value={player.roleFit} /><span><strong>{player.bestRole ?? "Not evaluated"}</strong><small>{player.tacticalFit == null ? "Live tactic required" : `${player.tacticalFit}% tactic fit`}</small></span></div>
              <div><strong>{knowledgeLabel(player)}</strong><small>{player.scoutConfidence == null ? "Confidence unknown" : `${player.scoutConfidence}% confidence`}</small></div>
              <div><strong>{interestLabel(player)}</strong><small>{interestLabel(player) === "Unknown" ? "Not mapped for this build" : "From FM26"}</small></div>
              <div className="form-market-cell"><span className="form-line"><i style={{ width: player.averageRating ? `${Math.max(5, (player.averageRating - 5) * 40)}%` : "0%" }} /></span><strong>{player.averageRating?.toFixed(2) ?? "Form unknown"}</strong><small>{player.value ?? "Value unknown"} · {player.wage ?? "Wage unknown"}</small></div>
              <div><strong>{rec.value}</strong><small>{rec.label} · {player.scoutConfidence ?? 0}% evidence</small></div>
              <div className="target-actions"><Button variant="outline" size="icon-sm" className={favorite ? "shortlist-active" : ""} aria-label={`${favorite ? "Remove" : "Add"} ${player.name} shortlist`} onClick={(event) => { event.stopPropagation(); onToggleFavorite(player.id); }}><Star fill={favorite ? "currentColor" : "none"} /></Button></div>
            </article>
          );
        })}
        {!players.length ? <div className="recruitment-zero-state"><ShieldCheck /><h2>{interestedOnly ? "No validated interest records yet" : "No players match this search"}</h2><p>{interestedOnly ? "The filter is on by default. Interest is not mapped safely for this FM26 build, so GlassScout will not invent viable targets. Turn it off to browse the indexed database." : "Try another name or clear the filters."}</p></div> : null}
      </section>
      <p className="knowledge-gate-note">Indexed players may appear with identity-only records. Unknown fields stay masked and are never used as exact recommendation evidence.</p>
    </main>
  );
}
