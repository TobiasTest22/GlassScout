"use client";

import type { CSSProperties } from "react";
import {
  AlertCircle,
  ArrowLeft,
  BarChart3,
  CheckCircle2,
  ChevronRight,
  CircleHelp,
  ClipboardList,
  GitCompareArrows,
  MapPinned,
  ShieldAlert,
  Star,
  UserRound,
} from "lucide-react";
import type { LiveFootballSnapshot, LivePlayer } from "@/domain/adapters";
import { Button } from "@/components/ui/button";
import { ConfidenceRing } from "@/components/confidence-ring";
import { PlayerFace } from "@/components/player-face";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

const technicalAttributes = ["Technique", "Dribbling", "Passing", "First Touch", "Finishing", "Crossing", "Long Shots"];
const mentalAttributes = ["Vision", "Decisions", "Composure", "Anticipation", "Work Rate", "Flair", "Teamwork"];
const physicalAttributes = ["Acceleration", "Pace", "Agility", "Balance", "Strength", "Stamina", "Natural Fitness"];

function evidenceValue(player: LivePlayer, attribute: string) {
  const value = player.attributes?.[attribute];
  return typeof value === "number" ? String(value) : "Unknown";
}

function AttributeGroup({ title, names, player }: { title: string; names: string[]; player: LivePlayer }) {
  return (
    <section className="attribute-column">
      <h3>{title}</h3>
      {names.map((name) => (
        <span key={name}><small>{name}</small><strong>{evidenceValue(player, name)}</strong></span>
      ))}
    </section>
  );
}

