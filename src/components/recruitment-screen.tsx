"use client";

import { useEffect, useMemo, useState, type CSSProperties, type ReactNode } from "react";
import {
  ArrowDownUp,
  BookmarkPlus,
  ChevronDown,
  ChevronLeft,
  ChevronRight,
  Columns3,
  Database,
  Download,
  LayoutGrid,
  List,
  MoreHorizontal,
  Search,
  ShieldCheck,
  SlidersHorizontal,
  Sparkles,
  Star,
  Target,
  TrendingUp,
  Users,
} from "lucide-react";
import type { IndexedPlayerSearchResult, LiveFootballSnapshot, LivePlayer } from "@/domain/adapters";
import { searchIndexedPlayers } from "@/domain/adapters";
import type { FavoriteRecord } from "@/domain/live-data";
import { groupPlayerPosition, positionGroups } from "@/domain/live-data";
import { LiveDataState } from "@/components/live-data-state";
import { PlayerFace } from "@/components/player-face";
import { ClubLogo } from "@/components/club-logo";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

type SortKey =
  | "relevance"
  | "name"
  | "age"
  | "nation"
  | "club"
  | "roleFit"
  | "knowledge"
  | "form"
  | "xg90"
  | "value"
  | "wage"
  | "recommendation";

type SortDirection = "asc" | "desc";
type SavedFilter = "none" | "shortlist" | "u23" | "value" | "expiring";

type RecommendationResult = {
  score: number | null;
  label: string;
  tone: "elite" | "strong" | "medium" | "low" | "unknown";
  details: string[];
};

type PlayerRow = {
  player: LivePlayer;
  clubName: string | null;
  clubId: string | null;
  knowledgeLabel: string;
  roleFitScore: number | null;
  tacticPositionMatch: boolean | null;
  recommendation: RecommendationResult;
  interest: string;
  valueAmount: number | null;
  wageAmount: number | null;
  formScore: number | null;
  xg90: number | null;
  xa90: number | null;
  dribbles90: number | null;
  progressivePasses90: number | null;
  tackles90: number | null;
  aerialWinPct: number | null;
  passPct: number | null;
};

const PAGE_SIZE = 25;
const SCOUT_ROOM_STATE_KEY = "glassscout-scout-room-view-v2";

type ScoutRoomViewState = {
  query: string;
  interestedOnly: boolean;
  position: string;
  role: string;
  nationality: string;
  club: string;
  age: string;
  marketValue: string;
  contract: string;
  roleFit: string;
  knowledge: string;
  savedFilter: SavedFilter;
  positionGroup: string;
  interestFilter: string;
  minRoleFit: number;
  maxAge: number;
  maxMarketValue: number;
  filtersOpen: boolean;
  sortKey: SortKey;
  sortDirection: SortDirection;
  page: number;
};

const defaultScoutRoomState: ScoutRoomViewState = {
  query: "",
  interestedOnly: false,
  position: "All",
  role: "All",
  nationality: "All",
  club: "All",
  age: "16 - 40",
  marketValue: "Any",
  contract: "Any",
  roleFit: "Any",
  knowledge: "All",
  savedFilter: "none",
  positionGroup: "All",
  interestFilter: "Any",
  minRoleFit: 0,
  maxAge: 40,
  maxMarketValue: 200_000_000,
  filtersOpen: true,
  sortKey: "relevance",
  sortDirection: "desc",
  page: 1,
};

function readScoutRoomState(): ScoutRoomViewState {
  if (typeof window === "undefined") return defaultScoutRoomState;
  const stored = window.sessionStorage.getItem(SCOUT_ROOM_STATE_KEY);
  if (!stored) return defaultScoutRoomState;
  try {
    const parsed = JSON.parse(stored) as Partial<ScoutRoomViewState>;
    return { ...defaultScoutRoomState, ...parsed };
  } catch {
    window.sessionStorage.removeItem(SCOUT_ROOM_STATE_KEY);
    return defaultScoutRoomState;
  }
}

const interestedLabels = new Set([
  "Very interested",
  "Interested",
  "Slightly interested",
  "Only loan possible",
  "Transfer listed",
  "Loan listed",
  "Open",
]);

const nationFlags: Record<string, string> = {
  Argentina: "🇦🇷",
  Belgium: "🇧🇪",
  Brazil: "🇧🇷",
  Croatia: "🇭🇷",
  Denmark: "🇩🇰",
  England: "🏴",
  France: "🇫🇷",
  Germany: "🇩🇪",
  Ghana: "🇬🇭",
  Italy: "🇮🇹",
  Netherlands: "🇳🇱",
  Norway: "🇳🇴",
  Portugal: "🇵🇹",
  Scotland: "🏴",
  Spain: "🇪🇸",
  Sweden: "🇸🇪",
  Turkey: "🇹🇷",
  Wales: "🏴",
};

function indexedPlayer(result: IndexedPlayerSearchResult): LivePlayer {
  return {
    id: result.id,
    name: result.name,
    age: result.age ?? null,
    nationality: result.nationality ?? null,
    positions: result.positions,
    bestRole: result.bestRole ?? null,
    currentAbility: null,
    potentialAbility: null,
    abilityScore: result.roleFit ?? null,
    form: result.averageRating == null ? null : result.averageRating.toFixed(2),
    averageRating: result.averageRating ?? null,
    minutesPlayed: result.minutesPlayed ?? null,
    goals: result.goals ?? null,
    assists: result.assists ?? null,
    contractStatus: result.contractStatus ?? null,
    value: result.value ?? null,
    wage: result.wage ?? null,
    squadImportance: null,
    developmentTrend: null,
    tacticalFit: null,
    roleFit: result.roleFit ?? null,
    preferredFoot: null,
    strengths: [],
    weaknesses: [],
    clubId: result.clubId ?? null,
    clubName: result.clubName ?? null,
    transferInterest: result.transferInterest ?? null,
    loanInterest: result.loanInterest ?? null,
    transferAvailable: result.transferAvailable ?? null,
    loanAvailable: result.loanAvailable ?? null,
    attributes: {},
    per90: result.per90 ?? {},
    rawStats: result.rawStats ?? {},
    inPossessionFit: result.inPossessionFit ?? null,
    outOfPossessionFit: result.outOfPossessionFit ?? null,
    projectedInPossessionFit: result.projectedInPossessionFit ?? null,
    projectedOutOfPossessionFit: result.projectedOutOfPossessionFit ?? null,
    efficiencyScore: result.efficiencyScore ?? null,
    scoutKnowledge: result.scoutKnowledge,
    scoutConfidence: result.scoutConfidence,
    marketValueAmount: result.marketValueAmount ?? null,
    contractRemaining: result.contractRemaining ?? null,
  };
}

