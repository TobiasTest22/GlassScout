import type { LivePlayer } from "@/domain/adapters";

export const positionGroups = [
  "Goalkeepers",
  "Centre-backs",
  "Full-backs / wing-backs",
  "Defensive midfielders",
  "Central midfielders",
  "Attacking midfielders",
  "Wingers",
  "Strikers",
  "Utility / other players",
] as const;

export type PositionGroup = (typeof positionGroups)[number];

export function groupPlayerPosition(player: LivePlayer): PositionGroup {
  const positions = player.positions.map((position) => position.toUpperCase());
  if (positions.some((position) => /\bGK\b/.test(position))) return "Goalkeepers";
  if (positions.some((position) => /\bDC\b|\bCB\b/.test(position))) return "Centre-backs";
  if (positions.some((position) => /\bDL\b|\bDR\b|\bLB\b|\bRB\b|\bWB/.test(position))) return "Full-backs / wing-backs";
  if (positions.some((position) => /\bDM\b|\bDMC\b/.test(position))) return "Defensive midfielders";
  if (positions.some((position) => /\bMC\b|\bCM\b/.test(position))) return "Central midfielders";
  if (positions.some((position) => /\bAMC\b|\bAM\b/.test(position))) return "Attacking midfielders";
  if (positions.some((position) => /\bAML\b|\bAMR\b|\bLW\b|\bRW\b/.test(position))) return "Wingers";
  if (positions.some((position) => /\bST\b|\bCF\b/.test(position))) return "Strikers";
  return "Utility / other players";
}

export function groupSquad(players: LivePlayer[]) {
  const grouped = new Map<PositionGroup, LivePlayer[]>(
    positionGroups.map((group) => [group, []]),
  );
  for (const player of players) {
    grouped.get(groupPlayerPosition(player))?.push(player);
  }
  return grouped;
}

export type FavoriteRecord = {
  playerId: string;
  note: string;
};

export function toggleFavorite(records: FavoriteRecord[], playerId: string): FavoriteRecord[] {
  return records.some((record) => record.playerId === playerId)
    ? records.filter((record) => record.playerId !== playerId)
    : [...records, { playerId, note: "" }];
}

export function updateFavoriteNote(records: FavoriteRecord[], playerId: string, note: string): FavoriteRecord[] {
  return records.map((record) => record.playerId === playerId ? { ...record, note } : record);
}

export function resolveFavorites(records: FavoriteRecord[], players: LivePlayer[]) {
  const playersById = new Map(players.map((player) => [player.id, player]));
  return records.flatMap((record) => {
    const player = playersById.get(record.playerId);
    return player ? [{ player, note: record.note }] : [];
  });
}
