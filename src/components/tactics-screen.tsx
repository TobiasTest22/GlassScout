"use client";

import { AlertTriangle, Target } from "lucide-react";
import type { LiveFootballSnapshot } from "@/domain/adapters";
import { LiveDataState } from "@/components/live-data-state";

export function TacticsScreen({
  snapshot,
  checking,
  onRefresh,
}: {
  snapshot: LiveFootballSnapshot;
  checking: boolean;
  onRefresh: () => Promise<unknown>;
}) {
  const tactic = snapshot.tactic;
  if (snapshot.status.state !== "connected" || !tactic) {
    return <main className="screen"><LiveDataState snapshot={snapshot} title="Tactics" checking={checking} onRefresh={onRefresh} /></main>;
  }

  const playersById = new Map(snapshot.players.map((player) => [player.id, player]));
  const weakRoles = tactic.slots.filter((slot) => {
    const player = slot.playerId ? playersById.get(slot.playerId) : null;
    return !player || (player.roleFit != null && player.roleFit < 65);
  });

  return (
    <main className="screen tactics-live-screen">
      <div className="planner-heading">
        <div><h1>Current Tactic</h1><p>{tactic.name ?? "Unnamed FM26 tactic"} · {tactic.formation}</p></div>
        <div className="live-source-label"><span className="live-dot" />Read from managed team</div>
      </div>
      <section className="tactic-live-layout">
        <div className="tactic-slots">
          <header><Target /><h2>Formation, roles and duties</h2></header>
          {tactic.slots.map((slot, index) => {
            const player = slot.playerId ? playersById.get(slot.playerId) : null;
            return (
              <article key={`${slot.position}-${index}`}>
                <strong>{slot.position}</strong>
                <span>{player?.name ?? "Unassigned"}</span>
                <span>{slot.role ?? "Role unreadable"}</span>
                <span>{slot.duty ?? "Duty unreadable"}</span>
                <span>{player?.tacticalFit == null ? "Fit unknown" : `${player.tacticalFit}% fit`}</span>
              </article>
            );
          })}
        </div>
        <aside className="tactic-analysis">
          <section><h2>Team instructions</h2>{tactic.teamInstructions.length ? <ul>{tactic.teamInstructions.map((instruction) => <li key={instruction}>{instruction}</li>)}</ul> : <p>Not readable from this FM26 build.</p>}</section>
          <section><h2>Weak roles & conflicts</h2>{weakRoles.length ? <ul>{weakRoles.map((slot, index) => <li key={`${slot.position}-${index}`}><AlertTriangle />{slot.position}: {slot.playerId ? "low role fit" : "no assigned player"}</li>)}</ul> : <p>No role conflict was found in readable data.</p>}</section>
          <section><h2>Suggested improvements</h2><p>Suggestions are generated only from extracted roles, duties, player attributes and team instructions. Missing inputs produce no recommendation.</p></section>
        </aside>
      </section>
    </main>
  );
}