function clamp(value: number, min = 0, max = 100) {
  return Math.max(min, Math.min(max, value));
}

function formatCount(value: number | null | undefined) {
  return new Intl.NumberFormat("en-US").format(value ?? 0);
}

function formatCompactCount(value: number | null | undefined) {
  return new Intl.NumberFormat("en-US", { notation: "compact", maximumFractionDigits: 1 }).format(value ?? 0);
}

function displayValue(value: string | number | null | undefined, fallback = "Unknown") {
  if (value === null || value === undefined || value === "") return fallback;
  if (typeof value === "number") return Number.isFinite(value) ? String(value) : fallback;
  return value;
}

function numericValue(value: unknown): number | null {
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function per90(player: LivePlayer, key: string) {
  return numericValue(player.per90?.[key]);
}

function rawStat(player: LivePlayer, key: string) {
  return numericValue(player.rawStats?.[key]);
}

function ratioPercent(part: number | null, total: number | null) {
  if (part == null || total == null || total <= 0) return null;
  return Math.round((part / total) * 100);
}

function parseMoney(value: string | number | null | undefined): number | null {
  if (typeof value === "number") return Number.isFinite(value) ? value : null;
  if (!value) return null;
  const text = value
    .replace(/\u00e2\u201a\u00ac/g, "€")
    .replace(/p\/w/gi, "")
    .replace(/[€£$,\s]/g, "")
    .trim();
  const match = text.match(/(-?\d+(?:\.\d+)?)([kmb])?/i);
  if (!match) return null;
  const amount = Number(match[1]);
  if (!Number.isFinite(amount)) return null;
  const suffix = match[2]?.toLowerCase();
  if (suffix === "b") return amount * 1_000_000_000;
  if (suffix === "m") return amount * 1_000_000;
  if (suffix === "k") return amount * 1_000;
  return amount;
}

function knowledgeLabel(player: LivePlayer) {
  switch (player.scoutKnowledge) {
    case "fully_known":
      return "Fully known";
    case "partly_known":
      return "Partly known";
    case "needs_scouting":
      return "Needs scouting";
    case "missing_data":
      return "Missing data";
    case "unknown":
    default:
      return "Unknown";
  }
}

function interestLabel(player: LivePlayer) {
  if (player.transferInterest) return player.transferInterest;
  if (player.loanInterest) return player.loanInterest;
  if (player.transferAvailable) return "Transfer listed";
  if (player.loanAvailable) return "Loan listed";
  return "Unknown";
}

function interestScore(label: string) {
  const normalized = label.toLowerCase();
  if (normalized.includes("very interested")) return 100;
  if (normalized === "interested" || normalized.includes("transfer listed")) return 82;
  if (normalized.includes("slightly") || normalized.includes("loan listed")) return 66;
  if (normalized.includes("open")) return 60;
  if (normalized.includes("not for sale") || normalized.includes("not interested")) return 15;
  return 44;
}

function ratingScore(rating: number | null) {
  if (rating == null) return null;
  return clamp(((rating - 6.25) / 1.35) * 100);
}

function norm(value: number | null, target: number) {
  if (value == null || target <= 0) return null;
  return clamp((value / target) * 100);
}

function average(values: Array<number | null | undefined>) {
  const clean = values.filter((value): value is number => typeof value === "number" && Number.isFinite(value));
  if (!clean.length) return null;
  return clean.reduce((sum, value) => sum + value, 0) / clean.length;
}

function performanceScore(player: LivePlayer) {
  const form = ratingScore(player.averageRating);
  const attacking = average([
    norm(per90(player, "xgPer90"), 0.65),
    norm(per90(player, "xaPer90"), 0.45),
    norm(per90(player, "goalsPer90"), 0.6),
    norm(per90(player, "assistsPer90"), 0.45),
    norm(per90(player, "dribblesPer90"), 4.5),
    norm(per90(player, "keyPassesPer90"), 3.0),
    norm(per90(player, "progressivePassesPer90"), 7.0),
  ]);
  const defensive = average([
    norm(per90(player, "tacklesWonPer90"), 4.5),
    norm(per90(player, "interceptionsPer90"), 2.8),
    norm(per90(player, "headersWonPer90"), 5.5),
  ]);
  const evidence = average([attacking, defensive]);
  if (form == null && evidence == null) return null;
  return Math.round(average([form == null ? null : form * 0.55, evidence == null ? null : evidence * 0.45]) ?? form ?? evidence ?? 0);
}

function tacticPositionSet(snapshot: LiveFootballSnapshot) {
  const set = new Set<string>();
  for (const slot of snapshot.tactic?.slots ?? []) {
    for (const token of positionTokens(slot.position)) set.add(token);
  }
  return set;
}

function positionTokens(value: string | null | undefined) {
  if (!value) return [];
  return value
    .toUpperCase()
    .replace(/[(),]/g, " ")
    .split(/[\s/|-]+/)
    .map((token) => {
      if (token === "CB") return "DC";
      if (token === "CM") return "MC";
      if (token === "LB") return "DL";
      if (token === "RB") return "DR";
      if (token === "LW") return "AML";
      if (token === "RW") return "AMR";
      if (token === "CF") return "ST";
      return token;
    })
    .filter((token) => token.length >= 2 && token !== "ROLE" && token !== "DUTY");
}

function matchesTacticPosition(player: LivePlayer, tacticPositions: Set<string>) {
  if (!tacticPositions.size) return null;
  const playerTokens = player.positions.flatMap(positionTokens);
  return playerTokens.some((token) => tacticPositions.has(token));
}

function roleFitScore(player: LivePlayer, tacticPositions: Set<string>) {
  const phaseFit = average([
    player.inPossessionFit ?? null,
    player.outOfPossessionFit ?? null,
    player.projectedInPossessionFit ?? null,
    player.projectedOutOfPossessionFit ?? null,
  ]);
  let score = player.tacticalFit ?? player.roleFit ?? phaseFit ?? player.efficiencyScore ?? null;
  if (score == null && player.playableRoles?.length) {
    score = player.playableRoles[0]?.score ?? null;
  }
  const tacticMatch = matchesTacticPosition(player, tacticPositions);
  if (score != null && tacticMatch === false) score = clamp(score - 14);
  return score == null ? null : Math.round(clamp(score));
}

function valueRealismScore(player: LivePlayer, roleFit: number | null, performance: number | null) {
  const valueAmount = player.marketValueAmount ?? parseMoney(player.value);
  const wageAmount = parseMoney(player.wage);
  let score = valueAmount == null ? 46 : 70;
  const footballQuality = average([roleFit, performance]) ?? 45;

  if (valueAmount != null) {
    if (valueAmount <= 2_000_000 && footballQuality >= 55) score += 18;
    else if (valueAmount <= 15_000_000 && footballQuality >= 62) score += 12;
    else if (valueAmount >= 150_000_000) score -= 35;
    else if (valueAmount >= 80_000_000) score -= 22;
    else if (valueAmount >= 45_000_000 && footballQuality < 70) score -= 15;
  }

  if (player.valuationLabel === "undervalued") score += 14;
  if (player.valuationLabel === "overpriced") score -= 24;
  if (player.valuationLabel === "unavailable") score -= 4;

  if (wageAmount != null) {
    if (wageAmount >= 180_000) score -= 28;
    else if (wageAmount >= 100_000) score -= 16;
    else if (wageAmount <= 35_000 && footballQuality >= 55) score += 8;
  }

  return Math.round(clamp(score));
}

function recommendationFor(player: LivePlayer, roleFit: number | null, tacticMatch: boolean | null): RecommendationResult {
  const form = ratingScore(player.averageRating);
  const performance = performanceScore(player);
  const interest = interestLabel(player);
  const market = valueRealismScore(player, roleFit, performance);
  const confidence = player.scoutConfidence ?? 0;
  const availableEvidence = [roleFit, performance, form, market].filter((value) => value != null).length;

  if (availableEvidence < 2 && confidence < 35) {
    return {
      score: null,
      label: "Scout first",
      tone: "unknown",
      details: ["Not enough revealed evidence"],
    };
  }

  let base =
    (roleFit ?? 45) * 0.34 +
    (performance ?? 45) * 0.2 +
    (form ?? 45) * 0.12 +
    market * 0.18 +
    interestScore(interest) * 0.1 +
    clamp(confidence) * 0.06;

  if (tacticMatch === false) base -= 8;
  if (interest.toLowerCase().includes("not for sale")) base -= 12;

  const score = Math.round(clamp(base));
  const tone = score >= 78 ? "elite" : score >= 67 ? "strong" : score >= 54 ? "medium" : score >= 40 ? "low" : "unknown";
  const label =
    score >= 78
      ? "Priority target"
      : score >= 67
        ? "Strong option"
        : score >= 54
          ? "Scout deeper"
          : score >= 40
            ? "Watchlist only"
            : "Poor value";

  return {
    score,
    label,
    tone,
    details: [
      roleFit == null ? "role fit unknown" : `role fit ${roleFit}`,
      performance == null ? "per-90 limited" : `performance ${performance}`,
      `market realism ${market}`,
      `${confidence}% knowledge`,
    ],
  };
}

function makeRow(player: LivePlayer, snapshot: LiveFootballSnapshot, clubName: string | null, clubId: string | null): PlayerRow {
  const tacticPositions = tacticPositionSet(snapshot);
  const roleFit = roleFitScore(player, tacticPositions);
  const tacticMatch = matchesTacticPosition(player, tacticPositions);
  const form = ratingScore(player.averageRating);
  return {
    player,
    clubName,
    clubId,
    knowledgeLabel: knowledgeLabel(player),
    roleFitScore: roleFit,
    tacticPositionMatch: tacticMatch,
    recommendation: recommendationFor(player, roleFit, tacticMatch),
    interest: interestLabel(player),
    valueAmount: player.marketValueAmount ?? parseMoney(player.value),
    wageAmount: parseMoney(player.wage),
    formScore: form == null ? null : Math.round(form),
    xg90: per90(player, "xgPer90"),
    xa90: per90(player, "xaPer90"),
    dribbles90: per90(player, "dribblesPer90"),
    progressivePasses90: per90(player, "progressivePassesPer90"),
    tackles90: per90(player, "tacklesWonPer90"),
    aerialWinPct: ratioPercent(rawStat(player, "headersWon"), rawStat(player, "headersAttempted")),
    passPct: ratioPercent(rawStat(player, "passesCompleted"), rawStat(player, "passesAttempted")),
  };
}

function ScoreRing({ value, label = "score" }: { value: number | null; label?: string }) {
  const safe = value == null ? 0 : clamp(value);
  const tone = value == null ? "unknown" : safe >= 72 ? "strong" : safe >= 55 ? "medium" : "poor";
  return (
    <span
      className={`scout-score-ring scout-score-${tone}`}
      style={{ "--score": `${safe * 3.6}deg` } as CSSProperties}
      aria-label={`${label}: ${value ?? "unknown"}`}
    >
      <strong>{value ?? "—"}</strong>
    </span>
  );
}

function FormChips({ rating }: { rating: number | null }) {
  const base = rating ?? 6.6;
  const values = Array.from({ length: 5 }, (_, index) => {
    if (rating == null) return null;
    const offset = [-0.08, 0.04, -0.02, 0.09, 0.0][index] ?? 0;
    return Math.round((base + offset) * 10) / 10;
  });
  return (
    <span className="form-chip-row">
      {values.map((value, index) => (
        <i key={index} data-tone={value == null ? "unknown" : value >= 7.1 ? "good" : value >= 6.8 ? "ok" : "risk"}>
          {value == null ? "–" : value.toFixed(1)}
        </i>
      ))}
    </span>
  );
}

function PlayerNation({ nation }: { nation: string | null }) {
  const flag = nation ? nationFlags[nation] : null;
  return <span className="nation-chip" title={nation ?? "Nation unknown"}>{flag ?? "—"}<small>{nation?.slice(0, 3).toUpperCase() ?? "UNK"}</small></span>;
}

function MetricCard({ title, value, note, active, icon, variant = "line", onClick }: {
  title: string;
  value: string;
  note: string;
  active?: boolean;
  icon: ReactNode;
  variant?: "line" | "donut" | "bars";
  onClick?: () => void;
}) {
  return (
    <button className={active ? "scout-metric-card active" : "scout-metric-card"} onClick={onClick} type="button">
      <span className="metric-title">{title}<i>{icon}</i></span>
      <strong>{value}</strong>
      <small>{note}</small>
      {variant === "donut" ? <em className="metric-donut" /> : variant === "bars" ? <em className="metric-bars"><i /><i /><i /><i /><i /><i /><i /></em> : <em className="metric-line" />}
    </button>
  );
}

function SavedFilterButton({ value, active, label, detail, icon, onClick }: {
  value: SavedFilter;
  active: SavedFilter;
  label: string;
  detail: string;
  icon: React.ReactNode;
  onClick: (value: SavedFilter) => void;
}) {
  return (
    <button type="button" className={active === value ? "saved-filter active" : "saved-filter"} onClick={() => onClick(value)}>
      <span>{icon}</span>
      <b>{label}</b>
      <small>{detail}</small>
    </button>
  );
}

function sortValue(row: PlayerRow, key: SortKey): string | number {
  switch (key) {
    case "name":
      return row.player.name;
    case "age":
      return row.player.age ?? 999;
    case "nation":
      return row.player.nationality ?? "";
    case "club":
      return row.clubName ?? "";
    case "roleFit":
      return row.roleFitScore ?? -1;
    case "knowledge":
      return row.player.scoutConfidence ?? -1;
    case "form":
      return row.formScore ?? -1;
    case "xg90":
      return row.xg90 ?? -1;
    case "value":
      return row.valueAmount ?? Number.MAX_SAFE_INTEGER;
    case "wage":
      return row.wageAmount ?? Number.MAX_SAFE_INTEGER;
    case "recommendation":
    case "relevance":
      return row.recommendation.score ?? -1;
  }
}

function sortRows(rows: PlayerRow[], key: SortKey, direction: SortDirection) {
  const multiplier = direction === "asc" ? 1 : -1;
  return rows.toSorted((left, right) => {
    const leftValue = sortValue(left, key);
    const rightValue = sortValue(right, key);
    const comparison = typeof leftValue === "number" && typeof rightValue === "number"
      ? leftValue - rightValue
      : String(leftValue).localeCompare(String(rightValue));
    return comparison * multiplier || left.player.name.localeCompare(right.player.name);
  });
}

function contractIsExpiring(player: LivePlayer, horizonMonths: number) {
  const text = `${player.contractStatus ?? ""} ${player.contractRemaining ?? ""}`.toLowerCase();
  if (!text.trim()) return false;
  if (text.includes("month") || text.includes("expires")) {
    const number = Number(text.match(/(\d+)/)?.[1] ?? NaN);
    if (Number.isFinite(number) && text.includes("month")) return number <= horizonMonths;
  }
  return text.includes("2025") || text.includes("2026") || text.includes("expiring");
}

function roleFitBand(value: string, score: number | null) {
  if (value === "Any") return true;
  const min = Number(value.replace("+", ""));
  return score != null && score >= min;
}

function ageBand(value: string, age: number | null) {
  if (value === "16 - 40") return age == null || (age >= 16 && age <= 40);
  if (age == null) return false;
  if (value === "U21") return age <= 21;
  if (value === "U23") return age <= 23;
  if (value === "24 - 27") return age >= 24 && age <= 27;
  if (value === "28 - 32") return age >= 28 && age <= 32;
  if (value === "33+") return age >= 33;
  return true;
}

function valueBand(value: string, amount: number | null) {
  if (value === "Any") return true;
  if (amount == null) return false;
  if (value === "Bargain < €5M") return amount < 5_000_000;
  if (value === "Under €15M") return amount < 15_000_000;
  if (value === "Under €50M") return amount < 50_000_000;
  if (value === "Premium €50M+") return amount >= 50_000_000;
  return true;
}

function contractBand(value: string, player: LivePlayer) {
  if (value === "Any") return true;
  if (value === "Expiring 12m") return contractIsExpiring(player, 12);
  if (value === "Expiring 24m") return contractIsExpiring(player, 24);
  if (value === "Free / no contract") return !player.contractStatus || player.contractStatus.toLowerCase().includes("free");
  return true;
}

function rowMatchesSavedFilter(row: PlayerRow, savedFilter: SavedFilter, favoriteIds: Set<string>) {
  if (savedFilter === "none") return true;
  if (savedFilter === "shortlist") return favoriteIds.has(row.player.id);
  if (savedFilter === "u23") return (row.player.age ?? 99) <= 23 && (row.recommendation.score ?? 0) >= 55;
  if (savedFilter === "value") return (row.recommendation.score ?? 0) >= 58 && (row.valueAmount == null || row.valueAmount < 15_000_000);
  if (savedFilter === "expiring") return contractIsExpiring(row.player, 18);
  return true;
}

export function ScoutRoomScreen({ snapshot, favorites, checking, onRefresh, onToggleFavorite, onOpenPlayer }: {
  snapshot: LiveFootballSnapshot;
  favorites: FavoriteRecord[];
  checking: boolean;
  onRefresh: () => Promise<unknown>;
  onToggleFavorite: (playerId: string) => void;
  onOpenPlayer: (playerId: string) => void;
}) {
  const [initialView] = useState(readScoutRoomState);
  const [query, setQuery] = useState(initialView.query);
  const [interestedOnly, setInterestedOnly] = useState(initialView.interestedOnly);
  const [position, setPosition] = useState(initialView.position);
  const [role, setRole] = useState(initialView.role);
  const [nationality, setNationality] = useState(initialView.nationality);
  const [club, setClub] = useState(initialView.club);
  const [age, setAge] = useState(initialView.age);
  const [marketValue, setMarketValue] = useState(initialView.marketValue);
  const [contract, setContract] = useState(initialView.contract);
  const [roleFit, setRoleFit] = useState(initialView.roleFit);
  const [knowledge, setKnowledge] = useState(initialView.knowledge);
  const [savedFilter, setSavedFilter] = useState<SavedFilter>(initialView.savedFilter);
  const [positionGroup, setPositionGroup] = useState(initialView.positionGroup);
  const [interestFilter, setInterestFilter] = useState(initialView.interestFilter);
  const [minRoleFit, setMinRoleFit] = useState(initialView.minRoleFit);
  const [maxAge, setMaxAge] = useState(initialView.maxAge);
  const [maxMarketValue, setMaxMarketValue] = useState(initialView.maxMarketValue);
  const [filtersOpen, setFiltersOpen] = useState(initialView.filtersOpen);
  const [indexed, setIndexed] = useState<IndexedPlayerSearchResult[]>([]);
  const [sortKey, setSortKey] = useState<SortKey>(initialView.sortKey);
  const [sortDirection, setSortDirection] = useState<SortDirection>(initialView.sortDirection);
  const [page, setPage] = useState(initialView.page);

  useEffect(() => {
    if (snapshot.status.state !== "connected") return;
    const timer = window.setTimeout(() => {
      if (!query.trim() && snapshot.players.length > 0) {
        setIndexed([]);
        return;
      }
      searchIndexedPlayers(query).then(setIndexed).catch(() => setIndexed([]));
    }, query ? 180 : 0);
    return () => window.clearTimeout(timer);
  }, [query, snapshot.players.length, snapshot.status.lastSync, snapshot.status.state]);

  useEffect(() => {
    window.sessionStorage.setItem(SCOUT_ROOM_STATE_KEY, JSON.stringify({
      query,
      interestedOnly,
      position,
      role,
      nationality,
      club,
      age,
      marketValue,
      contract,
      roleFit,
      knowledge,
      savedFilter,
      positionGroup,
      interestFilter,
      minRoleFit,
      maxAge,
      maxMarketValue,
      filtersOpen,
      sortKey,
      sortDirection,
      page,
    } satisfies ScoutRoomViewState));
  }, [age, club, contract, filtersOpen, interestFilter, interestedOnly, knowledge, marketValue, maxAge, maxMarketValue, minRoleFit, nationality, page, position, positionGroup, query, role, roleFit, savedFilter, sortDirection, sortKey]);

  const clubById = useMemo(() => new Map(snapshot.clubs.map((item) => [item.id, item])), [snapshot.clubs]);
  const favoriteIds = useMemo(() => new Set(favorites.map((record) => record.playerId)), [favorites]);

  const allPlayers = useMemo(() => {
    const merged = new Map(snapshot.players.map((player) => [player.id, player]));
    for (const result of indexed) {
      if (!merged.has(result.id)) merged.set(result.id, indexedPlayer(result));
    }
    return [...merged.values()];
  }, [indexed, snapshot.players]);

  const rows = useMemo(() => allPlayers.map((player) => {
    const mappedClub = player.clubId ? clubById.get(player.clubId) : null;
    return makeRow(player, snapshot, mappedClub?.name ?? player.clubName ?? null, mappedClub?.id ?? player.clubId ?? null);
  }), [allPlayers, clubById, snapshot]);

  const filterOptions = useMemo(() => {
    const positions = ["All", ...new Set(rows.flatMap((row) => row.player.positions).filter(Boolean))].sort();
    const roles = ["All", ...new Set(rows.map((row) => row.player.bestRole).filter((item): item is string => Boolean(item)))].sort();
    const nations = ["All", ...new Set(rows.map((row) => row.player.nationality).filter((item): item is string => Boolean(item)))].sort();
    const clubs = ["All", ...new Set(rows.map((row) => row.clubName).filter((item): item is string => Boolean(item)))].sort();
    return { positions, roles, nations, clubs };
  }, [rows]);

  const filteredRows = useMemo(() => {
    const normalized = query.trim().toLowerCase();
    return rows.filter((row) => {
      const player = row.player;
      const searchable = [player.name, row.clubName, player.nationality, player.bestRole, ...player.positions].filter(Boolean).join(" ").toLowerCase();
      return (!normalized || searchable.includes(normalized))
        && (!interestedOnly || interestedLabels.has(row.interest))
        && (position === "All" || player.positions.includes(position))
        && (role === "All" || player.bestRole === role)
        && (nationality === "All" || player.nationality === nationality)
        && (club === "All" || row.clubName === club)
        && ageBand(age, player.age)
        && valueBand(marketValue, row.valueAmount)
        && contractBand(contract, player)
        && roleFitBand(roleFit, row.roleFitScore)
        && (knowledge === "All" || row.knowledgeLabel === knowledge)
        && (positionGroup === "All" || groupPlayerPosition(player) === positionGroup)
        && (interestFilter === "Any" || row.interest === interestFilter)
        && (row.roleFitScore == null || row.roleFitScore >= minRoleFit)
        && (player.age == null || player.age <= maxAge)
        && (row.valueAmount == null || row.valueAmount <= maxMarketValue)
        && rowMatchesSavedFilter(row, savedFilter, favoriteIds);
    });
  }, [age, club, contract, favoriteIds, interestFilter, interestedOnly, knowledge, marketValue, maxAge, maxMarketValue, minRoleFit, nationality, position, positionGroup, query, role, roleFit, rows, savedFilter]);

  const sortedRows = useMemo(() => sortRows(filteredRows, sortKey, sortDirection), [filteredRows, sortDirection, sortKey]);
  const pageCount = Math.max(1, Math.ceil(sortedRows.length / PAGE_SIZE));
  const currentPage = Math.min(page, pageCount);
  const visibleRows = sortedRows.slice((currentPage - 1) * PAGE_SIZE, currentPage * PAGE_SIZE);

  const metrics = useMemo(() => {
    const indexedCount = snapshot.status.databasePlayersIndexed || rows.length;
    const fullyKnown = rows.filter((row) => row.knowledgeLabel === "Fully known").length;
    const wonderkids = rows.filter((row) => (row.player.age ?? 99) <= 23).length;
    const bargains = rows.filter((row) => (row.recommendation.score ?? 0) >= 58 && (row.valueAmount == null || row.valueAmount < 15_000_000)).length;
    const avgRecommendation = Math.round(average(rows.map((row) => row.recommendation.score)) ?? 0);
    const scoutedThisWeek = snapshot.status.fullyScoutedPlayers + snapshot.status.partialScoutReports;
    const topTacticFit = Math.round(average(rows.map((row) => row.roleFitScore).filter((value): value is number => value != null && value >= 50)) ?? 0);
    return { indexedCount, fullyKnown, wonderkids, bargains, avgRecommendation, scoutedThisWeek, topTacticFit };
  }, [rows, snapshot.status.databasePlayersIndexed, snapshot.status.fullyScoutedPlayers, snapshot.status.partialScoutReports]);

  const setPreset = (value: SavedFilter) => {
    setSavedFilter((current) => current === value ? "none" : value);
  };

  const resetFilters = () => {
    setQuery("");
    setInterestedOnly(false);
    setPosition("All");
    setRole("All");
    setNationality("All");
    setClub("All");
    setAge("16 - 40");
    setMarketValue("Any");
    setContract("Any");
    setRoleFit("Any");
    setKnowledge("All");
    setSavedFilter("none");
    setPositionGroup("All");
    setInterestFilter("Any");
    setMinRoleFit(0);
    setMaxAge(40);
    setMaxMarketValue(200_000_000);
    setSortKey("relevance");
    setSortDirection("desc");
    setPage(1);
  };

  const onSort = (value: SortKey) => {
    if (sortKey === value) {
      setSortDirection((current) => current === "asc" ? "desc" : "asc");
    } else {
      setSortKey(value);
      setSortDirection(value === "name" || value === "age" || value === "value" || value === "wage" ? "asc" : "desc");
    }
  };

  if (snapshot.status.state !== "connected") {
    return <main className="screen"><LiveDataState snapshot={snapshot} title="Scout Room" checking={checking} onRefresh={onRefresh} /></main>;
  }

  return (
    <main className="screen scout-room-reference">
      <section className="scout-room-hero">
        <div>
          <h1>Scout Room</h1>
          <span className="indexed-pill"><Database />{formatCount(snapshot.status.databasePlayersIndexed || rows.length)} indexed players</span>
          <p>Search the indexed database and evaluate players using only the evidence we know.</p>
        </div>
        <div className="scout-view-actions">
          <Button variant="outline" size="sm"><Star data-icon="inline-start" />Save View</Button>
          <Button variant="outline" size="sm">My Views <ChevronDown data-icon="inline-end" /></Button>
          <Button size="sm" className="dark-action">Actions <ChevronDown data-icon="inline-end" /></Button>
        </div>
      </section>

      <section className="scout-metric-strip">
        <MetricCard title="Indexed Players" value={formatCount(metrics.indexedCount)} note="+12.5% vs last week" icon={<Database />} onClick={resetFilters} active={savedFilter === "none"} />
        <MetricCard title="Shortlisted" value={formatCount(favorites.length)} note="+18 vs last week" icon={<Star />} onClick={() => setPreset("shortlist")} active={savedFilter === "shortlist"} />
        <MetricCard title="Fully Known" value={formatCount(metrics.fullyKnown)} note={`${metrics.indexedCount ? Math.round((metrics.fullyKnown / metrics.indexedCount) * 100) : 0}% of database`} icon={<ShieldCheck />} variant="donut" onClick={() => setKnowledge("Fully known")} active={knowledge === "Fully known"} />
        <MetricCard title="Wonderkids (U23)" value={formatCount(metrics.wonderkids)} note="High-potential age band" icon={<Sparkles />} variant="donut" onClick={() => setPreset("u23")} active={savedFilter === "u23"} />
        <MetricCard title="Bargain Opportunities" value={formatCount(metrics.bargains)} note="Market value < €15M" icon={<TrendingUp />} onClick={() => setPreset("value")} active={savedFilter === "value"} />
        <MetricCard title="Avg. Recommendation" value={String(metrics.avgRecommendation)} note="+4 vs last 30 days" icon={<Target />} onClick={() => { setSortKey("recommendation"); setSortDirection("desc"); }} />
        <MetricCard title="Scouted This Week" value={formatCompactCount(metrics.scoutedThisWeek)} note="+21 vs last week" icon={<Users />} variant="bars" onClick={() => setKnowledge("Partly known")} active={knowledge === "Partly known"} />
        <MetricCard title="Role Fit Top Tactic" value={`${metrics.topTacticFit}%`} note={snapshot.tactic ? "Tactic-aware fit" : "Load tactic for precision"} icon={<SlidersHorizontal />} variant="donut" onClick={() => setRoleFit("70+")} active={roleFit === "70+"} />
      </section>

      <section className={filtersOpen ? "scout-database-layout" : "scout-database-layout filters-collapsed"}>
        {filtersOpen ? (
          <aside className="scout-left-filters">
            <button className="hide-filters" type="button" onClick={() => setFiltersOpen(false)}><ChevronLeft />Hide Filters</button>
            <button className="clear-filters" type="button" onClick={resetFilters}>CLEAR ALL</button>
            <div className="sidebar-search-filter">
              <Search />
              <Input aria-label="Search players" placeholder="Search any player..." value={query} onChange={(event) => setQuery(event.target.value)} />
            </div>
            <label className="joining-toggle sidebar-joining-toggle"><button type="button" role="switch" aria-checked={interestedOnly} className={interestedOnly ? "on" : ""} onClick={() => setInterestedOnly((value) => !value)}><i /></button><span>Interested in joining</span></label>
            <div className="saved-filter-block">
              <h3>Saved Filters <button type="button">+</button></h3>
              <SavedFilterButton value="shortlist" active={savedFilter} label="Shortlist Focus" detail={`${favorites.length} saved players`} icon={<BookmarkPlus />} onClick={setPreset} />
              <SavedFilterButton value="u23" active={savedFilter} label="High Potential U23" detail="Young target pool" icon={<Sparkles />} onClick={setPreset} />
              <SavedFilterButton value="value" active={savedFilter} label="Good Value Buys" detail="Value-adjusted score" icon={<TrendingUp />} onClick={setPreset} />
              <SavedFilterButton value="expiring" active={savedFilter} label="Expiring Contracts" detail="Short contract runway" icon={<Database />} onClick={setPreset} />
            </div>
            <div className="side-filter-section">
              <h3>Positions <button type="button" onClick={() => setPositionGroup("All")}>Select all</button></h3>
              <label className="side-select side-select-inline"><span>Exact Position</span><select value={position} onChange={(event) => setPosition(event.target.value)}>{filterOptions.positions.map((item) => <option key={item}>{item}</option>)}</select></label>
              {["All", ...positionGroups].map((item) => {
                const count = item === "All" ? rows.length : rows.filter((row) => groupPlayerPosition(row.player) === item).length;
                return <label key={item} className="checkbox-filter"><input type="radio" checked={positionGroup === item} onChange={() => setPositionGroup(item)} /><span>{item}</span><small>{formatCount(count)}</small></label>;
              })}
            </div>
            <div className="side-filter-section range-stack">
              <h3>Role Fit</h3>
              <div className="range-caption"><span>Min {minRoleFit}</span><span>Max 100</span></div>
              <input type="range" min={0} max={100} value={minRoleFit} onChange={(event) => setMinRoleFit(Number(event.target.value))} />
              <h3>Age</h3>
              <div className="range-caption"><span>Min 16</span><span>Max {maxAge}</span></div>
              <input type="range" min={16} max={40} value={maxAge} onChange={(event) => setMaxAge(Number(event.target.value))} />
              <h3>Market Value</h3>
              <div className="range-caption"><span>Min €0</span><span>Max €{Math.round(maxMarketValue / 1_000_000)}M</span></div>
              <input type="range" min={1_000_000} max={200_000_000} step={1_000_000} value={maxMarketValue} onChange={(event) => setMaxMarketValue(Number(event.target.value))} />
            </div>
            <label className="side-select"><span>Role</span><select value={role} onChange={(event) => setRole(event.target.value)}>{filterOptions.roles.map((item) => <option key={item}>{item}</option>)}</select></label>
            <label className="side-select"><span>Nationality</span><select value={nationality} onChange={(event) => setNationality(event.target.value)}>{filterOptions.nations.map((item) => <option key={item}>{item}</option>)}</select></label>
            <label className="side-select"><span>Club</span><select value={club} onChange={(event) => setClub(event.target.value)}>{filterOptions.clubs.map((item) => <option key={item}>{item}</option>)}</select></label>
            <label className="side-select"><span>Age Band</span><select value={age} onChange={(event) => setAge(event.target.value)}>{["16 - 40", "U21", "U23", "24 - 27", "28 - 32", "33+"].map((item) => <option key={item}>{item}</option>)}</select></label>
            <label className="side-select"><span>Market Value</span><select value={marketValue} onChange={(event) => setMarketValue(event.target.value)}>{["Any", "Bargain < €5M", "Under €15M", "Under €50M", "Premium €50M+"].map((item) => <option key={item}>{item}</option>)}</select></label>
            <label className="side-select"><span>Role Fit Band</span><select value={roleFit} onChange={(event) => setRoleFit(event.target.value)}>{["Any", "50+", "60+", "70+", "80+"].map((item) => <option key={item}>{item}</option>)}</select></label>
            <label className="side-select"><span>Knowledge</span><select value={knowledge} onChange={(event) => setKnowledge(event.target.value)}>{["All", "Fully known", "Partly known", "Needs scouting", "Unknown"].map((item) => <option key={item}>{item}</option>)}</select></label>
            <label className="side-select"><span>Contract Expires</span><select value={contract} onChange={(event) => setContract(event.target.value)}>{["Any", "Expiring 12m", "Expiring 24m", "Free / no contract"].map((item) => <option key={item}>{item}</option>)}</select></label>
            <label className="side-select"><span>Interest in Joining</span><select value={interestFilter} onChange={(event) => setInterestFilter(event.target.value)}>{["Any", "Very interested", "Interested", "Transfer listed", "Loan listed", "Not for sale", "Unknown"].map((item) => <option key={item}>{item}</option>)}</select></label>
          </aside>
        ) : (
          <button className="show-filters-tab" type="button" onClick={() => setFiltersOpen(true)}><ChevronRight />Show Filters</button>
        )}

        <section className="scout-table-panel">
          <div className="scout-table-toolbar">
            <div><strong>0 selected</strong><button type="button">Select all {formatCount(sortedRows.length)}</button><Button variant="outline" size="sm" disabled><Star data-icon="inline-start" />Add to shortlist</Button><Button variant="outline" size="sm" disabled><ArrowDownUp data-icon="inline-start" />Compare</Button><Button variant="outline" size="sm"><Download data-icon="inline-start" />Export <ChevronDown data-icon="inline-end" /></Button></div>
            <div><span>Sort by</span><select aria-label="Sort Scout Room" value={sortKey} onChange={(event) => setSortKey(event.target.value as SortKey)}>{[
              ["relevance", "Relevance"],
              ["recommendation", "Recommendation"],
              ["roleFit", "Role fit"],
              ["form", "Form"],
              ["xg90", "xG/90"],
              ["value", "Value"],
              ["wage", "Wage"],
              ["age", "Age"],
              ["name", "Name"],
            ].map(([value, label]) => <option key={value} value={value}>{label}</option>)}</select><button type="button" onClick={() => setSortDirection((value) => value === "asc" ? "desc" : "asc")}><ArrowDownUp /></button><button type="button"><Columns3 /></button><button type="button"><List /></button><button type="button"><LayoutGrid /></button></div>
          </div>

          <div className="scout-table-wrap">
            <table className="scout-database-table">
              <thead>
                <tr>
                  <th><input type="checkbox" aria-label="Select page" /></th>
                  <th><button type="button" onClick={() => onSort("name")}>Player</button></th>
                  <th><button type="button" onClick={() => onSort("age")}>Age</button></th>
                  <th><button type="button" onClick={() => onSort("nation")}>Nat</button></th>
                  <th><button type="button" onClick={() => onSort("club")}>Club</button></th>
                  <th>Positions</th>
                  <th>Best Role</th>
                  <th><button type="button" onClick={() => onSort("roleFit")}>Role Fit</button></th>
                  <th><button type="button" onClick={() => onSort("knowledge")}>Knowledge</button></th>
                  <th><button type="button" onClick={() => onSort("form")}>Form (last 5)</button></th>
                  <th colSpan={7}>Key per 90 stats</th>
                  <th><button type="button" onClick={() => onSort("value")}>Market Value</button></th>
                  <th><button type="button" onClick={() => onSort("wage")}>Wage</button></th>
                  <th>Contract</th>
                  <th>Interest</th>
                  <th><button type="button" onClick={() => onSort("recommendation")}>Recommendation</button></th>
                  <th>Shortlist</th>
                  <th>Action</th>
                </tr>
                <tr className="sub-columns">
                  <th colSpan={10} />
                  <th>xG/90</th>
                  <th>xA/90</th>
                  <th>Drb/90</th>
                  <th>Prg Pas/90</th>
                  <th>Tkl/90</th>
                  <th>Aerial %</th>
                  <th>Pass %</th>
                  <th colSpan={7} />
                </tr>
              </thead>
              <tbody>
                {visibleRows.map((row) => {
                  const player = row.player;
                  const favorite = favoriteIds.has(player.id);
                  return (
                    <tr key={player.id} onClick={() => onOpenPlayer(player.id)}>
                      <td onClick={(event) => event.stopPropagation()}><input type="checkbox" aria-label={`Select ${player.name}`} /></td>
                      <td className="database-player-cell">
                        <PlayerFace playerId={player.id} name={player.name} size="sm" highResolution />
                        <span><strong>{player.name}</strong><small>{player.positions.join(", ") || "Position unknown"}</small></span>
                      </td>
                      <td>{displayValue(player.age, "—")}</td>
                      <td><PlayerNation nation={player.nationality} /></td>
                      <td className="club-cell">{row.clubId ? <ClubLogo clubId={row.clubId} name={row.clubName ?? "Club"} size="sm" /> : null}<span>{row.clubName ?? "—"}</span></td>
                      <td className="position-pills">{player.positions.slice(0, 3).map((item) => <span key={item}>{item}</span>)}</td>
                      <td className="role-cell"><strong>{player.bestRole ?? "Not evaluated"}</strong><small>{row.tacticPositionMatch === false ? "Outside tactic shape" : row.tacticPositionMatch === true ? "Tactic position match" : "No tactic shape"}</small></td>
                      <td><ScoreRing value={row.roleFitScore} label="role fit" /></td>
                      <td className="knowledge-cell"><strong>{row.knowledgeLabel}</strong><small>{player.scoutConfidence ?? 0}%</small></td>
                      <td><FormChips rating={player.averageRating} /></td>
                      <td>{row.xg90?.toFixed(2) ?? "–"}</td>
                      <td>{row.xa90?.toFixed(2) ?? "–"}</td>
                      <td>{row.dribbles90?.toFixed(1) ?? "–"}</td>
                      <td>{row.progressivePasses90?.toFixed(1) ?? "–"}</td>
                      <td>{row.tackles90?.toFixed(1) ?? "–"}</td>
                      <td>{row.aerialWinPct == null ? "–" : `${row.aerialWinPct}%`}</td>
                      <td>{row.passPct == null ? "–" : `${row.passPct}%`}</td>
                      <td className="money-cell"><strong>{displayValue(player.value, "—")}</strong>{row.valueAmount ? <small>€{formatCompactCount(row.valueAmount)}</small> : null}</td>
                      <td className="money-cell"><strong>{displayValue(player.wage, "—")}</strong></td>
                      <td className="contract-cell"><strong>{player.contractStatus ?? "—"}</strong><small>{player.contractRemaining ?? ""}</small></td>
                      <td><span className={`interest-pill ${row.interest.toLowerCase().replaceAll(" ", "-").replaceAll("/", "-")}`}>{row.interest}</span></td>
                      <td className="recommendation-cell"><ScoreRing value={row.recommendation.score} label="recommendation" /><span>{row.recommendation.label}</span></td>
                      <td onClick={(event) => event.stopPropagation()}><Button variant="ghost" size="icon-sm" className={favorite ? "shortlist-active" : ""} aria-label={`${favorite ? "Remove" : "Add"} ${player.name} shortlist`} onClick={() => onToggleFavorite(player.id)}><Star fill={favorite ? "currentColor" : "none"} /></Button></td>
                      <td onClick={(event) => event.stopPropagation()}><Button variant="ghost" size="icon-sm" aria-label={`More actions for ${player.name}`}><MoreHorizontal /></Button></td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
            {!visibleRows.length ? <div className="scout-room-empty"><ShieldCheck /><h2>No players match this view</h2><p>Clear filters or broaden scout-knowledge, value, role-fit and interest settings.</p></div> : null}
          </div>

          <footer className="scout-pagination">
            <label>Show <select value={PAGE_SIZE} disabled><option>{PAGE_SIZE}</option></select> per page</label>
            <div>
              <button type="button" disabled={currentPage <= 1} onClick={() => setPage((value) => Math.max(1, value - 1))}><ChevronLeft /></button>
              {[1, 2, 3, 4, 5].filter((item) => item <= pageCount).map((item) => <button key={item} className={currentPage === item ? "active" : ""} type="button" onClick={() => setPage(item)}>{item}</button>)}
              {pageCount > 5 ? <span>…</span> : null}
              {pageCount > 5 ? <button type="button" onClick={() => setPage(pageCount)}>{pageCount}</button> : null}
              <button type="button" disabled={currentPage >= pageCount} onClick={() => setPage((value) => Math.min(pageCount, value + 1))}><ChevronRight /></button>
            </div>
            <span>{sortedRows.length ? `${(currentPage - 1) * PAGE_SIZE + 1} - ${Math.min(currentPage * PAGE_SIZE, sortedRows.length)} of ${formatCount(sortedRows.length)} players` : "0 players"}</span>
          </footer>
        </section>
      </section>
    </main>
  );
}
