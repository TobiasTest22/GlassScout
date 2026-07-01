"use client";

import { AlertTriangle, Check, Cpu, Database, LockKeyhole, RefreshCw, ShieldCheck } from "lucide-react";
import type { LiveConnectorStatus } from "@/domain/adapters";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card, CardAction, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";

export function MemoryScreen({
  status,
  checking,
  onCheck,
}: {
  status: LiveConnectorStatus;
  checking: boolean;
  onCheck: () => void;
}) {
  return (
    <main className="screen center-screen">
      <div className="planner-heading">
        <div><h1>FM26 Connection</h1><p>Read-only process access, active-save readiness and live entity diagnostics.</p></div>
        <Button onClick={onCheck} disabled={checking}><RefreshCw data-icon="inline-start" />{checking ? "Checking…" : "Run diagnostics"}</Button>
      </div>
      <div className="connection-summary">
        <span className={status.processDetected ? "status-good" : "status-neutral"}><Cpu /><small>Process</small><strong>{status.processDetected ? `Detected · PID ${status.processId}` : "Not detected"}</strong></span>
        <span className={status.memoryAccess === "read_only_handle_open" ? "status-good" : "status-neutral"}><LockKeyhole /><small>Memory access</small><strong>{status.memoryAccess.replaceAll("_", " ")}</strong></span>
        <span className={status.saveDetected ? "status-good" : "status-watch"}><Database /><small>Active save</small><strong>{status.saveDetected === null ? "Not readable" : status.saveDetected ? "Detected" : "Not detected"}</strong></span>
        <span className={status.parserStatus === "ready" ? "status-good" : "status-watch"}><ShieldCheck /><small>Entity parser</small><strong>{status.parserStatus}</strong></span>
      </div>
      <div className="memory-grid">
        <Card className="glass-card memory-primary">
          <CardHeader><CardTitle>Live memory adapter</CardTitle><CardAction><Badge variant={status.state === "connected" ? "secondary" : "outline"}>{status.state.replaceAll("_", " ")}</Badge></CardAction></CardHeader>
          <CardContent>
            <div className="connector-state"><span className="connector-icon"><Cpu /></span><div><strong>{status.message}</strong><p>{status.processPath ?? "FM26 executable path unavailable."}</p></div></div>
            <div className="diagnostic-table">
              <span><small>Memory bytes read</small><strong>{status.bytesRead}</strong></span>
              <span><small>Executable header</small><strong>{status.executableHeaderValid ? "Verified" : "Not verified"}</strong></span>
              <span><small>Entities loaded</small><strong>{status.playersLoaded} / {status.clubsLoaded}</strong></span>
              <span><small>Write capability</small><strong>Disabled</strong></span>
            </div>
            {status.warnings.map((warning) => <div className="diagnostic-warning" key={warning}><AlertTriangle />{warning}</div>)}
          </CardContent>
        </Card>
        <Card className="glass-card">
          <CardHeader><CardTitle>Data integrity</CardTitle><CardDescription>Connector contract</CardDescription></CardHeader>
          <CardContent className="firewall-stats">
            <div><strong>0</strong><span>Fallback entities</span></div>
            <div><strong>100%</strong><span>Write paths disabled</span></div>
            <Progress value={100} />
            <small>When live entity extraction is unavailable, GlassScout returns empty collections and a diagnostic error.</small>
          </CardContent>
        </Card>
        <Card className="glass-card">
          <CardHeader><CardTitle>Launch readiness</CardTitle></CardHeader>
          <CardContent className="safeguard-list">
            {[
              ["FM26 process probe", status.processDetected],
              ["Executable memory probe", status.executableHeaderValid],
              ["Installed-build entity parser", status.parserStatus === "ready"],
              ["Active save discovery", status.saveDetected === true],
            ].map(([label, ready]) => <div key={String(label)}>{ready ? <Check /> : <AlertTriangle />}<span>{label}</span><b>{ready ? "Ready" : "Not ready"}</b></div>)}
          </CardContent>
        </Card>
      </div>
    </main>
  );
}
