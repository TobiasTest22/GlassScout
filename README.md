# GlassScout FM26

GlassScout FM26 is a desktop-only recruitment, squad and tactics workspace designed to read the active Football Manager 2026 process locally.

## Current status

The desktop connector can currently:

- detect a running `fm.exe` process;
- open it with query and VM-read permissions only;
- resolve the executable path;
- read and validate the executable `MZ` header;
- return an explicit structured error when the active save entity map is unavailable.

The connector cannot yet locate FM26's active-save entities for the installed 2026 build. Managed team, players, clubs, attributes and tactics therefore remain unreadable. Every data collection returned by the connector is empty; the UI shows a connection/data error and never substitutes demo records.

This is an honest pre-release foundation, not a claim that FM26 save extraction is complete. See [FM26 live-memory research](docs/fm26-live-memory-research.md).

## Desktop product surfaces

- My Team, grouped by live FM26 position data when available.
- Current Tactic, roles, duties, instructions, conflicts and squad-fit analysis when readable.
- Recruitment filters over the live player pool.
- Favorited Players with add/remove, notes, filtering, sorting and comparison. Favorites store only player IDs and resolve against the newest live snapshot.
- Memory Center diagnostics for process, memory, parser and active-save readiness.
- No CSV flow, Import Center, fake team, placeholder player, seeded database or static fallback data.

## Development

```powershell
npm install
npm test
npm run lint
npm run build
npm run dev
```

The web target is only a UI preview. Live FM26 inspection requires the Tauri desktop runtime:

```powershell
npm run desktop:dev
```

The standalone read-only process probe is:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/fm26-read-probe.ps1
```

## Release

- `.github/workflows/ci.yml` validates frontend and Rust tests on Windows.
- `.github/workflows/desktop-release.yml` creates a draft prerelease.

Releases must remain prerelease until active-save detection and entity extraction are verified against the installed FM26 build.
