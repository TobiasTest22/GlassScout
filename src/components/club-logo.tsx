"use client";

import { useEffect, useState } from "react";
import { Shield } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";

type ClubLogoResult = { found: boolean; clubId: string; dataUrl: string | null };
const cache = new Map<string, string | null>();

export function ClubLogo({ clubId, name, size = "md" }: { clubId: string; name: string; size?: "sm" | "md" | "lg" }) {
  const [source, setSource] = useState<string | null | undefined>(() => cache.get(clubId));
  useEffect(() => {
    if (cache.has(clubId) || !("__TAURI_INTERNALS__" in window)) return;
    let active = true;
    invoke<ClubLogoResult>("club_logo_data", { clubId }).then((result) => {
      const next = result.found ? result.dataUrl : null;
      cache.set(clubId, next);
      if (active) setSource(next);
    }).catch(() => { cache.set(clubId, null); if (active) setSource(null); });
    return () => { active = false; };
  }, [clubId]);
  return <span className={`club-logo club-logo-${size}`}>{source ? <img src={source} alt={`${name} badge`} /> : <Shield aria-label="Club badge unavailable" />}</span>;
}
