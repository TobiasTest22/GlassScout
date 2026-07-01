"use client";

import { Dna, RefreshCw } from "lucide-react";
import type { LiveFootballSnapshot } from "@/domain/adapters";
import { LiveDataState } from "@/components/live-data-state";

export function RoleDnaScreen({ snapshot, checking, onRefresh }: { snapshot: LiveFootballSnapshot; checking: boolean; onRefresh: () => Promise<unknown> }) {
  if (snapshot.status.state !== "connected" || !snapshot.players.length) {
    return <main className="screen"><LiveDataState snapshot={snapshot} title="Role DNA / Position Converter" checking={checking} onRefresh={onRefresh} /></main>;
  }
  const players = snapshot.players.toSorted((a, b) => (b.roleFit ?? -1) - (a.roleFit ?? -1));
  return (
    <main className="screen role-dna-screen">
      <div className="planner-heading"><div><h1>Role DNA / Position Converter</h1><p>Ability-based positions from visible technical, mental and physical attributes.</p></div><div className="live-source-label"><Dna />{players.length} players analysed</div></div>
      <section className="role-dna-table">
        <header><span>Player</span><span>Current position</span><span>Best ability-based position</span><span>Best role</span><span>DNA score</span><span>Retraining</span><span>Reasoning</span></header>
        {players.map((player) => (
          <article key={player.id}>
            <strong>{player.name}</strong>
            <span>{player.positions.join(" / ") || "Unknown"}</span>
            <span>{player.bestCalculatedPosition ?? "Unavailable"}</span>
            <span>{player.bestRole ?? "Unavailable"}</span>
            <span>{player.roleFit == null ? "—" : `${player.roleFit}%`}</span>
            <span>{player.retrainingSuggestion ?? "No change suggested"}</span>
            <span>{player.roleReasoning?.join(" · ") || "More visible attributes required"}</span>
          </article>
        ))}
      </section>
      <div className="recruitment-footnote"><RefreshCw />Scores update whenever GlassScout reads a newer live-game snapshot.</div>
    </main>
  );
}
