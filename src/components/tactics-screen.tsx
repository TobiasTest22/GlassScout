"use client";

import { AlertTriangle, CheckCircle2, Cpu, ShieldCheck } from "lucide-react";
import type { LiveFootballSnapshot } from "@/domain/adapters";
import { TacticalBoard } from "@/components/tactical-board";

export function TacticsScreen({ snapshot }: { snapshot: LiveFootballSnapshot }) {
  const ready = snapshot.tacticSource === "live-memory" && snapshot.tactic != null;
  const objectDetected = snapshot.status.liveMemoryTacticRead === "object_detected_unmapped";

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
        <TacticalBoard snapshot={snapshot} />
        <aside className="tactical-inspector">
          <section>
            <header><Cpu /><h2>Live tactic source</h2></header>
            <dl>
              <div><dt>Source</dt><dd>{snapshot.tacticSource === "live-memory" ? "Active FM26 tactic" : "None"}</dd></div>
              <div><dt>Tactic object</dt><dd>{objectDetected || ready ? "Detected" : "Not detected"}</dd></div>
              <div><dt>Formation</dt><dd>{snapshot.tactic?.formation ?? "Not validated"}</dd></div>
              <div><dt>Roles and duties</dt><dd>{ready ? "Available" : "Not validated"}</dd></div>
              <div><dt>Instructions</dt><dd>{snapshot.tactic?.teamInstructions.length ? "Available" : "Not validated"}</dd></div>
            </dl>
          </section>
          <section>
            <header>{ready ? <CheckCircle2 /> : <AlertTriangle />}<h2>Analysis readiness</h2></header>
            <p>
              {ready
                ? "The active tactic has passed build-specific validation and can be used for tactical fit."
                : objectDetected
                  ? "GlassScout found FM26’s live tactic manager, but the packed formation, in-possession and out-of-possession slot layout is not fully decoded for this build. No tactic or fit score is invented."
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
