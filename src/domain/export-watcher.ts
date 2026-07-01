import type { LiveClub, LiveConnectorStatus, LiveFootballSnapshot, LivePlayer } from "./adapters";
import { estimateTruePrice, evaluateRoleDna } from "./player-evaluation";

export type ExportFileSummary = {
  name: string;
  type: "squad" | "player-search" | "unknown";
  modified: number;
  rows: number;
  status: "ready" | "warning" | "error";
};

export type ExportDiagnostics = {
  files: ExportFileSummary[];
  columnsMatched: number;
  sourceColumns: number;
  missingFields: string[];
  blockedColumns: string[];
  importedAt: string | null;
  warnings: string[];
};

export type ParsedExport = {
  players: LivePlayer[];
  clubs: LiveClub[];
  diagnostics: ExportDiagnostics;
};

const blockedHeaders = new Set([
  "ca", "pa", "currentability", "potentialability", "hiddenattributes",
  "consistency", "importantmatches", "injuryproneness", "professionalism",
]);

const attributeHeaders = [
  "acceleration", "agility", "balance", "jumpingreach", "naturalfitness", "pace", "stamina", "strength",
  "corners", "crossing", "dribbling", "finishing", "firsttouch", "freekicks", "heading", "longshots",
  "longthrows", "marking", "passing", "penaltytaking", "tackling", "technique",
  "aggression", "anticipation", "bravery", "composure", "concentration", "decisions", "determination",
  "flair", "leadership", "offball", "positioning", "teamwork", "vision", "workrate",
  "aerialreach", "commandofarea", "communication", "eccentricity", "handling", "kicking",
  "oneonones", "punchingtendency", "reflexes", "rushingout", "throwing",
];

const aliases: Record<string, string[]> = {
  name: ["name", "player", "playername"],
  age: ["age"],
  nationality: ["nationality", "nation", "nat"],
  club: ["club", "team"],
  position: ["position", "positions", "pos"],
  value: ["value", "marketvalue", "fmvalue"],
  wage: ["wage", "salary"],
  contract: ["contractexpiry", "contractexpires", "expires", "contract"],
  form: ["form"],
  averageRating: ["avgrating", "averagerating", "avrat", "rating"],
  minutes: ["minutes", "mins", "min"],
  goals: ["goals", "gls"],
  assists: ["assists", "ast"],
};

