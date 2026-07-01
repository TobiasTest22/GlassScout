"use client";

import { useMemo, useState } from "react";
import { Search, Star, Trash2, Users } from "lucide-react";
import type { LiveFootballSnapshot } from "@/domain/adapters";
import { resolveFavorites, type FavoriteRecord } from "@/domain/live-data";
import { LiveDataState } from "@/components/live-data-state";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

type FavoriteSort = "name" | "age" | "roleFit" | "tacticalFit";

export function FavoritedPlayersScreen({
  snapshot,
  favorites,
  checking,
  onRefresh,
  onToggleFavorite,
  onUpdateNote,
}: {
  snapshot: LiveFootballSnapshot;
  favorites: FavoriteRecord[];
  checking: boolean;
  onRefresh: () => Promise<unknown>;
  onToggleFavorite: (playerId: string) => void;
  onUpdateNote: (playerId: string, note: string) => void;
}) {
  const [query, setQuery] = useState("");
  const [sort, setSort] = useState<FavoriteSort>("name");
  const [compareIds, setCompareIds] = useState<string[]>([]);
  const resolved = useMemo(() => resolveFavorites(favorites, snapshot.players), [favorites, snapshot.players]);
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

  if (snapshot.status.state !== "connected" || snapshot.players.length === 0) {
    return <main className="screen"><LiveDataState snapshot={snapshot} title="Favorites / Shortlist" checking={checking} onRefresh={onRefresh} /></main>;
  }

  return (
    <main className="screen favorites-screen">
      <div className="planner-heading">
        <div><h1>Favorites / Shortlist</h1><p>Your working target list, resolved against the latest verified dataset.</p></div>
        <div className="live-source-label"><Star />{resolved.length} live favorites</div>
      </div>
      <section className="favorites-toolbar">
        <div className="recruitment-search"><Search /><Input aria-label="Filter favorite players" placeholder="Filter favorites…" value={query} onChange={(event) => setQuery(event.target.value)} /></div>
        <label>Sort by<select value={sort} onChange={(event) => setSort(event.target.value as FavoriteSort)}><option value="name">Name</option><option value="age">Age</option><option value="roleFit">Role fit</option><option value="tacticalFit">Tactical fit</option></select></label>
        <span><Users />Select up to two players to compare</span>
      </section>

      {visible.length ? (
        <section className="favorites-table">
          <header><span>Compare</span><span>Player</span><span>Latest FM26 data</span><span>Fit</span><span>Favorite note</span><span>Remove</span></header>
          {visible.map(({ player, note }) => (
            <article key={player.id}>
              <input type="checkbox" aria-label={`Compare ${player.name}`} checked={compareIds.includes(player.id)} onChange={() => toggleCompare(player.id)} />
              <div><strong>{player.name}</strong><small>{player.age ?? "Age unknown"} · {player.positions.join(" / ")} · {player.bestRole ?? "Role unknown"}</small></div>
              <div><strong>{player.form ?? "Form unknown"}</strong><small>{player.value ?? "Value unknown"} · {player.contractStatus ?? "Contract unknown"}</small></div>
              <div><strong>{player.roleFit == null ? "—" : `${player.roleFit}% role`}</strong><small>{player.tacticalFit == null ? "Tactical —" : `${player.tacticalFit}% tactical`}</small></div>
              <Input aria-label={`Note for ${player.name}`} value={note} placeholder="Reason for favoriting…" onChange={(event) => onUpdateNote(player.id, event.target.value)} />
              <Button variant="ghost" size="icon-sm" aria-label={`Remove ${player.name} favorite`} onClick={() => onToggleFavorite(player.id)}><Trash2 /></Button>
            </article>
          ))}
        </section>
      ) : (
        <section className="favorites-empty"><Star /><h2>No favorited players</h2><p>Add players from Recruitment. Favorites never create or preserve stale player records; they resolve against the latest live FM26 snapshot.</p></section>
      )}

      {compareIds.length === 2 ? (
        <section className="favorite-comparison">
          <h2>Comparison</h2>
          {compareIds.map((id) => {
            const player = snapshot.players.find((item) => item.id === id);
            return player ? <article key={id}><strong>{player.name}</strong><span>{player.bestRole ?? "Role unknown"}</span><span>{player.roleFit == null ? "Role fit unknown" : `${player.roleFit}% role fit`}</span><span>{player.tacticalFit == null ? "Tactical fit unknown" : `${player.tacticalFit}% tactical fit`}</span></article> : null;
          })}
        </section>
      ) : null}
    </main>
  );
}
