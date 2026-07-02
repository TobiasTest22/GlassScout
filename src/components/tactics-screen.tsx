"use client";

import { AlertTriangle, CheckCircle2, Cpu, ShieldCheck } from "lucide-react";
import type { LiveFootballSnapshot } from "@/domain/adapters";
import { TacticalBoard } from "@/components/tactical-board";

export function TacticsScreen({ snapshot, onOpenPlayer }: { snapshot: LiveFootballSnapshot; onOpenPlayer?: (playerId: string) => void }) {
  const ready = snapshot.tacticSource === "live-memory" && snapshot.tactic != null;
  const objectDetected = snapshot.status.liveMemoryTacticRead === "object_detected_unmapped";
  const tacticObjectDetected = ready || objectDetected;
  const rolesResolved = snapshot.tactic?.rolesResolved ?? 0;
  const dutiesResolved = snapshot.tactic?.dutiesResolved ?? 0;
  const roleDutyStatus = snapshot.tactic?.roleDutyDecoderStatus ?? "packet-not-validated";

  return (
    <main className="screen tactical-workspace">
      <div className="planner-heading">
        <div>
          <h1>Tactical Board</h1>
          <p>The board follows the active tactic in the connected FM26 save. There is no file import or fallback tactic.</p>
        </div>
        <div className="live-source-label"><span className="live-dot" />Live FM26</div>
      </div>

      <div className="tactical-workspace-grid">
        <TacticalBoard snapshot={snapshot} onOpenPlayer={onOpenPlayer} />
        <aside className="tactical-inspector">
          <section>
            <header><Cpu /><h2>Live tactic source</h2></header>
            <dl>
              <div><dt>Source</dt><dd>{snapshot.tacticSource === "live-memory" ? "Active FM26 tactic" : "None"}</dd></div>
              <div><dt>Tactic object</dt><dd>{tacticObjectDetected ? "Detected" : "Not detected"}</dd></div>
              <div><dt>Formation</dt><dd>{snapshot.tactic?.formation ?? "Not validated"}</dd></div>
              <div><dt>FM26 enum</dt><dd>{snapshot.tactic?.formationEnum ?? "Not available"}</dd></div>
              <div><dt>Pitch layout</dt><dd>{snapshot.tactic?.layoutStatus === "exact-template" ? "Template mapped" : ready ? "Selected XI only" : "Not validated"}</dd></div>
              <div><dt>Selected XI</dt><dd>{snapshot.tactic?.slots.length ? `${snapshot.tactic.slots.length} live slots` : "Not validated"}</dd></div>
              <div><dt>Roles and duties</dt><dd>{ready ? `${rolesResolved}/11 roles · ${dutiesResolved}/11 duties` : "Not validated"}</dd></div>
              <div><dt>Role packet</dt><dd>{ready ? roleDutyStatus.replaceAll("-", " ") : "Not validated"}</dd></div>
              <div><dt>Instructions</dt><dd>{snapshot.tactic?.teamInstructions.length ? "Available" : ready ? "Decoder pending" : "Not validated"}</dd></div>
            </dl>
          </section>
          <section>
            <header>{ready ? <CheckCircle2 /> : <AlertTriangle />}<h2>Analysis readiness</h2></header>
            <p>
              {ready
                ? snapshot.tactic?.layoutStatus === "exact-template"
                  ? rolesResolved === 11
                    ? dutiesResolved === 11
                      ? "The active FM26 formation, selected XI, roles and duties have passed live memory validation."
                      : "The active FM26 formation, selected XI and role masks have passed live memory validation. Duty masks are still pending for this read."
                    : "The active FM26 formation template and selected XI have passed memory validation. Role/duty fit remains disabled until the packed role codes validate."
                  : "The active FM26 formation code and selected XI are live. Exact pitch placement for this formation still needs template validation, so no role/duty fit is invented."
                : objectDetected
                  ? "GlassScout found FM26’s live tactic manager, but the selected-slot block did not validate for this read. No tactic or fit score is invented."
                  : "The live tactic object was not available. Open FM26, load the save and select the active tactic."}
            </p>
          </section>
          <section>
            <header><ShieldCheck /><h2>Safe behavior</h2></header>
            <p>Squad and player data remain available when tactic decoding is unavailable. Tactic failure never blocks the workspace.</p>
          </section>
        </aside>
      </div>
    </main>
  );
}
