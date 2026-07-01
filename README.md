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

The installed build is `6000.0.52.8888375`, x64 Unity IL2CPP. GlassScout now fingerprints the executable and requires an exact versioned entity-map match before following any pointer. See [FM26 parser investigation](docs/fm26-parser-investigation.md).

When no verified map exists, **Export Watcher** is the supported workflow. It imports FM26 visible-column CSV exports in the browser or desktop app, can refresh a user-selected folder every five seconds, and feeds My Team, Recruitment, Role DNA and valuation screens without exposing hidden values.

## Desktop product surfaces

- My Team, grouped by live FM26 position data when available.
- Current Tactic, roles, duties, instructions, conflicts and squad-fit analysis when readable.
- Recruitment filters over the live player pool.
- Favorited Players with add/remove, notes, filtering, sorting and comparison. Favorites store only player IDs and resolve against the newest live snapshot.
- Data / Sync Status with exact build fingerprint, entity-map status, CSV import and browser folder watching.
- Role DNA / Position Converter and transparent true-price estimation from visible attributes and performance.
- No CSV flow, Import Center, fake team, placeholder player, seeded database or static fallback data.

## Install the Windows test build

Download [GlassScout FM26 0.1.0 for Windows x64](https://github.com/TobiasTest22/GlassScout/releases/download/app-v0.1.0/GlassScout.FM26_0.1.0_x64-setup.exe).

The NSIS setup installs GlassScout locally and uses Tauri's WebView2 bootstrapper when the required Windows web runtime is missing. The prerelease is not code-signed yet, so Windows SmartScreen may ask for confirmation. Installing the desktop app enables the local connector; it does not make the still-unverified active-save entity parser complete.

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
