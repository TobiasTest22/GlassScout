"use client";

import { useEffect, useState } from "react";
import { UserRound } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { cn } from "@/lib/utils";

type PlayerFaceResult = {
  found: boolean;
  playerId: string;
  dataUrl: string | null;
  source: "fm-unique-id" | "fallback";
};

const faceCache = new Map<string, string | null>();

export function PlayerFace({ playerId, name, size = "md", highResolution = false }: {
  playerId: string;
  name: string;
  size?: "sm" | "md" | "lg";
  highResolution?: boolean;
}) {
  const useIcon = size === "sm" && !highResolution;
  const cacheKey = `${playerId}:${useIcon ? "icon" : "portrait"}`;
  const [source, setSource] = useState<string | null | undefined>(() => faceCache.get(cacheKey));

  useEffect(() => {
    if (faceCache.has(cacheKey) || !("__TAURI_INTERNALS__" in window)) return;
    let active = true;
    invoke<PlayerFaceResult>("player_face_data", { playerId, icon: useIcon })
      .then((result) => {
        const next = result.found ? result.dataUrl : null;
        faceCache.set(cacheKey, next);
        if (active) setSource(next);
      })
      .catch(() => {
        faceCache.set(cacheKey, null);
        if (active) setSource(null);
      });
    return () => { active = false; };
  }, [cacheKey, playerId, useIcon]);

  return (
    <span className={cn("player-face", `player-face-${size}`)} title={`${name} · FM ID ${playerId}`}>
      {source ? <img src={source} alt={`${name} portrait`} /> : <UserRound aria-label="No player face available" />}
    </span>
  );
}
