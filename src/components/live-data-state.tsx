"use client";

import { AlertTriangle, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { LiveFootballSnapshot } from "@/domain/adapters";

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
  return (
    <section className="live-data-state" role="status" aria-live="polite">
      <div className="live-data-state-icon"><AlertTriangle /></div>
      <div>
        <span className="section-kicker">Waiting for FM26 live connection</span>
        <h1>{snapshot.status.processDetected ? `${title} is not ready` : "Open FM26 and load your save to begin"}</h1>
        <p>{snapshot.status.processDetected ? snapshot.status.message : "GlassScout will connect automatically when the active game becomes available."}</p>
      </div>
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
