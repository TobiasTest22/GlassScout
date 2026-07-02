"use client";

import { AlertTriangle, CheckCircle2, FileArchive, Upload } from "lucide-react";
import type { LiveFootballSnapshot } from "@/domain/adapters";
import { TacticalBoard } from "@/components/tactical-board";
import { Button } from "@/components/ui/button";

function formatFileSize(value: number | null | undefined) {
  if (value == null) return "Unknown size";
  if (value < 1024) return `${value} bytes`;
  return `${(value / 1024).toFixed(1)} KB`;
}

function formatImportDate(value: string | null | undefined) {
  if (!value) return "Unknown date";
  const date = new Date(Number(value));
  return Number.isNaN(date.valueOf()) ? "Unknown date" : date.toLocaleString();
}

export function TacticsScreen({
  snapshot,
  importing,
  onImportTactic,
}: {
  snapshot: LiveFootballSnapshot;
  importing: boolean;
  onImportTactic: () => Promise<void>;
}) {
  const parsed = snapshot.tactic != null && (
    snapshot.tacticFileStatus === "parsed" || snapshot.tacticFileStatus === "partially_parsed"
  );

  return (
    <main className="screen tactical-workspace">
      <div className="planner-heading">
        <div>
          <h1>Tactical Board</h1>
          <p>Live squad context with a user-selected FMF tactic. No role or duty is inferred from memory.</p>
        </div>
        <Button onClick={onImportTactic} disabled={importing}>
          <Upload data-icon="inline-start" />
          {importing ? "Importing…" : snapshot.tacticFileName ? "Replace FMF tactic" : "Import FMF tactic"}
        </Button>
      </div>

      <div className="tactical-workspace-grid">
        <TacticalBoard
          snapshot={snapshot}
          onImport={onImportTactic}
          importing={importing}
        />
        <aside className="tactical-inspector">
          <section>
            <header><FileArchive /><h2>Tactic source</h2></header>
            <dl>
              <div><dt>File</dt><dd>{snapshot.tacticFileName ?? "Not imported"}</dd></div>
              <div><dt>Parser</dt><dd>{snapshot.tacticFileStatus.replaceAll("_", " ")}</dd></div>
              <div><dt>Formation</dt><dd>{snapshot.tactic?.formation ?? "Unknown"}</dd></div>
              <div><dt>Imported</dt><dd>{formatImportDate(snapshot.tacticImportedAt)}</dd></div>
              <div><dt>Size</dt><dd>{formatFileSize(snapshot.tacticFileSize)}</dd></div>
              <div><dt>Archive entries</dt><dd>{snapshot.tacticArchiveEntries?.join(", ") || "Unavailable"}</dd></div>
            </dl>
          </section>
          <section>
            <header>{parsed ? <CheckCircle2 /> : <AlertTriangle />}<h2>Analysis readiness</h2></header>
            <p>
              {parsed
                ? "Decoded formation data is available. Fit scores still require visible player attributes."
                : snapshot.tacticFileName
                  ? "The FMF container is stored safely, but roles, duties and team instructions are not decoded. The board remains deliberately empty."
                  : "Import the FMF tactic selected in FM26 to begin tactical analysis."}
            </p>
          </section>
          <section>
            <header><AlertTriangle /><h2>Instruction evidence</h2></header>
            {[
              "Mentality",
              "Pressing intensity",
              "Defensive line",
              "Tempo and passing",
              "Width and build-up",
              "Rotations and player instructions",
            ].map((label) => (
              <span className="instruction-row" key={label}><b>{label}</b><small>Unknown</small></span>
            ))}
          </section>
        </aside>
      </div>

      {snapshot.tacticFileWarnings.length ? (
        <section className="tactic-file-warnings">
          {snapshot.tacticFileWarnings.map((warning) => <p key={warning}>{warning}</p>)}
        </section>
      ) : null}
      {snapshot.tacticFileErrors?.length ? (
        <section className="tactic-import-errors">
          {snapshot.tacticFileErrors.map((error) => <span key={error}><AlertTriangle />{error}</span>)}
        </section>
      ) : null}
    </main>
  );
}