export function parseFm26Export(text: string, fileName = "export.csv", modified = Date.now()): ParsedExport {
  const delimiter = detectDelimiter(text);
  const rows = parseDelimited(text, delimiter);
  if (rows.length < 2) throw new Error("The export contains no player rows.");
  const headers = rows[0].map(normalizeHeader);
  const blockedColumns = rows[0].filter((_, index) => blockedHeaders.has(headers[index]));
  const safeIndexes = headers.map((header, index) => blockedHeaders.has(header) ? -1 : index);
  const matchedHeaders = new Set(headers.filter((header) =>
    !blockedHeaders.has(header) &&
    (attributeHeaders.includes(header) || Object.values(aliases).some((values) => values.includes(header)) || per90MetricName(header))
  ));
  const players: LivePlayer[] = [];
  const clubNames = new Set<string>();
  const warnings: string[] = [];

  for (const [rowIndex, row] of rows.slice(1).entries()) {
    const safeRow = Object.fromEntries(headers.flatMap((header, index) => safeIndexes[index] < 0 ? [] : [[header, row[index]?.trim() ?? ""]]));
    const name = getAlias(safeRow, "name");
    if (!name) continue;
    const clubName = getAlias(safeRow, "club") || "Imported squad (club column missing)";
    clubNames.add(clubName);
    const attributes = Object.fromEntries(attributeHeaders.map((attribute) => [attribute, parseAttribute(safeRow[attribute])]));
    const minutesPlayed = parseNumber(getAlias(safeRow, "minutes"));
    const goals = parseNumber(getAlias(safeRow, "goals"));
    const assists = parseNumber(getAlias(safeRow, "assists"));
    const per90 = extractPer90(safeRow, minutesPlayed, goals, assists);
    const role = evaluateRoleDna(attributes);
    const positions = splitPositions(getAlias(safeRow, "position"));
    const marketValueAmount = parseMoney(getAlias(safeRow, "value"));
    const visibleAttributeCount = Object.values(attributes).filter((value) => value != null).length;
    const scoutKnowledge = visibleAttributeCount >= 18 ? "fully_known" : visibleAttributeCount >= 6 ? "partly_known" : visibleAttributeCount > 0 ? "needs_scouting" : "missing_data";
    const currentGroup = positions[0] ?? null;
    const retrainingSuggestion = role.position && currentGroup && !positionMatches(currentGroup, role.position)
      ? `Consider ${role.position} retraining; visible attributes rate ${role.score}% for ${role.role}.`
      : null;
    const basePlayer: LivePlayer = {
      id: stableId(`${name}-${clubName}-${rowIndex}`),
      name,
      age: parseNumber(getAlias(safeRow, "age")),
      nationality: getAlias(safeRow, "nationality") || null,
      positions,
      bestRole: role.role,
      currentAbility: null,
      potentialAbility: null,
      form: getAlias(safeRow, "form") || null,
      averageRating: parseNumber(getAlias(safeRow, "averageRating")),
      minutesPlayed,
      goals,
      assists,
      contractStatus: getAlias(safeRow, "contract") || null,
      value: getAlias(safeRow, "value") || null,
      wage: getAlias(safeRow, "wage") || null,
      squadImportance: null,
      developmentTrend: null,
      tacticalFit: role.score,
      roleFit: role.score,
      strengths: role.reasoning,
      weaknesses: [],
      clubId: stableId(clubName),
      transferInterest: null,
      loanInterest: null,
      transferAvailable: null,
      loanAvailable: null,
      attributes,
      per90,
      scoutKnowledge,
      bestCalculatedPosition: role.position,
      retrainingSuggestion,
      roleReasoning: role.reasoning,
      riskLevel: scoutKnowledge === "fully_known" ? "low" : scoutKnowledge === "partly_known" ? "medium" : "high",
      marketValueAmount,
    };
    const valuation = estimateTruePrice(basePlayer);
    players.push({
      ...basePlayer,
      truePrice: valuation.estimate,
      fairPriceRange: valuation.range,
      valuationLabel: valuation.label,
      valuationReasoning: valuation.reasoning,
    });
  }
  if (!players.length) throw new Error("No player names were found. Include a Name column in the FM26 export.");
  if (clubNames.has("Imported squad (club column missing)")) warnings.push("Club column missing; the dataset is labelled as an imported squad.");
  if (blockedColumns.length) warnings.push(`${blockedColumns.length} hidden-value column(s) were blocked.`);

  const clubs = [...clubNames].map((name) => ({ id: stableId(name), name, nation: null, league: null }));
  const missingFields = (Object.keys(aliases) as Array<keyof typeof aliases>).filter((field) => !aliases[field].some((alias) => headers.includes(alias)));
  return {
    players,
    clubs,
    diagnostics: {
      files: [{ name: fileName, type: /squad/i.test(fileName) ? "squad" : /search|player/i.test(fileName) ? "player-search" : "unknown", modified, rows: players.length, status: warnings.length ? "warning" : "ready" }],
      columnsMatched: matchedHeaders.size,
      sourceColumns: headers.length,
      missingFields,
      blockedColumns,
      importedAt: new Date().toISOString(),
      warnings,
    },
  };
}

export function mergeParsedExports(exports: ParsedExport[], baseStatus: LiveConnectorStatus): { snapshot: LiveFootballSnapshot; diagnostics: ExportDiagnostics } {
  const playerMap = new Map<string, LivePlayer>();
  const clubMap = new Map<string, LiveClub>();
  for (const item of exports) {
    item.players.forEach((player) => playerMap.set(player.id, player));
    item.clubs.forEach((club) => clubMap.set(club.id, club));
  }
  const players = [...playerMap.values()];
  const clubs = [...clubMap.values()];
  const clubCounts = new Map<string, number>();
  players.forEach((player) => player.clubId && clubCounts.set(player.clubId, (clubCounts.get(player.clubId) ?? 0) + 1));
  const managedClubId = [...clubCounts].toSorted((a, b) => b[1] - a[1])[0]?.[0] ?? null;
  const diagnostics: ExportDiagnostics = {
    files: exports.flatMap((item) => item.diagnostics.files),
    columnsMatched: Math.max(0, ...exports.map((item) => item.diagnostics.columnsMatched)),
    sourceColumns: Math.max(0, ...exports.map((item) => item.diagnostics.sourceColumns)),
    missingFields: [...new Set(exports.flatMap((item) => item.diagnostics.missingFields))],
    blockedColumns: [...new Set(exports.flatMap((item) => item.diagnostics.blockedColumns))],
    importedAt: new Date().toISOString(),
    warnings: [...new Set(exports.flatMap((item) => item.diagnostics.warnings))],
  };
  const status: LiveConnectorStatus = {
    ...baseStatus,
    state: "connected",
    parserStatus: baseStatus.parserStatus,
    playersLoaded: players.length,
    clubsLoaded: clubs.length,
    lastSync: diagnostics.importedAt,
    message: `${players.length} players loaded from user-exported visible FM26 data.`,
  };
  return {
    snapshot: {
      status,
      managedClubId,
      managerName: null,
      season: null,
      clubs,
      players,
      tactic: null,
      dataError: null,
      dataSource: "export-watcher",
      dataWarnings: diagnostics.warnings,
    },
    diagnostics,
  };
}

