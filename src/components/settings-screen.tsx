"use client";

import { ChevronDown, Cpu, Database, LockKeyhole, RefreshCw } from "lucide-react";
import type { LiveFootballSnapshot } from "@/domain/adapters";
import { Button } from "@/components/ui/button";

export function SettingsScreen({
  snapshot,
  checking,
  onRefresh,
}: {
  snapshot: LiveFootballSnapshot;
  checking: boolean;
  onRefresh: () => Promise<unknown>;
}) {
  const status = snapshot.status;
  return (
    <main className="screen settings-screen">
      <div className="planner-heading">
        <div><h1>Settings</h1><p>Live FM26 connector and local application controls.</p></div>
      </div>
      <section className="settings-list">
        <article><Database /><div><strong>Active FM26 game</strong><span>{status.processDetected ? "Football Manager 26 detected" : "Waiting for FM26"}</span></div><Button variant="outline" onClick={onRefresh} disabled={checking}><RefreshCw data-icon="inline-start" className={checking ? "spin" : undefined} />Load Active Save</Button></article>
        <article><LockKeyhole /><div><strong>Memory safety</strong><span>Query and read access only. GlassScout cannot write to FM26.</span></div><b>{status.memoryAccess.replaceAll("_", " ")}</b></article>
      </section>

      <details className="advanced-diagnostics">
        <summary><span><Cpu />Advanced diagnostics</span><ChevronDown /></summary>
        <p>Technical connection and visibility-gate details for troubleshooting supported FM26 builds.</p>
        <dl>
          <div><dt>Process</dt><dd>{status.processDetected ? `Detected · PID ${status.processId}` : "Not detected"}</dd></div>
          <div><dt>Executable</dt><dd>{status.processPath ?? "Unavailable"}</dd></div>
          <div><dt>FM26 build</dt><dd>{status.gameBuild ?? "Unavailable"}</dd></div>
          <div><dt>Product version</dt><dd>{status.productVersion ?? "Unavailable"}</dd></div>
          <div><dt>Architecture</dt><dd>{status.architecture ?? "Unavailable"}</dd></div>
          <div><dt>Module base</dt><dd>{status.moduleBase ?? "Unavailable"}</dd></div>
          <div><dt>Memory probe</dt><dd>{status.executableHeaderValid ? `Passed · ${status.bytesRead} bytes` : "Not verified"}</dd></div>
          <div><dt>Entity map</dt><dd>{status.entityMapStatus === "matched" ? status.entityMapProfileId : status.entityMapStatus ?? "Not checked"}</dd></div>
          <div><dt>Pointer validation</dt><dd>{status.pointerValidation?.replaceAll("_", " ") ?? "Not run"}</dd></div>
          <div><dt>Active save</dt><dd>{status.saveDetected === true ? "Detected" : "Not readable"}</dd></div>
          <div><dt>Read-only access flags</dt><dd>{status.handleAccessFlags ?? "Unavailable"}</dd></div>
          <div><dt>Manager registry</dt><dd>{status.entityRoot ?? "Unavailable"}</dd></div>
          <div><dt>Active manager</dt><dd>{status.savePointer ?? "Unavailable"}</dd></div>
          <div><dt>Managed club</dt><dd>{status.managedClubPointer ?? "Unavailable"}</dd></div>
          <div><dt>Squad collection</dt><dd>{status.playerCollectionPointer ?? "Unavailable"}</dd></div>
          <div><dt>Database index</dt><dd>{status.databaseIndexStatus.replaceAll("_", " ")}</dd></div>
          <div><dt>Database scope</dt><dd>{status.databaseScope.replaceAll("-", " ")}</dd></div>
          <div><dt>Managed squad players</dt><dd>{status.managedSquadPlayers}</dd></div>
          <div><dt>Player records indexed</dt><dd>{status.databasePlayersIndexed}</dd></div>
          <div><dt>Background records gated</dt><dd>{status.backgroundPlayersIndexed}</dd></div>
          <div><dt>Visibility-safe players</dt><dd>{status.visiblePlayersLoaded}</dd></div>
          <div><dt>Fully scouted players</dt><dd>{status.fullyScoutedPlayers}</dd></div>
          <div><dt>Partial scout reports</dt><dd>{status.partialScoutReports}</dd></div>
          <div><dt>Live memory tactic read</dt><dd>{status.liveMemoryTacticRead ?? "disabled"}</dd></div>
          <div><dt>Tactic manager</dt><dd>{status.tacticManagerPointer ?? "Unavailable"}</dd></div>
          <div><dt>Tactic source</dt><dd>{snapshot.tacticSource.replaceAll("_", " ")}</dd></div>
          <div><dt>Last successful read</dt><dd>{status.lastSuccessfulRead?.replaceAll("_", " ") ?? "None"}</dd></div>
          <div><dt>Failure stage</dt><dd>{status.failureStage?.replaceAll("_", " ") ?? "None"}</dd></div>
          <div><dt>Windows error</dt><dd>{status.windowsErrorCode ?? "None"}</dd></div>
          <div className="diagnostic-hash"><dt>Executable SHA-256</dt><dd>{status.executableSha256 ?? "Unavailable"}</dd></div>
        </dl>
        <div className="advanced-diagnostic-message"><strong>Connector result</strong><span>{status.message}</span></div>
      </details>
    </main>
  );
}
