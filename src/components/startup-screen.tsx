"use client";

import { useState } from "react";
import { Check, CircleDot, Database, Globe2, LockKeyhole, ShieldCheck, UsersRound } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { LiveConnectorStatus } from "@/domain/adapters";

const waitingStatus: LiveConnectorStatus = {
  processDetected: false,
  processId: null,
  processPath: null,
  saveDetected: null,
  memoryAccess: "not_checked",
  parserStatus: "unverified",
  state: "not_checked",
  playersLoaded: 0,
  clubsLoaded: 0,
  lastSync: null,
  bytesRead: 0,
  executableHeaderValid: false,
  canWriteMemory: false,
  message: "Waiting for a read-only desktop diagnostic.",
  warnings: [],
};

export function StartupScreen({
  onConnect,
  onEnter,
}: {
  onConnect: () => Promise<LiveConnectorStatus>;
  onEnter: (status: LiveConnectorStatus) => void;
}) {
  const [checking, setChecking] = useState(false);
  const [status, setStatus] = useState(waitingStatus);

  const connect = async () => {
    setChecking(true);
    const next = await onConnect();
    setStatus(next);
    setChecking(false);
  };

  return (
    <main className="startup-screen">
      <div className="startup-brand">
        <span className="brand-mark"><span /></span>
        <span><strong>GlassScout</strong><small>FM26</small></span>
      </div>

      <header className="startup-heading">
        <h1>Choose your data world</h1>
        <p>GlassScout keeps every recommendation tied to what your recruitment team actually knows.</p>
      </header>

      <section className="mode-grid">
        <article className="mode-card mode-card-active">
          <div className="mode-card-title">
            <span className="mode-icon">FM<br />26</span>
            <div><h2>Football Manager 2026</h2><strong>Supported now</strong></div>
          </div>
          <p>Connect to a running FM26 save through a local, read-only memory adapter.</p>
          <ul>
            <li><UsersRound />Live squad and player data</li>
            <li><ShieldCheck />Scout knowledge preserved</li>
            <li><LockKeyhole />No game-state writes</li>
          </ul>
          <Button onClick={connect} disabled={checking}>
            <Database data-icon="inline-start" />
            {checking ? "Checking FM26…" : "Connect to FM26"}
          </Button>
          <small>FM data reflects the game database, not guaranteed real-life accuracy.</small>
        </article>

        <article className="mode-card mode-card-disabled" data-disabled="true">
          <div className="mode-card-title">
            <span className="mode-icon"><Globe2 /></span>
            <div><h2>Real-life Team</h2><strong>Future mode</strong></div>
          </div>
          <p>Will require a licensed football data provider such as FotMob or similar.</p>
          <ul>
            <li><UsersRound />Live real-world squad data</li>
            <li><ShieldCheck />Provider-backed scouting evidence</li>
            <li><LockKeyhole />Licensed data access</li>
          </ul>
          <Button disabled>Coming later</Button>
          <small>No real-life data is simulated or bundled.</small>
        </article>
      </section>

      <section className="startup-diagnostics" aria-live="polite">
        <h2>Connection diagnostics</h2>
        <div>
          <span><CircleDot /><strong>Process</strong><b className={status.processDetected ? "diag-good" : ""}>{status.processDetected ? `Detected · PID ${status.processId}` : "Waiting"}</b></span>
          <span><Database /><strong>Memory access</strong><b className={status.memoryAccess === "read_only_handle_open" ? "diag-good" : ""}>{status.memoryAccess.replaceAll("_", " ")}</b></span>
          <span><Check /><strong>Parser</strong><b className={status.parserStatus === "ready" ? "diag-good" : "diag-watch"}>{status.parserStatus}</b></span>
          <span><CircleDot /><strong>Loaded</strong><b>{status.playersLoaded} players · {status.clubsLoaded} clubs</b></span>
        </div>
        <p>{status.message}</p>
        {status.state !== "not_checked" ? (
          <Button variant="outline" onClick={() => onEnter(status)}>
            Open FM26 workspace
          </Button>
        ) : null}
      </section>

      <footer className="trust-strip">
        <span><Check />Local-first</span>
        <span><LockKeyhole />Read-only</span>
        <span><ShieldCheck />No simulated data</span>
      </footer>
    </main>
  );
}
