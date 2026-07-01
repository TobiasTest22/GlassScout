"use client";

import { useMemo, useState } from "react";
import { ArrowDownUp, Search, Star, UserCheck } from "lucide-react";
import type { LiveFootballSnapshot } from "@/domain/adapters";
import type { FavoriteRecord } from "@/domain/live-data";
import { LiveDataState } from "@/components/live-data-state";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

type SortKey = "name" | "age" | "roleFit" | "tacticalFit";

export function RecruitmentScreen({
  snapshot,
  favorites,
  checking,
  onRefresh,
  onToggleFavorite,
}: {
  snapshot: LiveFootballSnapshot;
  favorites: FavoriteRecord[];
  checking: boolean;
  onRefresh: () => Promise<unknown>;
  onToggleFavorite: (playerId: string) => void;
}) {
  const [query, setQuery] = useState("");
  const [position, setPosition] = useState("All positions");
  const [interestOnly, setInterestOnly] = useState(false);
  const [transferOnly, setTransferOnly] = useState(false);
  const [loanOnly, setLoanOnly] = useState(false);
  const [sort, setSort] = useState<SortKey>("roleFit");

  const clubById = useMemo(() => new Map(snapshot.clubs.map((club) => [club.id, club])), [snapshot.clubs]);
  const positions = useMemo(
    () => ["All positions", ...new Set(snapshot.players.flatMap((player) => player.positions))],
    [snapshot.players],
  );
  const favoriteIds = useMemo(() => new Set(favorites.map((record) => record.playerId)), [favorites]);
  const players = useMemo(() => {
    const normalizedQuery = query.trim().toLowerCase();
    return snapshot.players
      .filter((player) => {
        const club = player.clubId ? clubById.get(player.clubId) : null;
        const matchesQuery = !normalizedQuery || [player.name, club?.name, ...player.positions]
          .filter(Boolean)
          .some((value) => value?.toLowerCase().includes(normalizedQuery));
        const matchesPosition = position === "All positions" || player.positions.includes(position);
        const matchesInterest = !interestOnly || Boolean(player.transferInterest || player.loanInterest);
        return matchesQuery
          && matchesPosition
          && matchesInterest
          && (!transferOnly || player.transferAvailable === true)
          && (!loanOnly || player.loanAvailable === true);
      })
      .toSorted((a, b) => {
        if (sort === "name") return a.name.localeCompare(b.name);
        if (sort === "age") return (a.age ?? Number.MAX_SAFE_INTEGER) - (b.age ?? Number.MAX_SAFE_INTEGER);
        return (b[sort] ?? -1) - (a[sort] ?? -1);
      });
  }, [clubById, interestOnly, loanOnly, position, query, snapshot.players, sort, transferOnly]);

  if (snapshot.status.state !== "connected" || snapshot.players.length === 0) {
    return <main className="screen"><LiveDataState snapshot={snapshot} title="Recruitment" checking={checking} onRefresh={onRefresh} /></main>;
  }

  return (
    <main className="screen live-recruitment-screen">
      <div className="planner-heading">
        <div><h1>Recruitment</h1><p>Live players extracted from the active FM26 save.</p></div>
        <div className="live-source-label"><span className="live-dot" />Synced {snapshot.status.lastSync}</div>
      </div>

      <section className="recruitment-filters" aria-label="Recruitment filters">
        <div className="recruitment-search"><Search /><Input aria-label="Search recruitment players" placeholder="Search player, club or position…" value={query} onChange={(event) => setQuery(event.target.value)} /></div>
        <label className="filter-check"><input type="checkbox" checked={interestOnly} onChange={(event) => setInterestOnly(event.target.checked)} /><UserCheck />Interest recorded</label>
        <label className="filter-check"><input type="checkbox" checked={transferOnly} onChange={(event) => setTransferOnly(event.target.checked)} />Transfer available</label>
        <label className="filter-check"><input type="checkbox" checked={loanOnly} onChange={(event) => setLoanOnly(event.target.checked)} />Loan available</label>
        <select aria-label="Filter by position" value={position} onChange={(event) => setPosition(event.target.value)}>
          {positions.map((value) => <option key={value}>{value}</option>)}
        </select>
      </section>

      <div className="player-pool-toolbar">
        <span>{players.length} live players</span>
        <label>Sort by
          <select aria-label="Sort recruitment players" value={sort} onChange={(event) => setSort(event.target.value as SortKey)}>
            <option value="roleFit">Role fit</option>
            <option value="tacticalFit">Tactical fit</option>
            <option value="age">Age</option>
            <option value="name">Name</option>
          </select>
        </label>
      </div>

      <section className="live-player-table recruitment-live-table">
        <header><span>Player</span><span>Best role</span><span>Interest</span><span>Availability</span><span>Role fit</span><span>Value / wage</span><span>Favorite</span></header>
        {players.map((player) => {
          const club = player.clubId ? clubById.get(player.clubId) : null;
          return (
            <article key={player.id}>
              <div className="live-player-name"><span>{player.name.split(" ").map((part) => part[0]).join("").slice(0, 2)}</span><div><strong>{player.name}</strong><small>{player.age ?? "Age unknown"} · {player.nationality ?? "Nationality unknown"} · {player.positions.join(" / ")}</small><b>{club?.name ?? "Club unknown"} · {club?.league ?? "League unknown"}</b></div></div>
              <div className="role-fit-cell"><strong>{player.bestRole ?? "Unknown"}</strong><small>FM26 output</small></div>
              <div className="interest-cell"><span>{player.transferInterest ?? "Unknown"}</span><small>Loan: {player.loanInterest ?? "Unknown"}</small></div>
              <div className="availability-cell"><span>{player.transferAvailable === true ? "Transfer" : player.loanAvailable === true ? "Loan" : "Not listed"}</span></div>
              <div className="role-fit-cell"><strong>{player.roleFit == null ? "—" : `${player.roleFit}%`}</strong><small>Tactical {player.tacticalFit == null ? "—" : `${player.tacticalFit}%`}</small></div>
              <div className="money-cell"><strong>{player.value ?? "Unknown"}</strong><small>{player.wage ?? "Wage unknown"}</small></div>
              <Button variant={favoriteIds.has(player.id) ? "secondary" : "outline"} size="icon-sm" aria-label={`${favoriteIds.has(player.id) ? "Remove" : "Add"} ${player.name} favorite`} onClick={() => onToggleFavorite(player.id)}><Star /></Button>
            </article>
          );
        })}
      </section>
      <div className="recruitment-footnote"><ArrowDownUp />All values above come from the current live snapshot; missing fields remain unknown.</div>
    </main>
  );
}
