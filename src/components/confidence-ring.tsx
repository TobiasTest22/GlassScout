"use client";

import { cn } from "@/lib/utils";

export function ConfidenceRing({ value, size = "md", tone = "mint" }: { value: number; size?: "sm" | "md" | "lg"; tone?: "mint" | "amber" }) {
  const degree = Math.max(0, Math.min(100, value)) * 3.6;
  return (
    <div
      className={cn("confidence-ring", `confidence-ring-${size}`, tone === "amber" && "confidence-ring-amber")}
      style={{ "--ring-progress": `${degree}deg` } as React.CSSProperties}
      aria-label={`${value}% confidence`}
    >
      <span>{value}</span>
    </div>
  );
}
