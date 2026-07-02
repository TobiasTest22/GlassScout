"use client";

import { useEffect, useState } from "react";
import { Check, Database, Download, LockKeyhole, RefreshCw, ShieldCheck } from "lucide-react";
import { Button, buttonVariants } from "@/components/ui/button";
import type { LiveConnectorStatus } from "@/domain/adapters";
import { windowsInstallerUrl } from "@/domain/distribution";
import { cn } from "@/lib/utils";

const loadStageLabels: Record<string, string> = {
  detecting_fm26: "Detecting FM26…",
  validating_active_save: "Validating active save…",
  reading_managed_club: "Reading managed club…",
  loading_managed_squad: "Loading managed squad…",
  indexing_player_database: "Indexing wider player database…",
  building_visibility_index: "Building visibility-safe index…",
  ready: "Ready",
};

export function StartupScreen({
  onConnect,
  onEnter,
}: {
  onConnect: () => Promise<LiveConnectorStatus>;
  onEnter: (status: LiveConnectorStatus) => void;
}) {
  const [desktopRuntime, setDesktopRuntime] = useState(false);
  const [loading, setLoading] = useState(false);
  const [status, setStatus] = useState<LiveConnectorStatus | null>(null);
  const [loadStage, setLoadStage] = useState("detecting_fm26");

  useEffect(() => {
    setDesktopRuntime("__TAURI_INTERNALS__" in window);
  }, []);

  useEffect(() => {
    if (!desktopRuntime) return;
    let remove: (() => void) | null = null;
    void import("@tauri-apps/api/event")
      .then(({ listen }) => listen<string>("glassscout-load-progress", (event) => {
        setLoadStage(event.payload);
      }))
      .then((unlisten) => {
        remove = unlisten;
      });
    return () => remove?.();
  }, [desktopRuntime]);

  const loadActiveSave = async () => {
    setLoading(true);
    setLoadStage("detecting_fm26");
    try {
      const next = await onConnect();
      setStatus(next);
      if (next.state === "connected") onEnter(next);
    } finally {
      setLoading(false);
    }
  };

  return (
    <main className="startup-screen live-startup">
      <div className="startup-brand">
        <span className="brand-mark"><span /></span>
        <span><strong>GlassScout</strong><small>FM26</small></span>
      </div>

      <section className="live-start-card">
        <span className="live-start-icon">{desktopRuntime ? <Database /> : <ShieldCheck />}</span>
        <p className="section-kicker">Live FM26 connection</p>
        <h1>{desktopRuntime ? "Load your active FM26 save" : "Install GlassScout for Windows"}</h1>
        <p>
          {desktopRuntime
            ? "Open FM26 and load your save, then start a read-only scan. GlassScout loads your club and squad first, then indexes the wider player database behind strict visibility gates."
            : "GlassScout reads the active FM26 game through its installed Windows desktop connector."}
        </p>
        {desktopRuntime ? (
          <Button onClick={loadActiveSave} disabled={loading}>
            {loading ? <RefreshCw data-icon="inline-start" className="spin" /> : <Database data-icon="inline-start" />}
            {loading ? loadStageLabels[loadStage] ?? "Reading active save…" : "Load Active Save"}
          </Button>
        ) : (
          <a className={cn(buttonVariants({ size: "lg" }), "setup-download")} href={windowsInstallerUrl}>
            <Download data-icon="inline-start" />Download GlassScout for Windows
          </a>
        )}
        <div className="live-start-trust">
          <span><Check />Active game only</span>
          <span><LockKeyhole />Read-only access</span>
          <span><ShieldCheck />No simulated players</span>
        </div>
        {status && status.state !== "connected" ? (
          <p className="load-failure" role="alert">{status.message}</p>
        ) : null}
      </section>
    </main>
  );
}
