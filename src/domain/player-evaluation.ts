import type { LivePlayer } from "./adapters";

type RoleProfile = {
  position: string;
  role: string;
  weights: Record<string, number>;
};

const roleProfiles: RoleProfile[] = [
  { position: "GK", role: "Sweeper Keeper", weights: { reflexes: 1.4, handling: 1.2, oneonones: 1.1, rushingout: 1, passing: .8, decisions: .9, acceleration: .5 } },
  { position: "CB", role: "Ball-Playing Centre-Back", weights: { marking: 1.2, tackling: 1.2, positioning: 1.2, jumpingreach: 1, strength: 1, pace: .8, passing: .8, decisions: .8, composure: .7 } },
  { position: "FB/WB", role: "Complete Wing-Back", weights: { pace: 1, acceleration: 1, stamina: 1.1, workrate: 1, crossing: 1, dribbling: .8, tackling: .8, positioning: .7 } },
  { position: "DM", role: "Deep-Lying Playmaker", weights: { passing: 1.2, vision: 1.2, decisions: 1, firsttouch: .8, positioning: .8, teamwork: .7, composure: .7 } },
  { position: "CM", role: "Box-to-Box Midfielder", weights: { stamina: 1.2, workrate: 1.1, passing: .9, decisions: .9, offball: .8, tackling: .7, technique: .7, acceleration: .6 } },
  { position: "AM", role: "Advanced Playmaker", weights: { passing: 1.2, vision: 1.3, technique: 1, firsttouch: 1, decisions: .9, dribbling: .8, flair: .7 } },
  { position: "W", role: "Inside Forward", weights: { acceleration: 1.2, pace: 1.1, dribbling: 1.1, technique: .8, finishing: .8, offball: .8, decisions: .7 } },
  { position: "ST", role: "Complete Forward", weights: { finishing: 1.2, offball: 1.1, composure: 1, acceleration: .8, pace: .8, firsttouch: .8, strength: .7, heading: .6 } },
];

export type RoleEvaluation = {
  position: string | null;
  role: string | null;
  score: number | null;
  reasoning: string[];
};

export function evaluateRoleDna(attributes: Record<string, number | null>): RoleEvaluation {
  const evaluations = roleProfiles.flatMap((profile) => {
    let weighted = 0;
    let totalWeight = 0;
    const contributors: Array<[string, number, number]> = [];
    for (const [attribute, weight] of Object.entries(profile.weights)) {
      const value = attributes[attribute];
      if (value == null || value < 1 || value > 20) continue;
      weighted += value * weight;
      totalWeight += weight;
      contributors.push([attribute, value, weight]);
    }
    if (contributors.length < 3 || totalWeight === 0) return [];
    const coverage = contributors.length / Object.keys(profile.weights).length;
    const score = Math.round((weighted / totalWeight / 20) * 100 * (.75 + (.25 * coverage)));
    const strengths = contributors.toSorted((a, b) => (b[1] * b[2]) - (a[1] * a[2])).slice(0, 3);
    return [{ ...profile, score, strengths }];
  });
  const best = evaluations.toSorted((a, b) => b.score - a.score)[0];
  if (!best) return { position: null, role: null, score: null, reasoning: ["At least three relevant visible attributes are required."] };
  return {
    position: best.position,
    role: best.role,
    score: best.score,
    reasoning: best.strengths.map(([name, value]) => `${humanize(name)} ${value}`),
  };
}

export function estimateTruePrice(player: Pick<LivePlayer, "marketValueAmount" | "age" | "averageRating" | "minutesPlayed" | "roleFit" | "contractStatus">) {
  const marketValue = player.marketValueAmount;
  if (marketValue == null || marketValue <= 0) {
    return { estimate: null, range: null, label: "unavailable" as const, reasoning: ["A readable FM market value is required."] };
  }
  let multiplier = 1;
  const reasoning = ["Starts from the visible FM market value."];
  if (player.age != null && player.age <= 23) { multiplier += .15; reasoning.push("Young-player resale premium +15%."); }
  if (player.age != null && player.age >= 30) { multiplier -= .15; reasoning.push("Age/resale adjustment -15%."); }
  if (player.averageRating != null && player.averageRating >= 7.1) { multiplier += .1; reasoning.push("Strong current performance +10%."); }
  if (player.minutesPlayed != null && player.minutesPlayed < 450) { multiplier -= .08; reasoning.push("Limited-minute evidence -8%."); }
  if (player.roleFit != null && player.roleFit >= 80) { multiplier += .08; reasoning.push("High role fit +8%."); }
  if (player.contractStatus?.toLowerCase().includes("expir")) { multiplier -= .18; reasoning.push("Expiring-contract adjustment -18%."); }
  multiplier = Math.max(.55, Math.min(1.55, multiplier));
  const estimate = roundMoney(marketValue * multiplier);
  const range: [number, number] = [roundMoney(estimate * .88), roundMoney(estimate * 1.12)];
  const ratio = estimate / marketValue;
  const label: "undervalued" | "fair" | "overpriced" = ratio >= 1.12 ? "undervalued" : ratio <= .88 ? "overpriced" : "fair";
  return { estimate, range, label, reasoning };
}

function roundMoney(value: number) {
  return Math.round(value * 10) / 10;
}

function humanize(value: string) {
  return value.replace(/([a-z])([A-Z])/g, "$1 $2").replace(/^./, (letter) => letter.toUpperCase());
}
