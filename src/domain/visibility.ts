export type VisibilityStatus =
  | "unknown"
  | "rumoured"
  | "estimated"
  | "ranged"
  | "scout_confirmed"
  | "coach_confirmed"
  | "analyst_confirmed"
  | "fully_visible"
  | "blocked_hidden";

export type Observation = {
  field: string;
  value: number | string | null;
  rawValue?: number | string | null;
  range?: [number, number];
  source: string;
  confidence: number;
  visibilityStatus: VisibilityStatus;
  dateSeen: string;
  playerId: string;
  isHiddenBlocked: boolean;
  isEstimated: boolean;
  isRange: boolean;
  isExactVisible: boolean;
};

const usableStatuses = new Set<VisibilityStatus>([
  "rumoured",
  "estimated",
  "ranged",
  "scout_confirmed",
  "coach_confirmed",
  "analyst_confirmed",
  "fully_visible",
]);

export function filterVisibleObservations(observations: Observation[]) {
  return observations.flatMap((observation) => {
    if (
      observation.isHiddenBlocked ||
      observation.visibilityStatus === "blocked_hidden" ||
      !usableStatuses.has(observation.visibilityStatus)
    ) {
      return [];
    }

    const safeObservation = { ...observation };
    delete safeObservation.rawValue;
    return [safeObservation];
  });
}

export type RoleWeight = { field: string; weight: number };

export type RoleScore = {
  min: number | null;
  max: number | null;
  estimate: number | null;
  confidence: number;
  label: "Elite fit" | "Strong fit" | "Good fit" | "Possible fit" | "Unclear" | "Poor fit" | "Not enough information";
  missingKeyAttributes: string[];
};

export function scoreRole(observations: Observation[], weights: RoleWeight[]): RoleScore {
  const safe = filterVisibleObservations(observations);
  const byField = new Map(safe.map((item) => [item.field, item]));
  let weightedMin = 0;
  let weightedMax = 0;
  let availableWeight = 0;
  let confidenceWeight = 0;

  for (const roleWeight of weights) {
    const observation = byField.get(roleWeight.field);
    if (!observation) continue;
    const range = observation.range ?? (
      typeof observation.value === "number" ? [observation.value, observation.value] as [number, number] : undefined
    );
    if (!range) continue;
    weightedMin += range[0] * roleWeight.weight;
    weightedMax += range[1] * roleWeight.weight;
    availableWeight += roleWeight.weight;
    confidenceWeight += observation.confidence * roleWeight.weight;
  }

  const totalWeight = weights.reduce((sum, item) => sum + item.weight, 0);
  const missingKeyAttributes = weights.filter((item) => !byField.has(item.field)).map((item) => item.field);
  const coverage = totalWeight ? availableWeight / totalWeight : 0;

  if (coverage < 0.4 || !availableWeight) {
    return { min: null, max: null, estimate: null, confidence: Math.round(coverage * 100), label: "Not enough information", missingKeyAttributes };
  }

  const min = Math.round((weightedMin / availableWeight / 20) * 100);
  const max = Math.round((weightedMax / availableWeight / 20) * 100);
  const estimate = Math.round((min + max) / 2);
  const confidence = Math.round((confidenceWeight / availableWeight) * coverage);
  const label = confidence < 45 ? "Unclear" : estimate >= 88 ? "Elite fit" : estimate >= 78 ? "Strong fit" : estimate >= 67 ? "Good fit" : estimate >= 55 ? "Possible fit" : "Poor fit";

  return { min, max, estimate, confidence, label, missingKeyAttributes };
}

export function buildAssistantContext(observations: Observation[]) {
  return filterVisibleObservations(observations).map(({ field, value, range, source, confidence, visibilityStatus }) => ({
    field,
    value,
    range,
    source,
    confidence,
    visibilityStatus,
  }));
}