export function PlayerProfileScreen({
  player,
  snapshot,
  favorite,
  onToggleFavorite,
  onBack,
}: {
  player: LivePlayer | null;
  snapshot: LiveFootballSnapshot;
  favorite: boolean;
  onToggleFavorite: () => void;
  onBack: () => void;
}) {
  if (!player) {
    return (
      <main className="screen">
        <Button variant="outline" onClick={onBack}><ArrowLeft data-icon="inline-start" />Back</Button>
        <section className="favorites-empty"><h1>Player unavailable</h1><p>This player is not present in the latest visibility-safe dataset.</p></section>
      </main>
    );
  }

  const club = player.clubId ? snapshot.clubs.find((item) => item.id === player.clubId) : null;
  const mappedAttributeCount = Object.values(player.attributes ?? {}).filter((value) => typeof value === "number").length;
  const knownEvidence = player.scoutConfidence ?? Math.min(100, Math.round((mappedAttributeCount / 47) * 100));
  const unknownEvidence = 100 - knownEvidence;

  return (
    <main className="player-dossier">
      <header className="dossier-header">
        <div className="dossier-heading">
          <Button variant="ghost" size="icon" aria-label="Back to players" onClick={onBack}><ArrowLeft /></Button>
          <PlayerFace playerId={player.id} name={player.name} size="lg" />
          <div><h1>{player.name}</h1><p>Visibility-safe live FM26 dossier</p></div>
        </div>
        <div className="dossier-actions">
          <Button variant="outline" disabled><GitCompareArrows data-icon="inline-start" />Compare</Button>
          <Button variant="outline" onClick={onToggleFavorite}><Star data-icon="inline-start" />{favorite ? "Shortlisted" : "Shortlist"}</Button>
          <Button disabled><ClipboardList data-icon="inline-start" />Request full report</Button>
        </div>
      </header>

      <section className="player-facts">
        <span><b>Nationality</b><strong>{player.nationality ?? "Unknown"}</strong></span>
        <span><b>Club</b><strong>{club?.name ?? "Unknown"}</strong></span>
        <span><b>Age / DOB</b><strong>{player.age ?? "Unknown"}{player.dateOfBirth ? ` · ${player.dateOfBirth}` : ""}</strong></span>
        <span><b>Position</b><strong>{player.positions.join(" / ") || "Unknown"}</strong></span>
        <span><b>Preferred foot</b><strong>{player.preferredFoot ?? "Unknown"}</strong></span>
        <span><b>Value</b><strong>{player.value ?? "Unknown"}</strong></span>
        <span><b>Wage</b><strong>{player.wage ?? "Unknown"}</strong></span>
        <span><b>Contract</b><strong>{player.contractStatus ?? "Unknown"}</strong></span>
        <span className="fact-confidence"><b>Knowledge</b>{player.scoutConfidence == null ? <strong>Unknown</strong> : <ConfidenceRing value={player.scoutConfidence} size="sm" />}</span>
      </section>

      <Tabs defaultValue="overview" className="dossier-tabs">
        <TabsList variant="line">
          {["overview", "tactical", "attributes", "performance", "career", "notes"].map((value) => (
            <TabsTrigger key={value} value={value}>
              {value === "tactical" ? "Tactical fit" : value[0].toUpperCase() + value.slice(1)}
            </TabsTrigger>
          ))}
        </TabsList>

        <TabsContent value="overview">
          <div className="dossier-layout">
            <div className="dossier-main-column">
              <section className="dossier-panel scout-summary-panel">
                <header><UserRound /><h2>Scout summary</h2></header>
                <p>
                  Live memory confirms <strong>{player.name}</strong> ({player.nationality ?? "nationality unknown"}, age {player.age ?? "unknown"}) belongs to {club?.name ?? "the current club"}.
                  The role evidence score uses only visible FM26 attributes for {player.bestRole ?? player.positions[0] ?? "the player’s position"}; hidden current and potential ability are never read.
                </p>
                <div className="summary-columns">
                  <div><h3><CheckCircle2 />Strengths</h3><span>{player.strengths.length ? player.strengths.join(" · ") : "No visible evidence"}</span></div>
                  <div><h3><AlertCircle />Weaknesses</h3><span>{player.weaknesses.length ? player.weaknesses.join(" · ") : "No visible evidence"}</span></div>
                  <div><h3><ClipboardList />Recommended next action</h3><span>{snapshot.tactic ? "Review formation-specific fit." : "Import an FMF tactic before judging tactical fit."}</span></div>
                </div>
              </section>

              <section className="dossier-panel role-fit-panel">
                <header><MapPinned /><h2>{snapshot.tactic ? "Role & tactical fit" : "Role evidence"}</h2></header>
                <div className="role-fit-layout">
                  <div className="role-fit-metrics">
                    <span><small>Best role</small><strong>{player.bestRole ?? "Unknown"}</strong></span>
                    <span><small>{snapshot.tactic ? "Tactical fit" : "Role evidence"}</small><strong>{player.roleFit == null ? "Unknown" : `${player.roleFit}%`}</strong></span>
                    <span><small>Knowledge</small><strong>{player.scoutConfidence == null ? "Unknown" : `${player.scoutConfidence}%`}</strong></span>
                  </div>
                  <div className="mini-role-map">
                    <span className="mini-box left" /><span className="mini-box right" /><span className="mini-half" /><span className="mini-centre" />
                    <i>Role map unavailable</i>
                  </div>
                  <div className="fit-legend">
                    <span><i data-tone="strong" />Strong fit</span>
                    <span><i data-tone="good" />Good fit</span>
                    <span><i data-tone="suitable" />Suitable</span>
                    <span><i data-tone="low" />Low fit</span>
                  </div>
                </div>
              </section>

              <section className="dossier-panel attribute-evidence-panel">
                <header><BarChart3 /><h2>Attribute evidence</h2><span>Visible values only</span></header>
                <div className="attribute-grid">
                  <AttributeGroup title="Technical" names={technicalAttributes} player={player} />
                  <AttributeGroup title="Mental" names={mentalAttributes} player={player} />
                  <AttributeGroup title="Physical" names={physicalAttributes} player={player} />
                </div>
              </section>
            </div>

            <aside className="dossier-side-column">
              <section className="dossier-panel knowledge-panel">
                <header><CircleHelp /><h2>Knowledge profile</h2></header>
                <div className="knowledge-overview">
                  <span className="knowledge-donut" style={{ "--known": `${knownEvidence}%` } as CSSProperties} />
                  <div>
                    <span><i data-tone="known" />Known mapped evidence <strong>{knownEvidence}%</strong></span>
                    <span><i data-tone="estimated" />Estimated <strong>0%</strong></span>
                    <span><i data-tone="unknown" />Unknown <strong>{unknownEvidence}%</strong></span>
                  </div>
                </div>
                <dl>
                  <div><dt>Last scouted</dt><dd>{player.lastScoutedDate ?? "Unknown"}</dd></div>
                  <div><dt>Observations</dt><dd>Unknown</dd></div>
                  <div><dt>Report reliability</dt><dd>{player.reportReliability ?? "Unknown"}</dd></div>
                </dl>
              </section>

              <section className="dossier-panel compact-dossier-panel">
                <header><ShieldAlert /><h2>Risk profile</h2></header>
                {["Adaptation", "Wage", "Tactical mismatch", "Injury history", "Registration"].map((risk) => (
                  <button key={risk}><span><i />{risk}</span><strong>Unknown</strong><ChevronRight /></button>
                ))}
              </section>

              <section className="dossier-panel career-panel">
                <header><BarChart3 /><h2>Career & form</h2></header>
                <div className="form-bars">{Array.from({ length: 10 }, (_, index) => <i key={index} />)}</div>
                <span><small>Current form</small><strong>{player.averageRating?.toFixed(2) ?? "Unknown"}</strong></span>
                <div className="career-stats">
                  <span><small>Appearances</small><strong>Unknown</strong></span>
                  <span><small>Goals</small><strong>{player.goals ?? "Unknown"}</strong></span>
                  <span><small>Assists</small><strong>{player.assists ?? "Unknown"}</strong></span>
                </div>
                <p>From visible data only</p>
              </section>

              <section className="dossier-panel decision-panel">
                <header><ClipboardList /><h2>Decision history</h2></header>
                <div><span>No visibility-safe decisions recorded</span></div>
              </section>
            </aside>
          </div>
        </TabsContent>

        {["tactical", "attributes", "performance", "career", "notes"].map((value) => (
          <TabsContent value={value} key={value}>
            <section className="dossier-locked-state">
              <ShieldAlert />
              <h2>{value === "tactical" ? "Tactical fit" : value[0].toUpperCase() + value.slice(1)} evidence is not available</h2>
              <p>GlassScout will populate this section only from FM26 data that is visible to the user’s club.</p>
            </section>
          </TabsContent>
        ))}
      </Tabs>
    </main>
  );
}
