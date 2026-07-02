"use client";

import { useMemo } from "react";
import { ExternalLink, ShieldCheck, Target, UsersRound } from "lucide-react";
import type { LiveFootballSnapshot, LivePlayer } from "@/domain/adapters";
import { groupSquad, positionGroups } from "@/domain/live-data";
import { LiveDataState } from "@/components/live-data-state";
import { PlayerFace } from "@/components/player-face";
import { Button } from "@/components/ui/button";

function scoreTone(value: number | null | undefined) {
  if (value == null) return "unknown";
  if (value >= 70) return "strong";
  if (value >= 50) return "medium";
  return "poor";
}

function FitRing({ value }: { value: number | null | undefined }) {
  const safeValue = value == null ? 0 : Math.max(0, Math.min(100, value));
  return (
    <span
      className={`fit-score-ring fit-score-${scoreTone(value)}`}
      style={{ "--fit-score": `${safeValue * 3.6}deg` } as React.CSSProperties}
      aria-label={value == null ? "Role evidence unavailable" : `${value} out of 100 role evidence`}
    >
      <strong>{value ?? "—"}</strong>
    </span>
  );
}

function valueOrUnknown(value: string | number | null | undefined) {
  return value == null || value === "" ? "Unknown" : value;
}

function PlayerRow({ player, onOpenPlayer }: { player: LivePlayer; onOpenPlayer: (playerId: string) => void }) {
  return (
    <article className="squad-player-row squad-player-row-rich">
      <div className="squad-player-identity">
        <PlayerFace playerId={player.id} name={player.name} size="sm" />
        <span><button className="player-name-link" onClick={() => onOpenPlayer(player.id)}>{player.name}</button><small>{valueOrUnknown(player.nationality)} · {valueOrUnknown(player.age)} yrs</small></span>
      </div>
      <span><strong>{player.positions.join(" / ") || "Unknown"}</strong><small>{player.preferredFoot ? `${player.preferredFoot} foot` : "Foot unknown"}</small></span>
      <span><strong>{valueOrUnknown(player.bestRole)}</strong><small>Visible-attribute role</small></span>
      <span className="squad-fit-cell"><FitRing value={player.roleFit} /><small>Role evidence</small></span>
      <span><strong>{valueOrUnknown(player.averageRating?.toFixed(2))}</strong><small>Current form</small></span>
      <span><strong>{valueOrUnknown(player.contractStatus)}</strong><small>{valueOrUnknown(player.wage)} wage</small></span>
      <span><strong>{valueOrUnknown(player.value)}</strong><small>Valuation</small></span>
      <span><strong>Unknown</strong><small>Fitness / injury</small></span>
      <span><strong>{valueOrUnknown(player.squadImportance)}</strong><small>Squad status</small></span>
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
  const mappedAttributes = squad.reduce((sum, player) => sum + Object.keys(player.attributes ?? {}).length, 0);

  if (snapshot.status.state !== "connected" || !snapshot.managedClubId || squad.length === 0) {
    return <main className="screen"><LiveDataState snapshot={snapshot} title="Squad" checking={checking} onRefresh={onRefresh} /></main>;
  }

  return (
    <main className="screen my-team-screen">
      <div className="planner-heading">
        <div><h1>Squad</h1><p>{managedClub?.name} · {snapshot.season ?? "Active FM26 save"} · {squad.length} current players</p></div>
        <div className="live-source-label"><span className="live-dot" />Live FM26</div>
      </div>

      <section className="squad-summary-grid squad-summary-grid-verified">
        <article><UsersRound /><span><small>Current squad</small><strong>{squad.length}</strong></span></article>
        <article><ShieldCheck /><span><small>Fully known</small><strong>{squad.filter((player) => player.scoutKnowledge === "fully_known").length}</strong></span></article>
        <article><Target /><span><small>Visible attributes read</small><strong>{mappedAttributes}</strong></span></article>
      </section>

      <section className="squad-live-table squad-live-table-rich">
        <header><span>Player</span><span>Position</span><span>Best role</span><span>Rating</span><span>Form</span><span>Contract / wage</span><span>Value</span><span>Condition</span><span>Status</span><span>Details</span></header>
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
        <h2>Data coverage</h2>
        <div>
          <article><strong>Verified now</strong><p>FM unique ID, age, date of birth, nationality, positions, preferred foot and visible 1–20 attributes.</p></article>
          <article><strong>Rating method</strong><p>The circular score uses position-relevant visible attributes only. It is not hidden current ability and is not tactic fit without a decoded tactic.</p></article>
          <article><strong>Current tactic</strong><p>{snapshot.tactic?.formation ?? "Import an FMF tactic to add formation-specific fit."}</p></article>
          <article><strong>Still unknown</strong><p>Form, contracts, wages, valuation, fitness and squad status remain blank until their current-build relationships are validated.</p></article>
        </div>
      </section>
    </main>
  );
}
