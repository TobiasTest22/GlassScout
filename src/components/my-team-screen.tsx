"use client";

import { useMemo } from "react";
import { ExternalLink, ShieldCheck, Target, UsersRound } from "lucide-react";
import type { LiveFootballSnapshot, LivePlayer } from "@/domain/adapters";
import { groupSquad, positionGroups } from "@/domain/live-data";
import { LiveDataState } from "@/components/live-data-state";
import { Button } from "@/components/ui/button";

function PlayerRow({ player, onOpenPlayer }: { player: LivePlayer; onOpenPlayer: (playerId: string) => void }) {
  return (
    <article className="squad-player-row squad-player-row-verified">
      <div><strong>{player.name}</strong><small>FM26 ID {player.id}</small></div>
      <span>{player.positions.join(" / ") || "Unknown"}<small>Live positional familiarity</small></span>
      <span>{player.bestCalculatedPosition ?? "Unavailable"}<small>Strongest readable position</small></span>
      <span className={`knowledge-${player.scoutKnowledge ?? "unknown"}`}>{(player.scoutKnowledge ?? "unknown").replaceAll("_", " ")}<small>Managed squad</small></span>
      <Button variant="outline" size="sm" onClick={() => onOpenPlayer(player.id)}>Profile<ExternalLink data-icon="inline-end" /></Button>
    </article>
  );
}

export function MyTeamScreen({
  snapshot,
  checking,
  onRefresh,
  onOpenPlayer,
}: {
  snapshot: LiveFootballSnapshot;
  checking: boolean;
  onRefresh: () => Promise<unknown>;
  onOpenPlayer: (playerId: string) => void;
}) {
  const squad = useMemo(
    () => snapshot.players.filter((player) => player.clubId === snapshot.managedClubId),
    [snapshot.managedClubId, snapshot.players],
  );
  const groups = useMemo(() => groupSquad(squad), [squad]);
  const managedClub = snapshot.clubs.find((club) => club.id === snapshot.managedClubId);

  if (snapshot.status.state !== "connected" || !snapshot.managedClubId || squad.length === 0) {
    return <main className="screen"><LiveDataState snapshot={snapshot} title="Squad Planner" checking={checking} onRefresh={onRefresh} /></main>;
  }

  return (
    <main className="screen my-team-screen">
      <div className="planner-heading">
        <div><h1>Squad Planner</h1><p>{managedClub?.name} · {snapshot.season ?? "Active FM26 save"} · {squad.length} current players</p></div>
        <div className="live-source-label"><span className="live-dot" />Live FM26</div>
      </div>

      <section className="squad-summary-grid squad-summary-grid-verified">
        <article><UsersRound /><span><small>Live squad players</small><strong>{squad.length}</strong></span></article>
        <article><ShieldCheck /><span><small>Names validated</small><strong>{squad.filter((player) => player.name.length > 0).length}</strong></span></article>
        <article><Target /><span><small>Positions validated</small><strong>{squad.filter((player) => player.positions.length > 0).length}</strong></span></article>
      </section>

      <section className="squad-live-table squad-live-table-verified">
        <header><span>Player</span><span>Positions</span><span>Primary</span><span>Knowledge</span><span>Details</span></header>
        {positionGroups.map((group) => {
          const players = groups.get(group) ?? [];
          if (players.length === 0) return null;
          return (
            <div className="squad-position-group" key={group}>
              <h2>{group}<span>{players.length}</span></h2>
              {players.map((player) => <PlayerRow key={player.id} player={player} onOpenPlayer={onOpenPlayer} />)}
            </div>
          );
        })}
      </section>

      <section className="squad-analysis-live">
        <h2>Live data coverage</h2>
        <div>
          <article><strong>Managed club</strong><p>{managedClub?.name} and {squad.length} current squad records are validated directly from the active save.</p></article>
          <article><strong>Current tactic</strong><p>{snapshot.tactic?.name ?? "The current tactic is unavailable."}</p></article>
          <article><strong>Position depth</strong><p>{positionGroups.map((group) => `${group}: ${groups.get(group)?.length ?? 0}`).join(" · ")}</p></article>
          <article><strong>Unavailable fields</strong><p>Age, attributes, roles, form, contracts, wages and valuations are left blank until their live records are mapped safely.</p></article>
        </div>
      </section>
    </main>
  );
}
