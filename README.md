# GlassScout FM26

GlassScout FM26 is a desktop-only recruitment, squad and tactics workspace designed to read the active Football Manager 2026 process locally.

## Current status

The desktop connector can currently:

- detect a running `fm.exe` process;
- open it with query and VM-read permissions only;
- require an exact file version, product version, architecture and SHA-256 match;
- resolve the active human manager, managed club and current squad;
- read live player identity and positional familiarity for the managed team;
- validate signatures, object types, collection bounds and pointer chains before returning entities.

Version `0.1.3` supports FM26 `6000.0.52.8888375`, product `6000.0.52f1-fm26-05f1 (87a0370e9917)`, x64, executable SHA-256 `3653C97F9CCEC2BE28EDC4FAAE67304B5B6C26733F2F07DEA3E7C591D3B9FF73`.

Live-memory tactic reading is disabled and cannot block squad/player connection. The user may choose one `.fmf` tactic file through the native Windows file picker. GlassScout validates and copies that file into local app data, then reports whether its format can be decoded. The current FM26 FMF container is recognized but not yet fully decoded, so no formation, role, duty or tactic-fit result is guessed.

Age, attributes, form, contracts, wages, valuations and FM26's own shortlist are not yet mapped safely for this exact build. GlassScout leaves those fields empty and explains the limitation. It does not read or return CA, PA or hidden attributes.

GlassScout is live-game only. When the installed FM26 build has no verified entity map, the application shows a clean blocking connection state and does not substitute another data source.

## Desktop product surfaces

- My Team, grouped by validated live FM26 position data.
- Tactic Evaluation with a user-selected `.fmf` file, local app-data storage and honest parser status.
- Recruitment filters over the live player pool.
- Favorited Players with add/remove, notes, filtering, sorting and comparison. Favorites store only player IDs and resolve against the newest live snapshot.
- Advanced diagnostics under Settings with the exact build fingerprint and entity-map status.
- Role DNA / Position Converter and transparent true-price estimation from visible attributes and performance.
- No alternate data flow, fake team, placeholder player or seeded database.

## Install the Windows test build

Download [GlassScout FM26 0.1.3 for Windows x64](https://github.com/TobiasTest22/GlassScout/releases/download/app-v0.1.3/GlassScout.FM26_0.1.3_x64-setup.exe).

The NSIS setup installs GlassScout locally and uses Tauri's WebView2 bootstrapper when the required Windows web runtime is missing. The prerelease is not code-signed yet, so Windows SmartScreen may ask for confirmation.

## Development

```powershell
npm install
npm test
npm run lint
npm run build
npm run dev
```

Live FM26 inspection requires the Tauri desktop runtime:

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
- Every application change must include a version bump, a rebuilt Windows installer, an installed-app launch check, and an updated release download.

The exact-build map must be revalidated whenever FM26 changes.
