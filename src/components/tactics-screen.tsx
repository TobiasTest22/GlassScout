"use client";

import { AlertTriangle, CheckCircle2, FileArchive, Upload } from "lucide-react";
import type { LiveFootballSnapshot } from "@/domain/adapters";
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
  const tactic = snapshot.tactic;
  const imported = snapshot.tacticSource === "fmf-file";
  const parsed = tactic != null && (
    snapshot.tacticFileStatus === "parsed" || snapshot.tacticFileStatus === "partially_parsed"
  );

  return (
    <main className="screen tactics-live-screen">
      <div className="planner-heading">
        <div>
          <h1>Tactic Evaluation</h1>
          <p>Player and squad data remain live from FM26. Tactics come only from the FMF file you choose.</p>
        </div>
        <Button onClick={onImportTactic} disabled={importing}>
          <Upload data-icon="inline-start" />
          {importing ? "Importing…" : imported ? "Replace FMF tactic" : "Import FMF tactic"}
        </Button>
      </div>

      {!imported ? (
        <section className="tactic-import-empty">
          <span><FileArchive /></span>
          <p className="section-kicker">No tactic file imported</p>
          <h2>Import your FM tactic file to evaluate tactic fit.</h2>
          <p>Save your tactic from FM26 as an <strong>.fmf</strong> file, then choose that file here. GlassScout will copy it into local app storage and inspect it safely.</p>
          <Button onClick={onImportTactic} disabled={importing}><Upload data-icon="inline-start" />Import FMF tactic</Button>
          {snapshot.tacticFileErrors?.length ? <div className="tactic-import-errors">{snapshot.tacticFileErrors.map((error) => <span key={error}><AlertTriangle />{error}</span>)}</div> : null}
          {snapshot.status.state !== "connected" ? <small>Live squad connection is also required before tactic-fit calculations can run.</small> : null}
        </section>
      ) : (
        <>
          <section className="tactic-file-status">
            <div className="tactic-file-icon"><FileArchive /></div>
            <div>
              <span className="section-kicker">Imported FMF tactic</span>
              <h2>{snapshot.tacticFileName}</h2>
              <p>{formatFileSize(snapshot.tacticFileSize)} · Imported {formatImportDate(snapshot.tacticImportedAt)}</p>
            </div>
            <div className={`tactic-parser-badge tactic-parser-${snapshot.tacticFileStatus}`}>
              {parsed ? <CheckCircle2 /> : <AlertTriangle />}
              {snapshot.tacticFileStatus.replaceAll("_", " ")}
            </div>
          </section>

          {!parsed ? (
            <section className="tactic-parser-limited">
              <AlertTriangle />
              <div>
                <h2>Tactic file imported. Parser support needs to be completed for this FMF format.</h2>
                <p>GlassScout detected and stored the file, but it did not decode a trustworthy formation, role or duty record. No tactic or fit score is being shown.</p>
                {snapshot.tacticDetectedFormat ? <span>Detected format: {snapshot.tacticDetectedFormat}</span> : null}
              </div>
            </section>
          ) : (
            <section className="tactic-file-parsed">
              <div><span>Formation</span><strong>{tactic.formation}</strong></div>
              <div><span>Roles decoded</span><strong>{tactic.slots.filter((slot) => slot.role).length}</strong></div>
              <div><span>Duties decoded</span><strong>{tactic.slots.filter((slot) => slot.duty).length}</strong></div>
              <p>Tactic-fit scores run only from the decoded FMF tactic and readable live player data.</p>
            </section>
          )}

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
        </>
      )}
    </main>
  );
}
