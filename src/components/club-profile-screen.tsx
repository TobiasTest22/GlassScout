"use client";

import { ArrowLeft, UsersRound } from "lucide-react";
import type { LiveFootballSnapshot } from "@/domain/adapters";
import { Button } from "@/components/ui/button";
import { ClubLogo } from "@/components/club-logo";
import { PlayerFace } from "@/components/player-face";

export function ClubProfileScreen({ clubId, snapshot, onBack, onOpenPlayer }: {
  clubId: string | null;
  snapshot: LiveFootballSnapshot;
  onBack: () => void;
  onOpenPlayer: (id: string) => void;
}) {
  const club = snapshot.clubs.find((item) => item.id === clubId);
  if (!club) return <main className="screen"><Button variant="outline" onClick={onBack}><ArrowLeft data-icon="inline-start" />Back</Button><section className="favorites-empty"><h1>Club unavailable</h1><p>This team is not mapped in the current live snapshot.</p></section></main>;
  const squad = snapshot.players.filter((player) => player.clubId === club.id);
  return (
    <main className="screen club-profile-screen">
      <header className="club-profile-header"><Button variant="ghost" size="icon" onClick={onBack}><ArrowLeft /></Button><ClubLogo clubId={club.id} name={club.name} size="lg" /><div><h1>{club.name}</h1><p>{club.nation ?? "Nation unknown"} · {club.league ?? "Competition unknown"}</p></div><span><UsersRound />{squad.length} mapped players</span></header>
      <section className="club-squad-grid">
        {squad.map((player) => <button key={player.id} onClick={() => onOpenPlayer(player.id)}><PlayerFace playerId={player.id} name={player.name} size="md" highResolution /><span><strong>{player.name}</strong><small>{player.positions.join(" / ")} · {player.age ?? "Age unknown"}</small><b>{player.bestRole ?? "Role unknown"}</b></span></button>)}
      </section>
    </main>
  );
}
