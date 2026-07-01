import { describe, expect, it } from "vitest";
import { buildAssistantContext, filterVisibleObservations, scoreRole, type Observation } from "./visibility";

const base: Observation = {
  field: "passing",
  value: 15,
  rawValue: 18,
  source: "scout-report",
  confidence: 82,
  visibilityStatus: "scout_confirmed",
  dateSeen: "2026-07-01",
  playerId: "p1",
  isHiddenBlocked: false,
  isEstimated: false,
  isRange: false,
  isExactVisible: true,
};

describe("visibility boundary", () => {
  it("removes blocked hidden observations and strips raw values", () => {
    const result = filterVisibleObservations([
      base,
      { ...base, field: "potentialAbility", visibilityStatus: "blocked_hidden", isHiddenBlocked: true },
    ]);
    expect(result).toHaveLength(1);
    expect(result[0]).not.toHaveProperty("rawValue");
  });

  it("keeps ranges as ranges", () => {
    const [result] = filterVisibleObservations([
      { ...base, value: null, range: [12, 16], isRange: true, isExactVisible: false, visibilityStatus: "ranged" },
    ]);
    expect(result.range).toEqual([12, 16]);
  });

  it("refuses precise role scores when information is sparse", () => {
    const result = scoreRole([base], [
      { field: "passing", weight: 1 },
      { field: "vision", weight: 1 },
      { field: "decisions", weight: 1 },
    ]);
    expect(result.label).toBe("Not enough information");
    expect(result.estimate).toBeNull();
  });

  it("scores visible exact and ranged observations without blocked values", () => {
    const observations = [
      base,
      { ...base, field: "vision", value: null, range: [13, 17] as [number, number], visibilityStatus: "ranged" as const, isRange: true, isExactVisible: false },
      { ...base, field: "decisions", value: 14 },
      { ...base, field: "potentialAbility", value: 20, visibilityStatus: "blocked_hidden" as const, isHiddenBlocked: true },
    ];
    const result = scoreRole(observations, [
      { field: "passing", weight: 1 },
      { field: "vision", weight: 1 },
      { field: "decisions", weight: 1 },
    ]);
    expect(result.min).toBeLessThan(result.max!);
    expect(result.missingKeyAttributes).not.toContain("potentialAbility");
  });

  it("never includes blocked values in assistant context", () => {
    const context = buildAssistantContext([
      base,
      { ...base, field: "consistency", value: 20, visibilityStatus: "blocked_hidden", isHiddenBlocked: true },
    ]);
    expect(JSON.stringify(context)).not.toContain("consistency");
    expect(JSON.stringify(context)).not.toContain("rawValue");
  });
});
