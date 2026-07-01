"use client";

import { AlertTriangle, Database, RefreshCw } from "lucide-react";
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
        <span className="section-kicker">Live FM26 data required</span>
        <h1>{title} is unavailable</h1>
        <p>{snapshot.dataError ?? snapshot.status.message}</p>
      </div>
      <dl>
        <div><dt>FM26 process</dt><dd>{snapshot.status.processDetected ? `Detected · PID ${snapshot.status.processId}` : "Not detected"}</dd></div>
        <div><dt>Memory probe</dt><dd>{snapshot.status.executableHeaderValid ? `Verified · ${snapshot.status.bytesRead} bytes read` : "Not verified"}</dd></div>
        <div><dt>Active save</dt><dd>{snapshot.status.saveDetected === true ? "Detected" : snapshot.status.saveDetected === false ? "Not detected" : "Not readable"}</dd></div>
        <div><dt>Entities loaded</dt><dd>{snapshot.players.length} players · {snapshot.clubs.length} clubs</dd></div>
      </dl>
      <div className="live-data-state-actions">
        <Button onClick={onRefresh} disabled={checking}>
          <RefreshCw data-icon="inline-start" className={checking ? "spin" : undefined} />
          {checking ? "Checking…" : "Run live check"}
        </Button>
        <span><Database />No fallback, seeded or sample data is shown.</span>
      </div>
    </section>
  );
}
