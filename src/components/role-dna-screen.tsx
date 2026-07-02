"use client";

import { Dna, RefreshCw } from "lucide-react";
import type { LiveFootballSnapshot } from "@/domain/adapters";
import { LiveDataState } from "@/components/live-data-state";

export function RoleDnaScreen({
  snapshot,
  checking,
  onRefresh,
}: {
  snapshot: LiveFootballSnapshot;
  checking: boolean;
  onRefresh: () => Promise<unknown>;
}) {
  if (snapshot.status.state !== "connected" || !snapshot.players.length) {
    return (
      <main className="screen">
        <LiveDataState snapshot={snapshot} title="Role DNA / Position Converter" checking={checking} onRefresh={onRefresh} />
      </main>
    );
  }

  const players = snapshot.players.toSorted((a, b) => (b.roleFit ?? -1) - (a.roleFit ?? -1));

  return (
    <main className="screen role-dna-screen">
      <div className="planner-heading">
        <div>
          <h1>Role DNA / Position Converter</h1>
          <p>FM26 split-fit scoring: in-possession role, out-of-possession role, then combined tactical fit from visible attributes.</p>
        </div>
        <div className="live-source-label"><Dna />{players.length} players analysed</div>
      </div>

      <section className="role-dna-table">
        <header>
          <span>Player</span>
          <span>Position</span>
          <span>Best mapped position</span>
          <span>IP role</span>
          <span>OOP role</span>
          <span>Combined</span>
          <span>Red flags</span>
          <span>Reasoning</span>
        </header>
        {players.map((player) => {
          const best = player.playableRoles?.[0];
          return (
            <article key={player.id}>
              <strong>{player.name}</strong>
              <span>{player.positions.join(" / ") || "Unknown"}</span>
              <span>{player.bestCalculatedPosition ?? "Unavailable"}</span>
              <span>{best?.inPossessionRole ?? "Unavailable"}</span>
              <span>{best?.outOfPossessionRole ?? "Unavailable"}</span>
              <span>{player.roleFit == null ? "—" : `${player.roleFit}%`}</span>
              <span>{best?.redFlags?.slice(0, 2).join(" · ") || "No major flag"}</span>
              <span>{player.roleReasoning?.join(" · ") || "More visible attributes required"}</span>
            </article>
          );
        })}
      </section>

      <div className="recruitment-footnote"><RefreshCw />Scores update whenever GlassScout reads a newer live-game snapshot.</div>
    </main>
  );
}
