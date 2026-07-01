"use client";

import { Database, LockKeyhole, RefreshCw } from "lucide-react";
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
  return (
    <main className="screen settings-screen">
      <div className="planner-heading"><div><h1>Settings</h1><p>Desktop connector and local-data controls.</p></div></div>
      <section className="settings-list">
        <article><Database /><div><strong>FM26 executable</strong><span>{snapshot.status.processPath ?? "Not detected"}</span></div><Button variant="outline" onClick={onRefresh} disabled={checking}><RefreshCw data-icon="inline-start" />Check now</Button></article>
        <article><LockKeyhole /><div><strong>Memory permissions</strong><span>Query and read access only. Writing to FM26 memory is not implemented.</span></div><b>{snapshot.status.memoryAccess.replaceAll("_", " ")}</b></article>
      </section>
    </main>
  );
}
