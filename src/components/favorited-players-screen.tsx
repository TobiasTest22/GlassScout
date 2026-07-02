"use client";

import { useEffect, useMemo, useState } from "react";
import { Search, Star, Trash2, Users } from "lucide-react";
import { indexedPlayersByIds, type IndexedPlayerSearchResult, type LiveFootballSnapshot, type LivePlayer } from "@/domain/adapters";
import { resolveFavorites, type FavoriteRecord } from "@/domain/live-data";
import { LiveDataState } from "@/components/live-data-state";
import { PlayerFace } from "@/components/player-face";
import { ConfidenceRing } from "@/components/confidence-ring";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

type FavoriteSort = "name" | "age" | "roleFit" | "tacticalFit";

function indexedFavorite(result: IndexedPlayerSearchResult): LivePlayer {
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

export function FavoritedPlayersScreen({
  snapshot,
  favorites,
  checking,
  onRefresh,
  onToggleFavorite,
  onUpdateNote,
  onOpenPlayer,
}: {
  snapshot: LiveFootballSnapshot;
  favorites: FavoriteRecord[];
  checking: boolean;
  onRefresh: () => Promise<unknown>;
  onToggleFavorite: (playerId: string) => void;
  onUpdateNote: (playerId: string, note: string) => void;
  onOpenPlayer: (playerId: string) => void;
}) {
  const [query, setQuery] = useState("");
  const [sort, setSort] = useState<FavoriteSort>("name");
  const [compareIds, setCompareIds] = useState<string[]>([]);
  const [indexedFavorites, setIndexedFavorites] = useState<IndexedPlayerSearchResult[]>([]);

  useEffect(() => {
    if (snapshot.status.state !== "connected") return;
    indexedPlayersByIds(favorites.map((favorite) => favorite.playerId))
      .then(setIndexedFavorites)
      .catch(() => setIndexedFavorites([]));
  }, [favorites, snapshot.status.lastSync, snapshot.status.state]);

  const allFavorites = useMemo(() => {
    const merged = new Map(snapshot.players.map((player) => [player.id, player]));
    for (const result of indexedFavorites) if (!merged.has(result.id)) merged.set(result.id, indexedFavorite(result));
    return [...merged.values()];
  }, [indexedFavorites, snapshot.players]);
  const resolved = useMemo(() => resolveFavorites(favorites, allFavorites), [allFavorites, favorites]);
  const visible = useMemo(() => resolved
    .filter(({ player }) => player.name.toLowerCase().includes(query.trim().toLowerCase()))
    .toSorted((a, b) => {
      if (sort === "name") return a.player.name.localeCompare(b.player.name);
      if (sort === "age") return (a.player.age ?? Number.MAX_SAFE_INTEGER) - (b.player.age ?? Number.MAX_SAFE_INTEGER);
      return (b.player[sort] ?? -1) - (a.player[sort] ?? -1);
    }), [query, resolved, sort]);

  const toggleCompare = (playerId: string) => {
    setCompareIds((current) => current.includes(playerId)
      ? current.filter((id) => id !== playerId)
      : current.length < 2 ? [...current, playerId] : [current[1], playerId]);
  };

  if (snapshot.status.state !== "connected") {
    return <main className="screen"><LiveDataState snapshot={snapshot} title="Shortlist" checking={checking} onRefresh={onRefresh} /></main>;
  }

  return (
    <main className="screen favorites-screen">
      <div className="planner-heading">
        <div><h1>Shortlist</h1><p>Live indexed identities with club-visible evidence and local decision notes.</p></div>
        <div className="live-source-label"><Star />{resolved.length} shortlisted</div>
      </div>
      <section className="favorites-source-note">FM26’s own shortlist relation is not mapped safely for this exact build. Local entries resolve against the current live player index; unknown evidence stays unknown.</section>
      <section className="favorites-toolbar">
        <div className="recruitment-search"><Search /><Input aria-label="Filter shortlisted players" placeholder="Filter shortlist…" value={query} onChange={(event) => setQuery(event.target.value)} /></div>
        <label>Sort by<select value={sort} onChange={(event) => setSort(event.target.value as FavoriteSort)}><option value="name">Name</option><option value="age">Age</option><option value="roleFit">Role fit</option><option value="tacticalFit">Tactical fit</option></select></label>
        <span><Users />Select up to two players to compare</span>
      </section>

      {visible.length ? (
        <section className="favorites-table">
          <header><span>Compare</span><span>Player</span><span>Knowledge</span><span>Role & fit</span><span>Form & market</span><span>Shortlist note</span><span>Remove</span></header>
          {visible.map(({ player, note }) => (
            <article key={player.id} onClick={() => onOpenPlayer(player.id)}>
              <input type="checkbox" aria-label={`Compare ${player.name}`} checked={compareIds.includes(player.id)} onClick={(event) => event.stopPropagation()} onChange={() => toggleCompare(player.id)} />
              <div className="favorite-player"><PlayerFace playerId={player.id} name={player.name} size="sm" highResolution /><span><strong>{player.name}</strong><small>{player.age ?? "Age unknown"} · {player.nationality ?? "Nation unknown"} · {player.positions.join(" / ") || "Position unknown"}</small></span></div>
              <div className="favorite-knowledge">{player.scoutConfidence == null ? <span className="unknown-confidence-ring"><strong>—</strong></span> : <ConfidenceRing value={player.scoutConfidence} />}<span><strong>{(player.scoutKnowledge ?? "unknown").replaceAll("_", " ")}</strong><small>{player.scoutConfidence == null ? "Confidence unknown" : `${player.scoutConfidence}% confidence`}</small></span></div>
              <div><strong>{player.bestRole ?? "Role not evaluated"}</strong><small>{player.roleFit == null ? "Fit unknown" : `${player.roleFit}% role fit`}</small></div>
              <div><strong>{player.form ?? "Form unknown"}</strong><small>{player.value ?? "Value unknown"} · {player.wage ?? "Wage unknown"} · {player.contractStatus ?? "Contract unknown"}</small></div>
              <Input aria-label={`Note for ${player.name}`} value={note} placeholder="Reason for shortlisting…" onChange={(event) => onUpdateNote(player.id, event.target.value)} />
              <Button variant="ghost" size="icon-sm" aria-label={`Remove ${player.name} from shortlist`} onClick={(event) => { event.stopPropagation(); onToggleFavorite(player.id); }}><Trash2 /></Button>
            </article>
          ))}
        </section>
      ) : (
        <section className="favorites-empty"><Star /><h2>No shortlisted players</h2><p>Add players from Scout Room. Their profile opens from the active save when enough data is mapped.</p></section>
      )}

      {compareIds.length === 2 ? (
        <section className="favorite-comparison">
          <h2>Comparison</h2>
          {compareIds.map((id) => {
            const player = allFavorites.find((item) => item.id === id);
            return player ? <article key={id}><strong>{player.name}</strong><span>{player.bestRole ?? "Role unknown"}</span><span>{player.roleFit == null ? "Role fit unknown" : `${player.roleFit}% role fit`}</span><span>{player.tacticalFit == null ? "Tactical fit unknown" : `${player.tacticalFit}% tactical fit`}</span></article> : null;
          })}
        </section>
      ) : null}
    </main>
  );
}
