"use client";

import { useState } from "react";
import { Check, CircleDot, Database, Download, ExternalLink, Globe2, LockKeyhole, MonitorDown, ShieldCheck, UsersRound } from "lucide-react";
import { Button, buttonVariants } from "@/components/ui/button";
import type { LiveConnectorStatus } from "@/domain/adapters";
import { windowsInstallerUrl, windowsReleaseUrl } from "@/domain/distribution";
import { cn } from "@/lib/utils";

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
  const browserPreview = status.message.startsWith("Browser previews cannot inspect");

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
        <p>Use the browser for a safe preview, or install the Windows app for local FM26 access.</p>
      </header>

      <section className="mode-grid">
        <article className="mode-card mode-card-active">
          <div className="mode-card-title">
            <span className="mode-icon">FM<br />26</span>
            <div><h2>Football Manager 2026</h2><strong>Windows test build</strong></div>
          </div>
          <p>The installed app runs locally and can request read-only access to the active FM26 process.</p>
          <ul>
            <li><UsersRound />Desktop squad and player pipeline</li>
            <li><MonitorDown />Windows setup includes GlassScout</li>
            <li><LockKeyhole />No game-state writes</li>
          </ul>
          <div className="mode-actions">
            <a className={cn(buttonVariants({ size: "lg" }), "setup-download")} href={windowsInstallerUrl}>
              <Download data-icon="inline-start" />Download Windows setup
            </a>
            <Button variant="outline" size="lg" onClick={connect} disabled={checking}>
              <Database data-icon="inline-start" />
              {checking ? "Checking…" : "Check this environment"}
            </Button>
          </div>
          <small>Windows 10/11 x64 · test prerelease · WebView2 is installed automatically when required.</small>
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
        <h2>{browserPreview ? "Browser preview" : "Connection diagnostics"}</h2>
        <div className="diagnostic-cells">
          <span><CircleDot /><strong>Process</strong><b className={status.processDetected ? "diag-good" : ""}>{status.processDetected ? `Detected · PID ${status.processId}` : "Waiting"}</b></span>
          <span><Database /><strong>Memory access</strong><b className={status.memoryAccess === "read_only_handle_open" ? "diag-good" : ""}>{status.memoryAccess.replaceAll("_", " ")}</b></span>
          <span><Check /><strong>Parser</strong><b className={status.parserStatus === "ready" ? "diag-good" : "diag-watch"}>{status.parserStatus}</b></span>
          <span><CircleDot /><strong>Loaded</strong><b>{status.playersLoaded} players · {status.clubsLoaded} clubs</b></span>
        </div>
        <p>{status.message}</p>
        {status.state !== "not_checked" ? (
          <div className="diagnostic-actions">
            {browserPreview ? (
              <a className={buttonVariants()} href={windowsInstallerUrl}><Download data-icon="inline-start" />Install desktop connector</a>
            ) : null}
            <Button variant="outline" onClick={() => onEnter(status)}>
              {browserPreview ? "Continue browser preview" : "Open FM26 workspace"}
            </Button>
          </div>
        ) : (
          <a className="release-notes-link" href={windowsReleaseUrl} target="_blank" rel="noreferrer">Release notes <ExternalLink /></a>
        )}
      </section>

      <footer className="trust-strip">
        <span><Check />Local-first</span>
        <span><LockKeyhole />Read-only</span>
        <span><ShieldCheck />No simulated data</span>
      </footer>
    </main>
  );
}
