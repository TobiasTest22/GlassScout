"use client";

import { useState, type CSSProperties } from "react";
import { FileUp, ShieldAlert } from "lucide-react";
import type { LiveFootballSnapshot, LivePlayer, LiveTacticSlot } from "@/domain/adapters";
import { Button } from "@/components/ui/button";

type TacticalPhase = "In Possession" | "Out of Possession" | "Combined";

const fallbackCoordinates = [
  [50, 86],
  [13, 69],
  [37, 70],
  [63, 70],
  [87, 69],
  [50, 53],
  [28, 45],
  [72, 45],
  [18, 22],
  [50, 16],
  [82, 22],
];

function coordinatesForSlot(slot: LiveTacticSlot, index: number) {
  const value = slot.position.toUpperCase();
  if (value.includes("GK")) return [50, 86];
  if (value.includes("DL") || value.includes("LB")) return [13, 69];
  if (value.includes("DR") || value.includes("RB")) return [87, 69];
  if (value.includes("DC") || value.includes("CB")) return index % 2 ? [38, 70] : [62, 70];
  if (value.includes("DM")) return [50, 55];
  if (value.includes("AML") || value.includes("LW")) return [18, 24];
  if (value.includes("AMR") || value.includes("RW")) return [82, 24];
  if (value.includes("AMC")) return [50, 32];
  if (value.includes("MC") || value.includes("CM")) return index % 2 ? [34, 45] : [66, 45];
  if (value.includes("ST") || value.includes("CF")) return [50, 15];
  return fallbackCoordinates[index % fallbackCoordinates.length];
}

function playerById(players: LivePlayer[], playerId: string | null) {
  if (!playerId) return null;
  return players.find((player) => player.id === playerId) ?? null;
}

export function TacticalBoard({
  snapshot,
  compact = false,
  onImport,
  importing = false,
  onOpenPlayer,
}: {
  snapshot: LiveFootballSnapshot;
  compact?: boolean;
  onImport?: () => void;
  importing?: boolean;
  onOpenPlayer?: (playerId: string) => void;
}) {
  const [phase, setPhase] = useState<TacticalPhase>("In Possession");
  const slots = snapshot.tactic?.slots ?? [];

  return (
    <section className={compact ? "tactical-board tactical-board-compact" : "tactical-board"}>
      <header className="tactical-board-header">
        <div>
          <h2>{compact ? "System overview" : "Tactical board"}</h2>
          {!compact ? <p>Roles and fit are shown only when the imported FMF has been decoded.</p> : null}
        </div>
        <div className="phase-toggle" role="group" aria-label="Tactical phase">
          {(["In Possession", "Out of Possession", "Combined"] as const).map((item) => (
            <button key={item} className={phase === item ? "active" : ""} onClick={() => setPhase(item)}>
              {item}
            </button>
          ))}
        </div>
      </header>

      <div className="pitch-canvas">
        <span className="pitch-halfway" />
        <span className="pitch-centre" />
        <span className="pitch-box pitch-box-top" />
        <span className="pitch-box pitch-box-bottom" />
        <span className="pitch-goal pitch-goal-top" />
        <span className="pitch-goal pitch-goal-bottom" />
        <div className="formation-card">
          <strong>{snapshot.tactic?.formation ?? "—"}</strong>
          <span>{snapshot.tactic ? phase : "No decoded tactic"}</span>
        </div>

        {slots.map((slot, index) => {
          const player = playerById(snapshot.players, slot.playerId);
          const [left, top] = coordinatesForSlot(slot, index);
          const fit = player?.roleFit;
          return (
            <button
              key={`${slot.position}-${index}`}
              className="pitch-player"
              style={{ left: `${left}%`, top: `${top}%` }}
              onClick={() => player && onOpenPlayer?.(player.id)}
              disabled={!player}
            >
              <span
                className="fit-ring"
                style={{ "--fit": `${fit ?? 0}%` } as CSSProperties}
                data-known={fit != null}
              >
                {fit ?? "—"}
              </span>
              <strong>{player?.name ?? "Unassigned"}</strong>
              <small>{slot.position} · {slot.role ?? "Role unknown"}{slot.duty ? ` · ${slot.duty}` : ""}</small>
            </button>
          );
        })}

        {!slots.length ? (
          <div className="pitch-empty">
            <ShieldAlert />
            <strong>{snapshot.tacticFileName ? "FMF imported, formation not decoded" : "No tactic loaded"}</strong>
            <span>
              {snapshot.tacticFileName
                ? "GlassScout will not invent a formation, role, duty or first XI."
                : "Import the tactic selected in FM26 to activate this board."}
            </span>
            {onImport ? (
              <Button variant="outline" onClick={onImport} disabled={importing}>
                <FileUp data-icon="inline-start" />
                {importing ? "Importing…" : snapshot.tacticFileName ? "Replace FMF tactic" : "Import FMF tactic"}
              </Button>
            ) : null}
          </div>
        ) : null}
      </div>
    </section>
  );
}
