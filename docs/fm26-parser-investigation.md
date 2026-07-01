# FM26 parser investigation

Checked locally on 1 July 2026.

## Installed build

- Executable: `C:\Program Files (x86)\Steam\steamapps\common\Football Manager 26\fm.exe`
- Process: `fm.exe`
- File version: `6000.0.52.8888375`
- Product version: `6000.0.52f1-fm26-05f1 (87a0370e9917)`
- Architecture: x64 (`PE Machine 0x8664`)
- Executable SHA-256: `3653C97F9CCEC2BE28EDC4FAAE67304B5B6C26733F2F07DEA3E7C591D3B9FF73`
- Runtime: Unity IL2CPP (`UnityPlayer.dll`, `GameAssembly.dll`, and `fm_Data\il2cpp_data\Metadata\global-metadata.dat` are present)

## What works

GlassScout finds `fm.exe` through the Windows process list, opens it with query and VM-read rights, enumerates its first module, and reads the `MZ` executable header from that module base. The local probe read 64 bytes successfully. No write, VM-operation, thread-creation, or all-access permission is requested.

## Exact failure

The original connector had no entity-map loader and no FM26 offsets, signatures, or pointer chains. It proved only that the executable module was readable. It did not know:

- a signature or pointer chain for the current session/save root;
- the global player or club collections;
- the managed manager/club object;
- tactic objects;
- field layouts and string encodings;
- the game-side scout-knowledge/visibility flags.

The failure is therefore missing build-specific structural knowledge. It is not caused by process permissions, executable name/path, module-base discovery, a 32/64-bit mismatch, or FM26 not running.

## Implemented entity-map gate

`src-tauri/entity-maps/index.json` is a versioned profile index. Profiles must match all of:

- file version;
- product version;
- executable SHA-256;
- architecture.

A future profile also declares the target module, byte signatures, and pointer chains. GlassScout refuses entity traversal when a profile is missing or when validation is not implemented/passed. The current profile list is deliberately empty because no independently verified map was found for this exact build.

## Method decision

### External memory reader

This remains technically possible, but only after legal, independent reverse engineering produces a verified profile for each FM26 patch. Every chain must be guarded by readable-region, bounds, signature, collection-size, and sentinel checks. No such profile was available for this build.

### FMST26 / Genie Scout style

Current commercial/community live scouts demonstrate that extraction is possible, but their offsets and decoders are not published under a reusable open-source license. GlassScout does not copy or infer proprietary maps from binaries.

### Unity / BepInEx

The installed game is a 64-bit Unity IL2CPP build. BepInEx IL2CPP can inject plugins and generate interop wrappers, but it modifies the game launch path and is sensitive to Unity/metadata updates. That is too invasive and fragile for GlassScout's default connector. A user-chosen export plugin can still be used to create visible-column CSV files; GlassScout only consumes the resulting files.

### Export Watcher

This is the implemented reliable fallback. Users may select CSV files or grant a browser folder handle. GlassScout:

- detects semicolon, comma, or tab delimiters;
- refreshes a selected folder every five seconds;
- maps common FM26 visible columns;
- discards CA, PA, and hidden personality/consistency columns;
- derives per-90 values only when minutes and totals are visible;
- computes role DNA and transparent valuation only from imported visible fields;
- keeps unavailable metrics unavailable.

No existing FM26 export files were found in this user's Sports Interactive documents folders during the local test, so a manual FM26 export is still required for real squad ingestion.
