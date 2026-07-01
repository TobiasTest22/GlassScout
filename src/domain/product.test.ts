import { describe, expect, it } from "vitest";
import { buildKnowledgeProfile, describeField } from "./scout-knowledge";
import { groupPlayerPosition, resolveFavorites, toggleFavorite, updateFavoriteNote } from "./live-data";
import type { LiveConnectorStatus, LivePlayer } from "./adapters";
import { mergeParsedExports, parseFm26Export } from "./export-watcher";
import { estimateTruePrice, evaluateRoleDna } from "./player-evaluation";

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

describe("FM26 export fallback", () => {
  const visibleExport = [
    "Name;Age;Club;Position;Value;Wage;Contract Expiry;Minutes;Goals;Assists;Average Rating;Passing;Vision;Decisions;Stamina;Work Rate;Tackling;PA",
    "Alex Example;22;Test FC;ST;€10M;€20K p/w;Expiring 2027;900;4;6;7.20;16;17;15;15;16;14;199",
  ].join("\n");

  it("parses semicolon FM26 exports and blocks hidden-value columns", () => {
    const parsed = parseFm26Export(visibleExport, "Squad_2026.csv", 1);
    expect(parsed.players).toHaveLength(1);
    expect(parsed.players[0]).toMatchObject({
      name: "Alex Example",
      currentAbility: null,
      potentialAbility: null,
      scoutKnowledge: "partly_known",
    });
    expect(parsed.players[0].attributes?.passing).toBe(16);
    expect(parsed.diagnostics.blockedColumns).toEqual(["PA"]);
    expect(parsed.players[0].per90?.goalsPer90).toBe(0.4);
  });

  it("turns imported visible data into a usable snapshot", () => {
    const parsed = parseFm26Export(visibleExport, "Squad_2026.csv", 1);
    const status: LiveConnectorStatus = {
      processDetected: true, processId: 1, processPath: "fm.exe", saveDetected: null,
      memoryAccess: "read_only_handle_open", parserStatus: "unverified", state: "parser_unverified",
      playersLoaded: 0, clubsLoaded: 0, lastSync: null, bytesRead: 2, executableHeaderValid: true,
      canWriteMemory: false, message: "Entity map missing", warnings: [],
    };
    const merged = mergeParsedExports([parsed], status);
    expect(merged.snapshot).toMatchObject({ dataSource: "export-watcher", dataError: null });
    expect(merged.snapshot.status.state).toBe("connected");
    expect(merged.snapshot.players).toHaveLength(1);
    expect(merged.snapshot.managedClubId).toBeTruthy();
  });
});

describe("transparent player evaluation", () => {
  it("can recommend a position outside the player's current listing from visible attributes", () => {
    const result = evaluateRoleDna({ passing: 17, vision: 18, decisions: 16, firsttouch: 10, positioning: 18, teamwork: 18, composure: 17 });
    expect(result.position).toBe("DM");
    expect(result.score).toBeGreaterThan(70);
    expect(result.reasoning.length).toBeGreaterThan(0);
  });

  it("does not estimate true price without a visible FM market value", () => {
    expect(estimateTruePrice({ marketValueAmount: null, age: 22, averageRating: 7.2, minutesPlayed: 900, roleFit: 80, contractStatus: null }).estimate).toBeNull();
  });
});
