import { describe, expect, it } from "vitest";
import { buildKnowledgeProfile, describeField } from "./scout-knowledge";
import { groupPlayerPosition, resolveFavorites, toggleFavorite, updateFavoriteNote } from "./live-data";
import type { LivePlayer } from "./adapters";

describe("scouting knowledge", () => {
  it("keeps unknown information unknown and excludes blocked fields from coverage", () => {
    const fields = [
      { value: 15, state: "known" as const, confidence: 90, source: "scout-report" as const, lastSeen: "2026-07-01" },
      { value: null, range: [11, 14] as [number, number], state: "ranged" as const, confidence: 65, source: "scout-report" as const, lastSeen: "2026-07-01" },
      { value: null, state: "unknown" as const, confidence: 0, source: "scout-report" as const, lastSeen: null },
      { value: 190, state: "blocked" as const, confidence: 100, source: "live-visible" as const, lastSeen: "2026-07-01" },
    ];
    const profile = buildKnowledgeProfile(fields);
    expect(profile).toMatchObject({ known: 33, estimated: 33, unknown: 33 });
    expect(describeField(fields[1])).toBe("11–14");
    expect(describeField(fields[2])).toBe("Unknown");
    expect(describeField(fields[3])).toBe("Unknown");
  });
});

const livePlayer: LivePlayer = {
    id: "1",
    name: "Player One",
    age: 21,
    nationality: "France",
    positions: ["CB"],
    bestRole: null,
    currentAbility: null,
    potentialAbility: null,
    form: null,
    averageRating: null,
    minutesPlayed: null,
    goals: null,
    assists: null,
    contractStatus: null,
    value: null,
    wage: null,
    squadImportance: null,
    developmentTrend: null,
    tacticalFit: null,
    roleFit: 78,
    transferInterest: "Somewhat interested",
    loanInterest: "Unknown",
    transferAvailable: true,
    loanAvailable: false,
    strengths: [],
    weaknesses: [],
    clubId: "club-1",
  };

describe("live squad grouping", () => {
  it("groups extracted players by their live position data", () => {
    expect(groupPlayerPosition(livePlayer)).toBe("Centre-backs");
    expect(groupPlayerPosition({ ...livePlayer, positions: ["DM", "CM"] })).toBe("Defensive midfielders");
  });
});

describe("favorites", () => {
  it("adds, annotates and removes a live player reference", () => {
    const added = toggleFavorite([], livePlayer.id);
    expect(added).toEqual([{ playerId: "1", note: "" }]);
    const noted = updateFavoriteNote(added, livePlayer.id, "Watch role fit");
    expect(resolveFavorites(noted, [livePlayer])).toEqual([{ player: livePlayer, note: "Watch role fit" }]);
    expect(toggleFavorite(noted, livePlayer.id)).toEqual([]);
  });

  it("does not preserve stale player payloads when a live entity disappears", () => {
    expect(resolveFavorites([{ playerId: "missing", note: "Old target" }], [livePlayer])).toEqual([]);
  });
});
