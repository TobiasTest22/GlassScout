"use client";

import { useMemo } from "react";
import { Activity, ArrowDown, ArrowUp, ShieldAlert, UsersRound } from "lucide-react";
import type { LiveFootballSnapshot, LivePlayer } from "@/domain/adapters";
import { groupSquad, positionGroups } from "@/domain/live-data";
import { LiveDataState } from "@/components/live-data-state";

function PlayerRow({ player }: { player: LivePlayer }) {
  return (
    <article className="squad-player-row">
      <div><strong>{player.name}</strong><small>{player.age ?? "Age unknown"} · {player.nationality ?? "Nationality unknown"}</small></div>
      <span>{player.positions.join(" / ") || "Unknown"}</span>
      <span>{player.bestRole ?? "Unknown"}</span>
      <span>{player.form ?? "—"}<small>{player.averageRating == null ? "Rating —" : `Rating ${player.averageRating.toFixed(2)}`}</small></span>
      <span>{player.minutesPlayed ?? "—"}<small>{player.goals ?? "—"} G · {player.assists ?? "—"} A</small></span>
      <span>{player.contractStatus ?? "Unknown"}<small>{player.value ?? "Value unknown"} · {player.wage ?? "Wage unknown"}</small></span>
      <span>{player.tacticalFit == null ? "—" : `${player.tacticalFit}%`}<small>{player.squadImportance ?? "Importance unknown"}</small></span>
      <span className={`trend-${player.developmentTrend ?? "unknown"}`}>
        {player.developmentTrend === "improving" ? <ArrowUp /> : player.developmentTrend === "declining" ? <ArrowDown /> : <Activity />}
        {player.developmentTrend ?? "Unknown"}
      </span>
    </article>
  );
}

export function MyTeamScreen({
  snapshot,
  checking,
  onRefresh,
}: {
  snapshot: LiveFootballSnapshot;
  checking: boolean;
  onRefresh: () => Promise<unknown>;
}) {
  const squad = useMemo(
    () => snapshot.players.filter((player) => player.clubId === snapshot.managedClubId),
    [snapshot.managedClubId, snapshot.players],
  );
  const groups = useMemo(() => groupSquad(squad), [squad]);
  const managedClub = snapshot.clubs.find((club) => club.id === snapshot.managedClubId);

  if (snapshot.status.state !== "connected" || !snapshot.managedClubId || squad.length === 0) {
    return <main className="screen"><LiveDataState snapshot={snapshot} title="My Team" checking={checking} onRefresh={onRefresh} /></main>;
  }

  const improving = squad.filter((player) => player.developmentTrend === "improving").length;
  const declining = squad.filter((player) => player.developmentTrend === "declining").length;
  const expiring = squad.filter((player) => player.contractStatus?.toLowerCase().includes("expir")).length;
  const lowFit = squad.filter((player) => player.tacticalFit != null && player.tacticalFit < 65).length;

  return (
    <main className="screen my-team-screen">
      <div className="planner-heading">
        <div><h1>My Team</h1><p>{managedClub?.name} · {snapshot.season} · {squad.length} current players</p></div>
        <div className="live-source-label"><span className="live-dot" />Current FM26 squad</div>
      </div>

      <section className="squad-summary-grid">
        <article><UsersRound /><span><small>Best XI mapped</small><strong>{snapshot.tactic?.slots.filter((slot) => slot.playerId).length ?? 0} / 11</strong></span></article>
        <article><ArrowUp /><span><small>Improving</small><strong>{improving}</strong></span></article>
        <article><ArrowDown /><span><small>Declining</small><strong>{declining}</strong></span></article>
        <article><ShieldAlert /><span><small>Contracts to review</small><strong>{expiring}</strong></span></article>
        <article><Activity /><span><small>Low tactical fit</small><strong>{lowFit}</strong></span></article>
      </section>

      <section className="squad-live-table">
        <header><span>Player</span><span>Position</span><span>Best role</span><span>Form</span><span>Minutes / output</span><span>Contract / cost</span><span>Tactical fit</span><span>Trend</span></header>
        {positionGroups.map((group) => {
          const players = groups.get(group) ?? [];
          if (players.length === 0) return null;
          return (
            <div className="squad-position-group" key={group}>
              <h2>{group}<span>{players.length}</span></h2>
              {players.map((player) => <PlayerRow key={player.id} player={player} />)}
            </div>
          );
        })}
      </section>

      <section className="squad-analysis-live">
        <h2>Squad analysis</h2>
        <div>
          <article><strong>Best XI</strong><p>{snapshot.tactic ? `${snapshot.tactic.formation} loaded from FM26 with ${snapshot.tactic.slots.filter((slot) => slot.playerId).length} assigned players.` : "Current tactic is not readable."}</p></article>
          <article><strong>Squad depth</strong><p>{positionGroups.map((group) => `${group}: ${groups.get(group)?.length ?? 0}`).join(" · ")}</p></article>
          <article><strong>Development</strong><p>{improving} improving · {declining} declining · {squad.length - improving - declining} stable or unknown.</p></article>
          <article><strong>Recruitment needs</strong><p>{lowFit ? `${lowFit} players currently fall below 65% tactical fit.` : "No low-fit players were identified in readable data."}</p></article>
        </div>
      </section>
    </main>
  );
}
