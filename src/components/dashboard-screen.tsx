"use client";

import {
  ArrowRight,
  Binoculars,
  ClipboardCheck,
  Database,
  FileWarning,
  IdCard,
  SearchCheck,
  ShieldCheck,
  UsersRound,
} from "lucide-react";
import { motion } from "framer-motion";
import type { Screen } from "@/components/app-sidebar";
import { LiveDataState } from "@/components/live-data-state";
import { TacticalBoard } from "@/components/tactical-board";
import { Button } from "@/components/ui/button";
import type { LiveFootballSnapshot } from "@/domain/adapters";
import { groupSquad, positionGroups } from "@/domain/live-data";

function percent(numerator: number, denominator: number) {
  if (!denominator) return 0;
  return Math.min(100, Math.round((numerator / denominator) * 100));
}

function HealthRow({
  icon: Icon,
  label,
  value,
  status,
  tone,
}: {
  icon: typeof UsersRound;
  label: string;
  value: number;
  status: string;
  tone: "good" | "attention" | "unknown";
}) {
  return (
    <button className="health-row">
      <span className="health-icon"><Icon /></span>
      <strong>{label}</strong>
      <span className="health-track"><i style={{ width: `${value}%` }} data-tone={tone} /></span>
      <small data-tone={tone}>{status}</small>
      <ArrowRight />
    </button>
  );
}

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
  const groups = groupSquad(squad);
  const healthyGroups = positionGroups.filter((group) => (groups.get(group)?.length ?? 0) >= 2).length;
  const roleDepth = percent(healthyGroups, positionGroups.length);
  const scoutingCoverage = percent(
    snapshot.status.visiblePlayersLoaded,
    snapshot.status.databasePlayersIndexed,
  );
  const externalVisible = snapshot.players.filter((player) => player.clubId !== snapshot.managedClubId);

  const briefing = [
    {
      icon: Database,
      title: `${snapshot.status.databasePlayersIndexed} player records indexed`,
      detail: `${snapshot.status.backgroundPlayersIndexed} wider-save records remain behind the knowledge gate.`,
      action: "Review scope",
      screen: "Settings" as Screen,
    },
    {
      icon: snapshot.tactic ? ClipboardCheck : FileWarning,
      title: snapshot.tactic ? `${snapshot.tactic.formation} tactic ready` : "Tactical decoding needs attention",
      detail: snapshot.status.liveMemoryTacticRead === "object_detected_unmapped"
        ? "The active live tactic object is detected; its packed formation and phase-role layout is not yet validated."
        : "Open FM26 and select the active tactic so GlassScout can inspect the live object.",
      action: "Open board",
      screen: "Tactical Board" as Screen,
    },
    {
      icon: ShieldCheck,
      title: `${squad.length} managed-club players visibility-safe`,
      detail: "IDs, ages, nations, positions, feet and visible attributes are live; unsupported fields remain Unknown.",
      action: "Open squad",
      screen: "Squad" as Screen,
    },
  ];

  return (
    <motion.main className="screen dashboard-screen dashboard-command" initial={false} animate={{ opacity: 1 }}>
      <div className="dashboard-command-grid">
        <TacticalBoard snapshot={snapshot} compact />

        <section className="command-panel recruitment-pulse">
          <header><h2>Scout Room pulse</h2><span>{externalVisible.length} live targets</span></header>
          <div className="recruitment-pulse-empty">
            <span className="target-avatar"><SearchCheck /></span>
            <div>
              <strong>No visibility-safe external targets yet</strong>
              <p>
                {snapshot.status.backgroundPlayersIndexed
                  ? `${snapshot.status.backgroundPlayersIndexed} wider-save records are indexed but hidden until club knowledge is mapped.`
                  : "Load the active save to build the wider player index."}
              </p>
            </div>
          </div>
          <Button variant="outline" onClick={() => onNavigate("Scout Room")}>
            <SearchCheck data-icon="inline-start" />Plan recruitment
          </Button>
        </section>

        <section className="command-panel department-briefing">
          <header><h2>Department briefing</h2><span>{club?.name ?? "Active club"}</span></header>
          <div>
            {briefing.map(({ icon: Icon, title, detail, action, screen }) => (
              <button key={title} className="briefing-row" onClick={() => onNavigate(screen)}>
                <span className="briefing-icon"><Icon /></span>
                <span><strong>{title}</strong><small>{detail}</small></span>
                <b>{action}</b>
                <ArrowRight />
              </button>
            ))}
          </div>
        </section>

        <section className="command-panel squad-health">
          <header><h2>Squad health</h2><span>Live evidence</span></header>
          <div>
            <HealthRow icon={UsersRound} label="Role depth" value={roleDepth} status={roleDepth >= 70 ? "Good" : "Attention"} tone={roleDepth >= 70 ? "good" : "attention"} />
            <HealthRow icon={ClipboardCheck} label="Contracts" value={0} status="Unknown" tone="unknown" />
            <HealthRow icon={IdCard} label="Registration" value={0} status="Unknown" tone="unknown" />
            <HealthRow icon={Binoculars} label="Scouting coverage" value={scoutingCoverage} status={scoutingCoverage >= 60 ? "Good" : "Restricted"} tone={scoutingCoverage >= 60 ? "good" : "attention"} />
          </div>
        </section>
      </div>
    </motion.main>
  );
}
