"use client";

import {
  BrainCircuit,
  Database,
  Dna,
  Gauge,
  LayoutDashboard,
  ListChecks,
  Star,
  Settings,
  Target,
  UsersRound,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";

export type Screen =
  | "Dashboard"
  | "My Team"
  | "Tactic Evaluation"
  | "Role DNA"
  | "Recruitment"
  | "Favorites / Shortlist"
  | "Data / Sync Status"
  | "Settings";

const navigation = [
  { label: "Dashboard", icon: LayoutDashboard },
  { label: "My Team", icon: UsersRound },
  { label: "Tactic Evaluation", icon: Target },
  { label: "Role DNA", icon: Dna },
  { label: "Recruitment", icon: ListChecks },
  { label: "Favorites / Shortlist", icon: Star },
  { label: "Data / Sync Status", icon: Database },
  { label: "Settings", icon: Settings },
] satisfies { label: Screen; icon: typeof Gauge }[];

export function AppSidebar({ screen, onNavigate }: { screen: Screen; onNavigate: (screen: Screen) => void }) {
  return (
    <aside className="app-sidebar">
      <button className="brand" onClick={() => onNavigate("Dashboard")} aria-label="Go to dashboard">
        <span className="brand-mark" aria-hidden="true">
          <span />
        </span>
        <span className="brand-copy">
          <strong>GlassScout</strong>
          <small>FM26</small>
        </span>
      </button>

      <nav className="nav-list" aria-label="Main navigation">
        {navigation.map(({ label, icon: Icon }) => (
          <Button
            key={label}
            variant="ghost"
            className={cn("nav-item", screen === label && "nav-item-active")}
            onClick={() => onNavigate(label)}
          >
            <Icon data-icon="inline-start" />
            <span>{label}</span>
          </Button>
        ))}
      </nav>

      <Tooltip>
        <TooltipTrigger render={<Button variant="ghost" size="icon" className="assistant-button" />}>
          <BrainCircuit />
          <span className="sr-only">Open recruitment assistant</span>
        </TooltipTrigger>
        <TooltipContent side="right">Recruitment assistant</TooltipContent>
      </Tooltip>
    </aside>
  );
}
