"use client";

import { ArrowLeft, BadgeEuro, BrainCircuit, ChartNoAxesCombined, ShieldCheck, Sparkles } from "lucide-react";
import type { LiveFootballSnapshot, LivePlayer } from "@/domain/adapters";
import { Button } from "@/components/ui/button";

function sortedAttributes(player: LivePlayer) {
  return Object.entries(player.attributes ?? {})
    .filter((entry): entry is [string, number] => typeof entry[1] === "number")
    .toSorted((a, b) => b[1] - a[1]);
}

export function PlayerProfileScreen({
  player,
  snapshot,
  onBack,
}: {
  player: LivePlayer | null;
  snapshot: LiveFootballSnapshot;
  onBack: () => void;
}) {
  if (!player) {
    return (
      <main className="screen">
        <Button variant="outline" onClick={onBack}><ArrowLeft />Back</Button>
        <section className="favorites-empty"><h1>Player unavailable</h1><p>This player is not present in the latest verified dataset.</p></section>
      </main>
    );
  }

  const club = player.clubId ? snapshot.clubs.find((item) => item.id === player.clubId) : null;
  const attributes = sortedAttributes(player);
  const strengths = attributes.slice(0, 4);
  const weaknesses = attributes.length >= 6 ? attributes.slice(-3).reverse() : [];
  const fairRange = player.fairPriceRange
    ? `€${player.fairPriceRange[0]}m–€${player.fairPriceRange[1]}m`
    : "Unavailable";
  const recommendation = player.riskLevel === "high"
    ? "Scout further before making a decision."
    : player.valuationLabel === "undervalued"
      ? "Add to the priority shortlist."
      : player.roleFit != null && player.roleFit >= 75
        ? "Strong tactical candidate; review contract and price."
        : "Keep under observation.";

  return (
    <main className="screen full-profile-screen">
      <Button variant="outline" onClick={onBack}><ArrowLeft />Back to players</Button>

      <section className="profile-hero player-profile-hero">
        <span className="profile-monogram">{player.name.split(" ").map((part) => part[0]).join("").slice(0, 2)}</span>
        <div>
          <h1>{player.name}</h1>
          <p>{player.age ?? "Age unknown"} · {player.nationality ?? "Nationality unknown"} · {club?.name ?? "Club unknown"}</p>
        </div>
        <span className={`profile-knowledge knowledge-${player.scoutKnowledge ?? "unknown"}`}>
          {(player.scoutKnowledge ?? "unknown").replaceAll("_", " ")}
        </span>
      </section>

      <div className="player-profile-grid">
        <section className="profile-panel">
          <header><BrainCircuit /><h2>Role DNA</h2></header>
          <div className="profile-detail-list">
            <span><small>Known positions</small><strong>{player.positions.join(" / ") || "Unavailable"}</strong></span>
            <span><small>Calculated position</small><strong>{player.bestCalculatedPosition ?? "Unavailable"}</strong></span>
            <span><small>Best role</small><strong>{player.bestRole ?? "Unavailable"}</strong></span>
            <span><small>Role / tactical fit</small><strong>{player.roleFit ?? "—"}% / {player.tacticalFit ?? "—"}%</strong></span>
          </div>
        </section>

        <section className="profile-panel">
          <header><BadgeEuro /><h2>Contract & valuation</h2></header>
          <div className="profile-detail-list">
            <span><small>FM value</small><strong>{player.value ?? "Unavailable"}</strong></span>
            <span><small>Estimated true price</small><strong>{player.truePrice == null ? "Unavailable" : `€${player.truePrice}m`}</strong></span>
            <span><small>Fair price range</small><strong>{fairRange}</strong></span>
            <span><small>Wage / contract</small><strong>{player.wage ?? "Unknown"} · {player.contractStatus ?? "Unknown"}</strong></span>
          </div>
        </section>

        <section className="profile-panel">
          <header><ChartNoAxesCombined /><h2>Visible performance</h2></header>
          <div className="profile-detail-list">
            <span><small>Average rating</small><strong>{player.averageRating?.toFixed(2) ?? "Unavailable"}</strong></span>
            <span><small>Goals per 90</small><strong>{player.per90?.goalsPer90 ?? "Unavailable"}</strong></span>
            <span><small>Assists per 90</small><strong>{player.per90?.assistsPer90 ?? "Unavailable"}</strong></span>
            <span><small>Minutes</small><strong>{player.minutesPlayed ?? "Unavailable"}</strong></span>
          </div>
        </section>

        <section className="profile-panel">
          <header><Sparkles /><h2>Development decision</h2></header>
          <div className="profile-decision">
            <strong>{player.retrainingSuggestion ?? "No evidence-backed retraining signal"}</strong>
            <p>{player.roleReasoning?.join(" · ") || "More visible attributes are required for a detailed role explanation."}</p>
            <span>{recommendation}</span>
          </div>
        </section>

        <section className="profile-panel profile-wide">
          <header><ShieldCheck /><h2>Strengths, weaknesses & evidence</h2></header>
          <div className="profile-evidence-grid">
            <div><small>Strongest visible attributes</small>{strengths.length ? strengths.map(([name, value]) => <span key={name}>{name}<strong>{value}</strong></span>) : <p>Attribute data unavailable.</p>}</div>
            <div><small>Lowest visible attributes</small>{weaknesses.length ? weaknesses.map(([name, value]) => <span key={name}>{name}<strong>{value}</strong></span>) : <p>Not enough visible attributes.</p>}</div>
            <div><small>Valuation reasoning</small>{player.valuationReasoning?.length ? player.valuationReasoning.map((reason) => <p key={reason}>{reason}</p>) : <p>True-price evidence unavailable.</p>}</div>
          </div>
        </section>
      </div>
    </main>
  );
}
