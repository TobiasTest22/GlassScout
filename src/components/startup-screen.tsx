"use client";

import { useEffect } from "react";
import { Check, Download, LockKeyhole, RefreshCw, ShieldCheck } from "lucide-react";
import { Button, buttonVariants } from "@/components/ui/button";
import type { LiveConnectorStatus } from "@/domain/adapters";
import { windowsInstallerUrl } from "@/domain/distribution";
import { cn } from "@/lib/utils";

export function StartupScreen({
  onConnect,
  onEnter,
}: {
  onConnect: () => Promise<LiveConnectorStatus>;
  onEnter: (status: LiveConnectorStatus) => void;
}) {
  const desktopRuntime = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

  useEffect(() => {
    if (!desktopRuntime) return;
    let active = true;
    void onConnect().then((status) => {
      if (active) onEnter(status);
    });
    return () => {
      active = false;
    };
  }, [desktopRuntime, onConnect, onEnter]);

  return (
    <main className="startup-screen live-startup">
      <div className="startup-brand">
        <span className="brand-mark"><span /></span>
        <span><strong>GlassScout</strong><small>FM26</small></span>
      </div>

      <section className="live-start-card">
        <span className="live-start-icon"><ShieldCheck /></span>
        <p className="section-kicker">Live FM26 connection</p>
        <h1>{desktopRuntime ? "Connecting to Football Manager 26" : "Install GlassScout for Windows"}</h1>
        <p>
          {desktopRuntime
            ? "Open FM26 and load your save. GlassScout will detect the active game and validate live access automatically."
            : "GlassScout reads the active FM26 game through its installed Windows desktop connector."}
        </p>
        {desktopRuntime ? (
          <Button disabled>
            <RefreshCw data-icon="inline-start" className="spin" />
            Checking live connection…
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
      </section>
    </main>
  );
}
