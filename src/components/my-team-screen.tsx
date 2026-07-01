"use client";

import { useMemo } from "react";
import { Activity, ArrowDown, ArrowUp, ShieldAlert, UsersRound } from "lucide-react";
import type { LiveFootballSnapshot, LivePlayer } from "@/domain/adapters";
import { groupSquad, positionGroups } from "@/domain/live-data";
import { LiveDataState } from "@/components/live-data-state";

function PlayerRow({ player }: { player: LivePlayer }) {
  const goalsPer90 = player.per90?.goalsPer90;
  const assistsPer90 = player.per90?.assistsPer90;
  return (
    <article className="squad-player-row squad-player-row-rich">
      <div><strong>{player.name}</strong><small>{player.age ?? "Age unknown"} · {player.nationality ?? "Nationality unknown"}</small></div>
      <span>{player.positions.join(" / ") || "Unknown"}<small>Calculated: {player.bestCalculatedPosition ?? "Unavailable"}</small></span>
      <span>{player.bestRole ?? "Unavailable"}<small>{player.roleFit == null ? "Fit unavailable" : `${player.roleFit}% role fit`}</small></span>
      <span>{player.tacticalFit == null ? "—" : `${player.tacticalFit}%`}<small>Tactical fit</small></span>
      <span>{player.value ?? "Unknown"}<small>{player.truePrice == null ? "True price unavailable" : `True price €${player.truePrice}m`}</small></span>
      <span>{player.wage ?? "Unknown"}<small>{player.contractStatus ?? "Contract unknown"}</small></span>
      <span>{goalsPer90 == null ? "G/90 —" : `${goalsPer90} G/90`}<small>{assistsPer90 == null ? "A/90 —" : `${assistsPer90} A/90`} · {player.averageRating == null ? "rating —" : player.averageRating.toFixed(2)}</small></span>
      <span className={`knowledge-${player.scoutKnowledge ?? "unknown"}`}>{(player.scoutKnowledge ?? "unknown").replaceAll("_", " ")}<small>{player.riskLevel ?? "unknown"} data risk</small></span>
      <span>{player.retrainingSuggestion ?? "No retraining signal"}<small>{player.roleReasoning?.join(" · ") || "Insufficient visible attributes"}</small></span>
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
        <div><h1>My Team</h1><p>{managedClub?.name} · {snapshot.season ?? "Imported snapshot"} · {squad.length} current players</p></div>
        <div className="live-source-label"><span className="live-dot" />{snapshot.dataSource === "export-watcher" ? "Export Watcher" : "Live Memory"}</div>
      </div>

      <section className="squad-summary-grid">
        <article><UsersRound /><span><small>Players analysed</small><strong>{squad.length}</strong></span></article>
        <article><ArrowUp /><span><small>Improving</small><strong>{improving}</strong></span></article>
        <article><ArrowDown /><span><small>Declining</small><strong>{declining}</strong></span></article>
        <article><ShieldAlert /><span><small>Contracts to review</small><strong>{expiring}</strong></span></article>
        <article><Activity /><span><small>Low tactical fit</small><strong>{lowFit}</strong></span></article>
      </section>

      <section className="squad-live-table squad-live-table-rich">
        <header><span>Player</span><span>Position</span><span>Best role</span><span>Fit</span><span>Value / true price</span><span>Wage / contract</span><span>Per 90</span><span>Knowledge</span><span>Retraining</span></header>
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
          <article><strong>Best XI</strong><p>{snapshot.tactic ? `${snapshot.tactic.formation} loaded with ${snapshot.tactic.slots.filter((slot) => slot.playerId).length} assigned players.` : "Current tactic is unavailable in this export."}</p></article>
          <article><strong>Role depth</strong><p>{positionGroups.map((group) => `${group}: ${groups.get(group)?.length ?? 0}`).join(" · ")}</p></article>
          <article><strong>Knowledge</strong><p>{squad.filter((player) => player.scoutKnowledge === "fully_known").length} fully known · {squad.filter((player) => player.scoutKnowledge === "needs_scouting").length} need scouting.</p></article>
          <article><strong>Recruitment needs</strong><p>{lowFit ? `${lowFit} players fall below 65% calculated fit.` : "No low-fit players were identified in available visible data."}</p></article>
        </div>
      </section>
    </main>
  );
}
