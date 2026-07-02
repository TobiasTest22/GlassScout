"use client";

import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type ClubLogoResult = { found: boolean; clubId: string; dataUrl: string | null };
const cache = new Map<string, string | null>();

export function ClubLogo({ clubId, name, size = "md" }: { clubId: string; name: string; size?: "sm" | "md" | "lg" }) {
  const [source, setSource] = useState<string | null | undefined>(() => {
    if (cache.has(clubId)) return cache.get(clubId);
    if (typeof window !== "undefined" && !("__TAURI_INTERNALS__" in window)) return null;
    return undefined;
  });
  useEffect(() => {
    if (cache.has(clubId)) return;
    if (!("__TAURI_INTERNALS__" in window)) {
      cache.set(clubId, null);
      return;
    }
    let active = true;
    invoke<ClubLogoResult>("club_logo_data", { clubId }).then((result) => {
      const next = result.found ? result.dataUrl : null;
      cache.set(clubId, next);
      if (active) setSource(next);
    }).catch(() => { cache.set(clubId, null); if (active) setSource(null); });
    return () => { active = false; };
  }, [clubId]);
  if (!source) return null;
  return <span className={`club-logo club-logo-${size}`}><img src={source} alt={`${name} badge from FM club ID ${clubId}`} /></span>;
}
