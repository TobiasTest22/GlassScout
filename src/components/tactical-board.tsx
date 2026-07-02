"use client";

import { useState, type CSSProperties } from "react";
import { ShieldAlert } from "lucide-react";
import type { LiveFootballSnapshot, LivePlayer, LiveTacticSlot } from "@/domain/adapters";

type TacticalPhase = "In Possession" | "Out of Possession" | "Combined";

const fallbackCoordinates = [[50, 83], [20, 68], [40, 68], [60, 68], [80, 68], [50, 54], [30, 42], [70, 42], [18, 25], [82, 25], [50, 14]];

function coordinatesForSlot(slot: LiveTacticSlot, index: number) {
  const value = slot.position.toUpperCase();
  if (value.includes("GK")) return [50, 84];
  if (value.includes("DL") || value.includes("LB")) return [18, 68];
  if (value.includes("DR") || value.includes("RB")) return [82, 68];
  if (value.includes("DC") || value.includes("CB")) return index % 2 ? [39, 68] : [61, 68];
  if (value.includes("DM")) return [50, 56];
  if (value.includes("AML") || value.includes("LW")) return [18, 25];
  if (value.includes("AMR") || value.includes("RW")) return [82, 25];
  if (value.includes("AMC")) return [50, 34];
  if (value.includes("MC") || value.includes("CM")) return index % 2 ? [34, 45] : [66, 45];
  if (value.includes("ST") || value.includes("CF")) return [50, 15];
  return fallbackCoordinates[index % fallbackCoordinates.length];
}

function playerById(players: LivePlayer[], playerId: string | null) {
  return playerId ? players.find((player) => player.id === playerId) ?? null : null;
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
  const slots = snapshot.tactic?.slots ?? [];

  return (
    <section className={compact ? "tactical-board tactical-board-compact" : "tactical-board"}>
      <header className="tactical-board-header">
        <div>
          <h2>{compact ? "System overview" : "Tactical board"}</h2>
          {!compact ? <p>Formation, roles and duties are shown only after live-memory validation.</p> : null}
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
          const [left, top] = coordinatesForSlot(slot, index);
          const fit = player?.roleFit;
          return (
            <button key={`${slot.position}-${index}`} className="pitch-player" style={{ left: `${left}%`, top: `${top}%` }} onClick={() => player && onOpenPlayer?.(player.id)} disabled={!player}>
              <span className="fit-ring" style={{ "--fit": `${fit ?? 0}%` } as CSSProperties} data-known={fit != null}>{fit ?? "—"}</span>
              <strong>{player?.name ?? "Unassigned"}</strong>
              <small>{slot.position} · {slot.role ?? "Role unknown"}{slot.duty ? ` · ${slot.duty}` : ""}</small>
            </button>
          );
        })}

        {!slots.length ? (
          <div className="pitch-empty">
            <ShieldAlert />
            <strong>{snapshot.status.liveMemoryTacticRead === "object_detected_unmapped" ? "Live tactic detected — layout not validated" : "Waiting for active FM26 tactic"}</strong>
            <span>GlassScout will not invent a formation, role, duty, instruction or first XI.</span>
          </div>
        ) : null}
      </div>
    </section>
  );
}
