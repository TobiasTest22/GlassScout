"use client";

import { AlertTriangle, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { LiveFootballSnapshot } from "@/domain/adapters";

function readable(value: string | null | undefined) {
  if (!value) return "None";
  return value.replaceAll("_", " ").replaceAll("-", " ");
}

export function LiveDataState({
  snapshot,
  title,
  checking,
  onRefresh,
}: {
  snapshot: LiveFootballSnapshot;
  title: string;
  checking: boolean;
  onRefresh: () => Promise<unknown>;
}) {
  const pipeline = snapshot.status.readPipeline ?? [];
  const visiblePipeline = pipeline.filter((stage) => stage.state !== "pending").slice(0, 4);
  return (
    <section className="live-data-state" role="status" aria-live="polite">
      <div className="live-data-state-icon"><AlertTriangle /></div>
      <div>
        <span className="section-kicker">Waiting for FM26 live connection</span>
        <h1>{snapshot.status.processDetected ? `${title} is not ready` : "Open FM26 and load your save to begin"}</h1>
        <p>{snapshot.status.processDetected ? snapshot.status.message : "GlassScout will connect automatically when the active game becomes available."}</p>
      </div>
      <dl>
        <div><dt>Failed stage</dt><dd>{readable(snapshot.status.failureStage)}</dd></div>
        <div><dt>Last good read</dt><dd>{readable(snapshot.status.lastSuccessfulRead)}</dd></div>
        <div><dt>Memory access</dt><dd>{readable(snapshot.status.memoryAccess)}</dd></div>
        <div><dt>Build map</dt><dd>{snapshot.status.entityMapProfileId ?? readable(snapshot.status.entityMapStatus)}</dd></div>
      </dl>
      {visiblePipeline.length ? (
        <div className="pipeline-mini-list">
          {visiblePipeline.map((stage) => (
            <span key={stage.key} data-state={stage.state}>
              <b>{stage.label}</b>
              <small>{stage.detail}</small>
            </span>
          ))}
        </div>
      ) : null}
      <div className="live-data-state-actions">
        <Button onClick={onRefresh} disabled={checking}>
          <RefreshCw data-icon="inline-start" className={checking ? "spin" : undefined} />
          {checking ? "Checking…" : "Run live check"}
        </Button>
        <span>Technical details are available under Settings → Advanced diagnostics.</span>
      </div>
    </section>
  );
}
