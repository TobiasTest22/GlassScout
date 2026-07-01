# FM26 live-memory research

Checked: 1 July 2026

## Local FM26 result

FM26 was running locally as `fm.exe` from the Steam installation. A diagnostic probe opened the process with only:

- `PROCESS_QUERY_LIMITED_INFORMATION`
- `PROCESS_VM_READ`

It then read 64 bytes from the executable module base. The read succeeded and returned the expected `MZ` executable header. No write, operation, thread, or all-access rights were requested.

This proves that a local GlassScout desktop process can obtain read-only access to the running FM26 process in this environment. It does **not** prove that FM26 entity offsets, pointer graphs, save discovery, or visibility semantics are known.

## Open-source architecture research

### FMScoutFramework

[ThanosSiopoudis/FMScoutFramework](https://github.com/ThanosSiopoudis/FMScoutFramework) is an older real-time Football Manager scout framework. Its useful architectural ideas are:

- isolate process access from object/entity managers;
- keep version-specific addresses and offsets behind interfaces;
- describe entity fields with dedicated offset definitions;
- separate process detection, memory reads, object traversal, and presentation.

The project is GPL-2.0 and targets old FM generations. GlassScout does not copy its code or offsets.

### fm-explorer

[robeady/fm-explorer](https://github.com/robeady/fm-explorer) is older exploratory tooling around Football Manager data and FMScoutFramework ideas. It is useful as historical evidence that stable entity discovery requires build-specific research, not as a compatible FM26 dependency.

### FM Editor Live research

[caprolt/fmeditor](https://github.com/caprolt/fmeditor) contains decompiled historical editor code showing the classic architecture: find `fm.exe`, derive the module base, select a build-specific offset set, locate global arrays, then traverse typed entities. That source also contains write paths and all-access process handles. GlassScout deliberately rejects those parts: its connector requests query + VM-read only, exposes `canWriteMemory: false`, and contains no write-memory API.

## GlassScout adapter design

The FM26 connector now has four separate readiness stages:

1. **Process detected** — `fm.exe` exists.
2. **Read-only handle open** — Windows grants query + VM-read rights.
3. **Parser verified** — offsets and pointer validation match the installed FM26 build.
4. **Save/entities loaded** — the parser identifies a loaded save and produces visibility-filtered players/clubs.

Stages 1 and 2 work locally. Stages 3 and 4 remain disabled until the installed FM26 build is reverse-engineered and validated.

## Safe parser plan

- Fingerprint the installed executable version and selected module hashes.
- Keep offsets in versioned data files, never inline in UI code.
- Validate every pointer: readable memory region, alignment, sensible collection bounds, stable sentinel fields.
- Require multiple independent structural checks before treating a candidate as a player or club collection.
- Read into raw connector records.
- Apply the visibility filter before SQLite, scoring, UI, or assistant access.
- Reject the entire parser version when invariants fail after an FM26 patch.
- Never request `PROCESS_VM_WRITE`, `PROCESS_VM_OPERATION`, thread creation, or all-access rights.

## Run the local probe

With FM26 running:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/fm26-read-probe.ps1
```

Expected result:

- `ProcessDetected: true`
- `ReadSucceeded: true`
- `Header: MZ`
- `WriteRightsRequested: false`

Entity counts remain zero until a compatible parser definition is installed.
