export type KnowledgeState = "known" | "estimated" | "ranged" | "unknown" | "blocked";

export type ScoutedField<T> = {
  value: T | null;
  range?: [number, number];
  state: KnowledgeState;
  confidence: number;
  source: "live-visible" | "scout-report" | "coach-report" | "analyst-report";
  lastSeen: string | null;
};

export type KnowledgeProfile = {
  known: number;
  estimated: number;
  unknown: number;
  confidence: number;
  reliability: "Weak" | "Fair" | "Good" | "Excellent";
};

export function buildKnowledgeProfile(fields: ScoutedField<unknown>[]): KnowledgeProfile {
  const usable = fields.filter((field) => field.state !== "blocked");
  if (!usable.length) return { known: 0, estimated: 0, unknown: 100, confidence: 0, reliability: "Weak" };

  const knownCount = usable.filter((field) => field.state === "known").length;
  const estimatedCount = usable.filter((field) => field.state === "estimated" || field.state === "ranged").length;
  const unknownCount = usable.filter((field) => field.state === "unknown").length;
  const confidence = Math.round(usable.reduce((sum, field) => sum + field.confidence, 0) / usable.length);
  const reliability = confidence >= 88 ? "Excellent" : confidence >= 68 ? "Good" : confidence >= 45 ? "Fair" : "Weak";

  return {
    known: Math.round((knownCount / usable.length) * 100),
    estimated: Math.round((estimatedCount / usable.length) * 100),
    unknown: Math.round((unknownCount / usable.length) * 100),
    confidence,
    reliability,
  };
}

export function describeField(field: ScoutedField<number | string>) {
  if (field.state === "blocked" || field.state === "unknown") return "Unknown";
  if (field.range) return `${field.range[0]}–${field.range[1]}`;
  return field.value == null ? "Unknown" : String(field.value);
}
