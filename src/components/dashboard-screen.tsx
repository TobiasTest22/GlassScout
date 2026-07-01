"use client";

import { ArrowRight, Database, RefreshCw, Target, TrendingUp, UserRoundSearch, UsersRound } from "lucide-react";
import { motion } from "framer-motion";
import { Button } from "@/components/ui/button";
import type { Screen } from "@/components/app-sidebar";
import type { LiveFootballSnapshot } from "@/domain/adapters";
import { groupSquad, positionGroups } from "@/domain/live-data";
import { LiveDataState } from "@/components/live-data-state";

export function DashboardScreen({
  snapshot,
  checking,
  onRefresh,
  onNavigate,
}: {
  snapshot: LiveFootballSnapshot;
  checking: boolean;
  onRefresh: () => Promise<unknown>;
  onNavigate: (screen: Screen) => void;
}) {
  if (snapshot.status.state !== "connected" || !snapshot.managedClubId) {
    return <motion.main className="screen dashboard-screen" initial={false} animate={{ opacity: 1 }}><LiveDataState snapshot={snapshot} title="Dashboard" checking={checking} onRefresh={onRefresh} /></motion.main>;
  }

  const club = snapshot.clubs.find((item) => item.id === snapshot.managedClubId);
  const squad = snapshot.players.filter((player) => player.clubId === snapshot.managedClubId);
  const groups = groupSquad(squad);
  const fittedPlayers = squad.filter((player) => player.tacticalFit != null);
  const averageFit = fittedPlayers.length ? Math.round(fittedPlayers.reduce((sum, player) => sum + (player.tacticalFit ?? 0), 0) / fittedPlayers.length) : null;
  const weakest = squad.filter((player) => player.roleFit != null).toSorted((a, b) => (a.roleFit ?? 0) - (b.roleFit ?? 0)).slice(0, 4);
  const retraining = squad.filter((player) => player.retrainingSuggestion).toSorted((a, b) => (b.roleFit ?? 0) - (a.roleFit ?? 0)).slice(0, 4);
  const undervalued = squad.filter((player) => player.valuationLabel === "undervalued").toSorted((a, b) => (b.truePrice ?? 0) - (a.truePrice ?? 0)).slice(0, 4);
  const recruitmentNeeds = positionGroups.filter((group) => (groups.get(group)?.length ?? 0) < 2);

  return (
    <motion.main className="screen dashboard-screen" initial={false} animate={{ opacity: 1 }}>
      <div className="page-intro">
        <div><h1>{club?.name ?? "Managed team"}</h1><p>{snapshot.season ?? "Active FM26 save"} · Live game data</p></div>
        <div className="heading-actions"><Button variant="outline" onClick={onRefresh} disabled={checking}><RefreshCw data-icon="inline-start" />Refresh</Button><Button onClick={() => onNavigate("Recruitment")}>Open recruitment<ArrowRight data-icon="inline-end" /></Button></div>
      </div>

      <section className="live-overview-grid dashboard-overview-rich">
        <button onClick={() => onNavigate("My Team")}><UsersRound /><span><small>Current squad</small><strong>{squad.length} players</strong></span></button>
        <button onClick={() => onNavigate("Tactic Evaluation")}><Target /><span><small>Current tactic fit</small><strong>{averageFit == null ? "Unavailable" : `${averageFit}%`}</strong></span></button>
        <button onClick={() => onNavigate("Settings")}><Database /><span><small>Last live sync</small><strong>{snapshot.status.lastSync ? new Date(snapshot.status.lastSync).toLocaleTimeString() : "Not synced"}</strong></span></button>
        <button onClick={() => onNavigate("Role DNA")}><TrendingUp /><span><small>Retraining candidates</small><strong>{retraining.length}</strong></span></button>
      </section>

      <section className="dashboard-intelligence">
        <article><header><Target /><h2>Weakest roles</h2></header>{weakest.length ? weakest.map((player) => <span key={player.id}><strong>{player.name}</strong><small>{player.bestRole ?? "Role unavailable"} · {player.roleFit}%</small></span>) : <p>Role evidence is unavailable.</p>}</article>
        <article><header><TrendingUp /><h2>Top retraining candidates</h2></header>{retraining.length ? retraining.map((player) => <span key={player.id}><strong>{player.name}</strong><small>{player.bestCalculatedPosition} · {player.roleFit}% DNA</small></span>) : <p>No evidence-backed retraining signal.</p>}</article>
        <article><header><UserRoundSearch /><h2>Recruitment priorities</h2></header>{recruitmentNeeds.length ? recruitmentNeeds.slice(0, 4).map((group) => <span key={group}><strong>{group}</strong><small>{groups.get(group)?.length ?? 0} readable options</small></span>) : <p>At least two readable options exist in every position group.</p>}</article>
        <article><header><Database /><h2>Undervalued players</h2></header>{undervalued.length ? undervalued.map((player) => <span key={player.id}><strong>{player.name}</strong><small>{player.value} FM value · €{player.truePrice}m estimate</small></span>) : <p>No transparent undervaluation signal in available data.</p>}</article>
      </section>

      {snapshot.dataWarnings?.length ? <section className="dashboard-data-warnings"><h2>Data warnings</h2>{snapshot.dataWarnings.map((warning) => <p key={warning}>{warning}</p>)}</section> : null}
    </motion.main>
  );
}