export function parseDelimited(text: string, delimiter = detectDelimiter(text)): string[][] {
  const rows: string[][] = [];
  let row: string[] = [];
  let value = "";
  let quoted = false;
  for (let index = 0; index < text.length; index += 1) {
    const char = text[index];
    if (char === "\"") {
      if (quoted && text[index + 1] === "\"") { value += "\""; index += 1; }
      else quoted = !quoted;
    } else if (char === delimiter && !quoted) {
      row.push(value); value = "";
    } else if ((char === "\n" || char === "\r") && !quoted) {
      if (char === "\r" && text[index + 1] === "\n") index += 1;
      row.push(value); value = "";
      if (row.some((cell) => cell.trim())) rows.push(row);
      row = [];
    } else value += char;
  }
  row.push(value);
  if (row.some((cell) => cell.trim())) rows.push(row);
  return rows;
}

export function detectDelimiter(text: string) {
  const header = text.split(/\r?\n/, 1)[0] ?? "";
  return ([";", ",", "\t"] as const).toSorted((a, b) => countOutsideQuotes(header, b) - countOutsideQuotes(header, a))[0];
}

function countOutsideQuotes(value: string, needle: string) {
  let quoted = false, count = 0;
  for (const char of value) {
    if (char === "\"") quoted = !quoted;
    else if (char === needle && !quoted) count += 1;
  }
  return count;
}

function normalizeHeader(value: string) {
  return value.toLowerCase().normalize("NFKD").replace(/[\u0300-\u036f]/g, "").replace(/[^a-z0-9%]/g, "");
}
function getAlias(row: Record<string, string>, field: keyof typeof aliases) {
  return aliases[field].map((alias) => row[alias]).find((value) => value) ?? "";
}
function parseNumber(value?: string) {
  if (!value) return null;
  const parsed = Number(value.replace(/,/g, "").replace(/[^\d.-]/g, ""));
  return Number.isFinite(parsed) ? parsed : null;
}
function parseAttribute(value?: string) {
  const parsed = parseNumber(value);
  return parsed != null && parsed >= 1 && parsed <= 20 ? parsed : null;
}
function parseMoney(value?: string) {
  if (!value) return null;
  const normalized = value.replace(/\s/g, "").replace(",", ".").toLowerCase();
  const parsed = Number(normalized.replace(/[^\d.-]/g, ""));
  if (!Number.isFinite(parsed)) return null;
  if (/[mb]$/.test(normalized)) return parsed;
  if (/k$/.test(normalized)) return parsed / 1000;
  return parsed >= 100_000 ? parsed / 1_000_000 : parsed;
}
function splitPositions(value: string) {
  return value ? value.split(/[,/;]+/).map((position) => position.trim()).filter(Boolean) : [];
}
function stableId(value: string) {
  let hash = 2166136261;
  for (const char of value.toLowerCase()) { hash ^= char.charCodeAt(0); hash = Math.imul(hash, 16777619); }
  return `fm-${(hash >>> 0).toString(16)}`;
}
function positionMatches(current: string, calculated: string) {
  const value = current.toUpperCase();
  if (calculated === "FB/WB") return /FB|WB|DL|DR|LB|RB/.test(value);
  if (calculated === "W") return /LW|RW|AML|AMR|W/.test(value);
  return value.includes(calculated);
}
function per90MetricName(header: string) {
  return /per90|p90|xg|xa|keypasses|chancescreated|tackles|interceptions|dribbles|progressive/.test(header);
}
function extractPer90(row: Record<string, string>, minutes: number | null, goals: number | null, assists: number | null) {
  const result: Record<string, number | null> = {};
  for (const [header, value] of Object.entries(row)) {
    if (per90MetricName(header)) result[header] = parseNumber(value);
  }
  if (minutes && minutes > 0) {
    result.goalsPer90 = goals == null ? null : Math.round((goals / minutes) * 9000) / 100;
    result.assistsPer90 = assists == null ? null : Math.round((assists / minutes) * 9000) / 100;
  }
  return result;
}
