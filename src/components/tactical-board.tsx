"use client";

import { useMemo, useState, type CSSProperties } from "react";
import { ShieldAlert, Sparkles } from "lucide-react";
import type { LiveFootballSnapshot, LivePlayer, LiveTacticSlot } from "@/domain/adapters";

type TacticalPhase = "In Possession" | "Out of Possession" | "Combined";

const fallbackCoordinates = [[50, 84], [20, 68], [40, 68], [60, 68], [80, 68], [50, 56], [34, 44], [66, 44], [18, 25], [82, 25], [50, 14]];

const positionCoordinates: Record<string, [number, number]> = {
  GK: [50, 84],
  SW: [50, 75],
  DR: [82, 68],
  DCR: [61, 68],
  DC: [50, 68],
  DCL: [39, 68],
  DL: [18, 68],
  WBR: [86, 58],
  WBL: [14, 58],
  DMR: [62, 56],
  DM: [50, 56],
  DML: [38, 56],
  MR: [82, 44],
  MCR: [62, 44],
  MC: [50, 44],
  MCL: [38, 44],
  ML: [18, 44],
  AMR: [82, 25],
  AMCR: [63, 28],
  AMC: [50, 30],
  AMCL: [37, 28],
  AML: [18, 25],
  STR: [62, 14],
  STCR: [60, 14],
  STC: [50, 14],
  ST: [50, 14],
  STCL: [40, 14],
  STL: [38, 14],
};

function normalizedPosition(value: string) {
  const upper = value.toUpperCase().replace(/[^A-Z0-9]/g, "");
  if (upper === "CB") return "DC";
  if (upper === "CM") return "MC";
  if (upper === "LB") return "DL";
  if (upper === "RB") return "DR";
  if (upper === "LW") return "AML";
  if (upper === "RW") return "AMR";
  if (upper === "CF") return "ST";
  return upper;
}

function positionFamily(value: string) {
  const position = normalizedPosition(value);
  if (position.includes("GK")) return "GK";
  if (position.startsWith("D") && !position.startsWith("DM")) return "DEF";
  if (position.startsWith("WB")) return "WB";
  if (position.startsWith("DM")) return "DM";
  if (position.startsWith("M") && !position.startsWith("MR") && !position.startsWith("ML")) return "CM";
  if (position === "MR" || position === "ML" || position.startsWith("AMR") || position.startsWith("AML")) return "WIDE";
  if (position.startsWith("AM")) return "AM";
  if (position.startsWith("ST")) return "ST";
  return position;
}

function coordinatesForSlot(slot: LiveTacticSlot, index: number, phase: TacticalPhase) {
  const normalized = normalizedPosition(slot.position);
  const base = positionCoordinates[normalized] ?? fallbackCoordinates[index % fallbackCoordinates.length];
  const [left, baseTop] = base;
  let top = baseTop;
  if (phase === "In Possession") {
    if (["DR", "DL", "WBR", "WBL", "MR", "ML", "AMR", "AML"].some((code) => normalized.startsWith(code))) top = Math.max(12, top - 5);
    if (normalized.startsWith("DM") || normalized.startsWith("MC") || normalized.startsWith("MCR") || normalized.startsWith("MCL")) top = Math.max(18, top - 3);
  } else if (phase === "Out of Possession") {
    if (!normalized.includes("GK")) top = Math.min(78, top + 5);
    if (normalized.startsWith("ST")) top = Math.min(45, top + 9);
  }
  return [left, top] as const;
}

function playerById(players: LivePlayer[], playerId: string | null) {
  return playerId ? players.find((player) => player.id === playerId) ?? null : null;
}

function playerCanCoverSlot(player: LivePlayer, slot: LiveTacticSlot) {
  const slotPosition = normalizedPosition(slot.position);
  const slotFamily = positionFamily(slotPosition);
  return player.positions.some((position) => {
    const normalized = normalizedPosition(position);
    return normalized === slotPosition || positionFamily(normalized) === slotFamily;
  }) || player.playableRoles?.some((role) => role.positions.some((position) => normalizedPosition(position) === slotPosition || positionFamily(position) === slotFamily)) === true;
}

function phaseFit(player: LivePlayer, phase: TacticalPhase) {
  if (phase === "In Possession") return player.inPossessionFit ?? player.projectedInPossessionFit ?? null;
  if (phase === "Out of Possession") return player.outOfPossessionFit ?? player.projectedOutOfPossessionFit ?? null;
  const values = [player.inPossessionFit, player.outOfPossessionFit, player.projectedInPossessionFit, player.projectedOutOfPossessionFit]
    .filter((value): value is number => typeof value === "number");
  if (!values.length) return null;
  return Math.round(values.reduce((sum, value) => sum + value, 0) / values.length);
}

function slotFit(player: LivePlayer | null, slot: LiveTacticSlot, phase: TacticalPhase) {
  if (!player) return null;
  const targetRole = (slot.roleShort ?? slot.role ?? "").toLowerCase();
  const roleMatch = player.playableRoles?.find((role) => {
    const roleName = `${role.shortRole} ${role.role}`.toLowerCase();
    return targetRole && roleName.includes(targetRole);
  });
  const cover = playerCanCoverSlot(player, slot);
  const base = roleMatch?.score ?? phaseFit(player, phase) ?? player.roleFit ?? player.tacticalFit ?? player.efficiencyScore ?? null;
  if (base == null) return cover ? 50 : null;
  return Math.round(Math.max(0, Math.min(100, cover ? base : base - 18)));
}

