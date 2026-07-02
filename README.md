# GlassScout FM26

GlassScout FM26 is a desktop-only recruitment, squad and tactics workspace designed to read the active Football Manager 2026 process locally.

## Current status

The desktop connector can currently:

- detect a running `fm.exe` process;
- open it with query and VM-read permissions only;
- require an exact file version, product version, architecture and SHA-256 match;
- resolve the active human manager, managed club and current squad;
- read live player identity and positional familiarity for the managed team;
- derive the validated FM26 player-object type and index the wider save database from readable private memory;
- keep every non-squad index record behind a visibility gate until club scout knowledge can be mapped safely;
- validate signatures, object types, collection bounds and pointer chains before returning entities.

Version `0.1.7` supports FM26 `6000.0.52.8888375`, product `6000.0.52f1-fm26-05f1 (87a0370e9917)`, x64, executable SHA-256 `3653C97F9CCEC2BE28EDC4FAAE67304B5B6C26733F2F07DEA3E7C591D3B9FF73`.

Live-memory tactic inspection cannot block squad/player connection. GlassScout detects the active FM26 tactic manager with read-only access; formation, phase roles, duties and instructions remain unavailable until their packed build-specific layout is validated. No formation or fit result is guessed.

The verified live test indexed 35,874 player records in the active save: 38 managed-squad records were visibility-safe and 35,836 wider-save records remained behind the knowledge gate. Managed-squad FM ID, name, date of birth, age, nationality, positional familiarity, preferred foot and 47 non-hidden attributes are mapped. Form, contract terms/expiry, wages, valuations, relationship-based scout knowledge, interest, attribute ranges, live tactic slots and FM26's own shortlist remain candidate or unmapped for this exact build. GlassScout leaves those fields empty until validation passes. It does not read or return CA, PA or hidden attributes.

GlassScout is live-game only. When the installed FM26 build has no verified entity map, the application shows a clean blocking connection state and does not substitute another data source.

## Desktop product surfaces

- Screenshot-matched command dashboard with active-club context, tactical board, recruitment pulse, department briefing and squad health.
- Squad grouped by validated live FM26 position data.
- Tactical Board reserved for validated active-save formation, phase roles, duties and instructions.
- Recruitment Hub with visibility, interest, realism, financial, availability and risk fields that remain `Unknown` until real evidence exists.
- Player dossiers matching the supplied scouting-report structure without filling unsupported fields.
- Shortlist with add/remove, notes, filtering, sorting and comparison. Shortlist records store only player IDs and resolve against the newest live snapshot.
- Advanced diagnostics under Settings with the exact build fingerprint, schema-v2 field coverage and candidate/unmapped counts.
- Role DNA / Position Converter and transparent true-price estimation from visible attributes and performance.
- No alternate data flow, fake team, placeholder player or seeded database.

## Install the Windows test build

Download [GlassScout FM26 0.1.7 for Windows x64](https://github.com/TobiasTest22/GlassScout/releases/download/app-v0.1.7/GlassScout.FM26_0.1.7_x64-setup.exe).

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

The local, developer-only Mapping Lab is enabled explicitly and never in the normal product flow:

```powershell
$env:GLASSSCOUT_MAPPING_MODE = "1"
npm run desktop:dev
```

It captures bounded player/person/contract windows to the local app-data `mapping-lab` directory and produces JSON snapshot/diff evidence. It accepts only an indexed FM ID or an exact unique player name, never an arbitrary process address. Candidate offsets remain excluded from live product data until the schema-v2 confidence and validation gate passes.

For the 0.1.7 validation pass, FM ID `2000478798` (Elias Dale, GK) was captured under FM26's `Interested` and `Doubtful` recruitment filters. The generated comparison found `0` changed and `3072` unchanged bytes across the bounded player, person and contract windows. That is useful negative evidence: player interest is relationship-owned and must not be guessed from the player object. The local JSON snapshots and comparison remain in app data rather than the repository.

The standalone read-only process probe is:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/fm26-read-probe.ps1
```

## Release

- `.github/workflows/ci.yml` validates frontend and Rust tests on Windows.
- `.github/workflows/desktop-release.yml` creates a draft prerelease.
- Every application change must include a version bump, a rebuilt Windows installer, an installed-app launch check, and an updated release download.

The exact-build map must be revalidated whenever FM26 changes.
