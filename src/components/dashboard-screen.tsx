"use client";

import { ArrowRight, Database, Target, UsersRound } from "lucide-react";
import { motion } from "framer-motion";
import { Button } from "@/components/ui/button";
import type { Screen } from "@/components/app-sidebar";
import type { LiveFootballSnapshot } from "@/domain/adapters";
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
    return (
      <motion.main className="screen dashboard-screen" initial={false} animate={{ opacity: 1 }}>
        <LiveDataState snapshot={snapshot} title="Dashboard" checking={checking} onRefresh={onRefresh} />
      </motion.main>
    );
  }

  const club = snapshot.clubs.find((item) => item.id === snapshot.managedClubId);
  const squad = snapshot.players.filter((player) => player.clubId === snapshot.managedClubId);

  return (
    <motion.main className="screen dashboard-screen" initial={false} animate={{ opacity: 1 }}>
      <div className="page-intro">
        <div><h1>{club?.name ?? "Managed team"}</h1><p>{snapshot.managerName} · {snapshot.season}</p></div>
        <Button onClick={() => onNavigate("Recruitment")}>Open recruitment<ArrowRight data-icon="inline-end" /></Button>
      </div>
      <section className="live-overview-grid">
        <button onClick={() => onNavigate("My Team")}><UsersRound /><span><small>Current squad</small><strong>{squad.length} players</strong></span></button>
        <button onClick={() => onNavigate("Tactics")}><Target /><span><small>Current tactic</small><strong>{snapshot.tactic?.formation ?? "Unavailable"}</strong></span></button>
        <button onClick={() => onNavigate("Memory Center")}><Database /><span><small>Last live sync</small><strong>{snapshot.status.lastSync ?? "Not synced"}</strong></span></button>
      </section>
    </motion.main>
  );
}