function bestAlternatives(players: LivePlayer[], slot: LiveTacticSlot, phase: TacticalPhase, selectedPlayerId: string | null) {
  return players
    .filter((player) => player.id !== selectedPlayerId && playerCanCoverSlot(player, slot))
    .map((player) => ({ player, fit: slotFit(player, slot, phase) }))
    .filter((item): item is { player: LivePlayer; fit: number } => item.fit != null)
    .toSorted((left, right) => right.fit - left.fit || left.player.name.localeCompare(right.player.name))
    .slice(0, 5);
}

export function TacticalBoard({
  snapshot,
  compact = false,
  onOpenPlayer,
}: {
  snapshot: LiveFootballSnapshot;
  compact?: boolean;
  onOpenPlayer?: (playerId: string) => void;
}) {
  const [phase, setPhase] = useState<TacticalPhase>("In Possession");
  const [selectedSlotIndex, setSelectedSlotIndex] = useState(0);
  const slots = snapshot.tactic?.slots ?? [];
  const selectedSlot = slots[selectedSlotIndex] ?? slots[0] ?? null;
  const selectedPlayer = selectedSlot ? playerById(snapshot.players, selectedSlot.playerId) : null;
  const alternatives = useMemo(
    () => selectedSlot ? bestAlternatives(snapshot.players, selectedSlot, phase, selectedSlot.playerId) : [],
    [phase, selectedSlot, snapshot.players],
  );

  return (
    <section className={compact ? "tactical-board tactical-board-compact" : "tactical-board"}>
      <header className="tactical-board-header">
        <div>
          <h2>{compact ? "System overview" : "Tactical board"}</h2>
          {!compact ? <p>Selected XI slots come from live FM26 memory. Slot score is recalculated from the player’s mapped position, role and visible attributes.</p> : null}
        </div>
        <div className="phase-toggle" role="group" aria-label="Tactical phase">
          {(["In Possession", "Out of Possession", "Combined"] as const).map((item) => (
            <button key={item} className={phase === item ? "active" : ""} onClick={() => setPhase(item)}>{item}</button>
          ))}
        </div>
      </header>

      <div className="pitch-canvas">
        <span className="pitch-halfway" /><span className="pitch-centre" />
        <span className="pitch-box pitch-box-top" /><span className="pitch-box pitch-box-bottom" />
        <span className="pitch-goal pitch-goal-top" /><span className="pitch-goal pitch-goal-bottom" />
        <div className="formation-card">
          <strong>{snapshot.tactic?.formation ?? "—"}</strong>
          <span>{snapshot.tactic ? phase : "Live tactic pending"}</span>
        </div>

        {slots.map((slot, index) => {
          const player = playerById(snapshot.players, slot.playerId);
          const [left, top] = coordinatesForSlot(slot, index, phase);
          const fit = slotFit(player, slot, phase);
          const roleLabel = slot.roleShort ?? slot.role ?? "Role pending";
          const selected = index === selectedSlotIndex;
          return (
            <button key={`${slot.position}-${index}`} className={selected ? "pitch-player selected" : "pitch-player"} style={{ left: `${left}%`, top: `${top}%` }} onClick={() => setSelectedSlotIndex(index)}>
              <span className="fit-ring" style={{ "--fit": `${fit ?? 0}%` } as CSSProperties} data-known={fit != null}>{fit ?? "—"}</span>
              <strong>{player?.name ?? "Unassigned"}</strong>
              <small>{slot.position} · {roleLabel}{slot.dutyShort || slot.duty ? ` · ${slot.dutyShort ?? slot.duty}` : ""}</small>
            </button>
          );
        })}

        {!slots.length ? (
          <div className="pitch-empty">
            <ShieldAlert />
            <strong>{snapshot.status.liveMemoryTacticRead === "object_detected_unmapped" ? "Live tactic detected — slot decoder pending" : "Waiting for active FM26 tactic"}</strong>
            <span>GlassScout will not invent a formation, role, duty, instruction or first XI.</span>
          </div>
        ) : null}
      </div>

      {selectedSlot ? (
        <section className="tactic-slot-panel">
          <header><Sparkles /><strong>{selectedSlot.position} slot</strong><span>{selectedSlot.roleShort ?? selectedSlot.role ?? "Role pending"}{selectedSlot.dutyShort || selectedSlot.duty ? ` · ${selectedSlot.dutyShort ?? selectedSlot.duty}` : ""}</span></header>
          <div className="slot-current-player">
            <span><small>Picked in FM26</small><strong>{selectedPlayer?.name ?? "Unassigned"}</strong></span>
            <span><small>Dynamic slot fit</small><strong>{slotFit(selectedPlayer, selectedSlot, phase) ?? "Unknown"}</strong></span>
            {selectedPlayer ? <button type="button" onClick={() => onOpenPlayer?.(selectedPlayer.id)}>Open profile</button> : null}
          </div>
          <div className="slot-alternatives">
            <small>Squad alternatives for this role/position</small>
            {alternatives.length ? alternatives.map((item) => (
              <button key={item.player.id} type="button" onClick={() => onOpenPlayer?.(item.player.id)}>
                <strong>{item.player.name}</strong><span>{item.player.positions.join(" / ")}</span><b>{item.fit}</b>
              </button>
            )) : <p>No mapped squad alternative has enough position/role evidence yet.</p>}
          </div>
        </section>
      ) : null}
    </section>
  );
}
