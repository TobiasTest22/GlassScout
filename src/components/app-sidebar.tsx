"use client";

import {
  Binoculars,
  CalendarDays,
  ChevronLeft,
  ClipboardList,
  LayoutDashboard,
  PanelsTopLeft,
  SearchCheck,
  Settings,
  ShieldCheck,
  Star,
  UserRoundSearch,
  UsersRound,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

export type Screen =
  | "Dashboard"
  | "Squad Planner"
  | "Tactical Board"
  | "Scout Room"
  | "Recruitment"
  | "Players"
  | "Shortlist"
  | "Reports"
  | "Player Profile"
  | "Settings";

const navigation = [
  { label: "Dashboard", icon: LayoutDashboard },
  { label: "Squad Planner", icon: UsersRound },
  { label: "Tactical Board", icon: PanelsTopLeft },
  { label: "Scout Room", icon: Binoculars },
  { label: "Recruitment", icon: SearchCheck },
  { label: "Players", icon: UserRoundSearch },
  { label: "Shortlist", icon: Star },
  { label: "Reports", icon: ClipboardList },
] satisfies { label: Screen; icon: typeof LayoutDashboard }[];

export function AppSidebar({
  screen,
  onNavigate,
}: {
  screen: Screen;
  onNavigate: (screen: Screen) => void;
}) {
  return (
    <aside className="app-sidebar">
      <button className="brand" onClick={() => onNavigate("Dashboard")} aria-label="Go to dashboard">
        <span className="brand-mark" aria-hidden="true"><span /></span>
        <span className="brand-copy"><strong>GlassScout</strong><small>FM26</small></span>
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

      <div className="sidebar-spacer" />
      <section className="revealed-data-card">
        <ShieldCheck />
        <div>
          <strong>Revealed data only</strong>
          <span>Hidden values blocked before scoring.</span>
        </div>
      </section>
      <Button
        variant="ghost"
        className={cn("nav-item", screen === "Settings" && "nav-item-active")}
        onClick={() => onNavigate("Settings")}
      >
        <Settings data-icon="inline-start" />
        <span>Settings</span>
      </Button>
      <div className="sidebar-footer">
        <span className="manager-avatar">TB</span>
        <span><strong>Recruitment desk</strong><small>Live FM26</small></span>
        <CalendarDays aria-hidden="true" />
      </div>
      <button className="sidebar-collapse" aria-label="Collapse sidebar"><ChevronLeft /></button>
    </aside>
  );
}
