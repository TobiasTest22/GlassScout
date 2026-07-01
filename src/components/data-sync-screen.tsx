"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { AlertTriangle, Check, Cpu, Database, FileSpreadsheet, FolderOpen, RefreshCw, ShieldCheck, Upload } from "lucide-react";
import type { LiveConnectorStatus, LiveFootballSnapshot } from "@/domain/adapters";
import { mergeParsedExports, parseFm26Export, type ExportDiagnostics } from "@/domain/export-watcher";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";

type BrowserFileHandle = { kind: "file"; name: string; getFile(): Promise<File> };
type BrowserDirectoryHandle = { name: string; values(): AsyncIterableIterator<BrowserFileHandle | { kind: "directory"; name: string }> };

const emptyDiagnostics: ExportDiagnostics = {
  files: [],
  columnsMatched: 0,
  sourceColumns: 0,
  missingFields: [],
  blockedColumns: [],
  importedAt: null,
  warnings: [],
};

export function DataSyncScreen({
  status,
  snapshot,
  diagnostics,
  checking,
  onCheck,
  onImport,
}: {
  status: LiveConnectorStatus;
  snapshot: LiveFootballSnapshot;
  diagnostics: ExportDiagnostics | null;
  checking: boolean;
  onCheck: () => Promise<unknown>;
  onImport: (snapshot: LiveFootballSnapshot, diagnostics: ExportDiagnostics) => void;
}) {
  const inputRef = useRef<HTMLInputElement>(null);
  const directoryInputRef = useRef<HTMLInputElement>(null);
  const [folder, setFolder] = useState<BrowserDirectoryHandle | null>(null);
  const [folderName, setFolderName] = useState<string | null>(null);
  const [watching, setWatching] = useState(false);
  const [importError, setImportError] = useState<string | null>(null);
  const visibleDiagnostics = diagnostics ?? emptyDiagnostics;

  const importFiles = useCallback(async (files: File[]) => {
    const csvFiles = files.filter((file) => file.name.toLowerCase().endsWith(".csv"));
    if (!csvFiles.length) {
      setImportError("No CSV files were selected.");
      return;
    }
    try {
      const parsed = await Promise.all(csvFiles.map(async (file) =>
        parseFm26Export(await file.text(), file.name, file.lastModified)
      ));
      const merged = mergeParsedExports(parsed, status);
      onImport(merged.snapshot, merged.diagnostics);
      setImportError(null);
    } catch (error) {
      setImportError(error instanceof Error ? error.message : "The export could not be parsed.");
    }
  }, [onImport, status]);

  const scanFolder = useCallback(async (handle: BrowserDirectoryHandle) => {
    const files: File[] = [];
    for await (const entry of handle.values()) {
      if (entry.kind === "file" && entry.name.toLowerCase().endsWith(".csv")) {
        files.push(await entry.getFile());
      }
    }
    files.sort((a, b) => b.lastModified - a.lastModified);
    await importFiles(files.slice(0, 10));
  }, [importFiles]);

  useEffect(() => {
    if (!folder || !watching) return;
    const interval = window.setInterval(() => { void scanFolder(folder); }, 5000);
    return () => window.clearInterval(interval);
  }, [folder, scanFolder, watching]);

  useEffect(() => {
    directoryInputRef.current?.setAttribute("webkitdirectory", "");
  }, []);

  const chooseFolder = async () => {
    const picker = (window as typeof window & { showDirectoryPicker?: () => Promise<BrowserDirectoryHandle> }).showDirectoryPicker;
    if (!picker) {
      directoryInputRef.current?.click();
      return;
    }
    try {
      const handle = await picker();
      setFolder(handle);
      setFolderName(handle.name);
      setWatching(true);
      await scanFolder(handle);
    } catch (error) {
      if (error instanceof DOMException && error.name === "AbortError") return;
      setImportError(error instanceof Error ? error.message : "Folder access was not granted.");
    }
  };

  return (
    <main className="screen data-sync-screen">
      <div className="planner-heading">
        <div><h1>Data / Sync Status</h1><p>Verified live-memory diagnostics and reliable visible-data imports.</p></div>
        {status.entityMapStatus !== "matched" ? <div className="entity-map-warning"><AlertTriangle />No verified entity map for this FM26 build<br /><strong>Hidden values remain blocked</strong></div> : null}
      </div>

      <section className="sync-mode-grid">
        <article className="sync-mode-panel">
          <header><Cpu /><h2>Live Memory</h2><Badge variant="outline">Not available</Badge></header>
          <dl>
            <div><dt>Process detected</dt><dd>{status.processDetected ? `Yes · PID ${status.processId}` : "No"}</dd></div>
            <div><dt>FM26 build</dt><dd>{status.gameBuild ?? "Unavailable"}</dd></div>
            <div><dt>Product version</dt><dd>{status.productVersion ?? "Unavailable"}</dd></div>
            <div><dt>Read-only memory probe</dt><dd>{status.executableHeaderValid ? `Passed · ${status.bytesRead} bytes` : "Unavailable"}</dd></div>
            <div><dt>Entity map</dt><dd className="sync-amber">{status.entityMapStatus === "matched" ? status.entityMapProfileId : "Missing"}</dd></div>
            <div><dt>Active save</dt><dd>{status.saveDetected === true ? "Detected" : "Unavailable"}</dd></div>
          </dl>
          <Button variant="outline" onClick={onCheck} disabled={checking}><RefreshCw data-icon="inline-start" className={checking ? "spin" : undefined} />Run diagnostics</Button>
        </article>

        <article className="sync-mode-panel export-panel">
          <header><FolderOpen /><h2>Export Watcher</h2><Badge variant="secondary">Recommended</Badge></header>
          <div className="export-actions">
            <Button variant="outline" onClick={chooseFolder}><FolderOpen data-icon="inline-start" />Choose export folder</Button>
            <Button onClick={() => inputRef.current?.click()}><Upload data-icon="inline-start" />Import CSV files</Button>
            <input ref={inputRef} type="file" accept=".csv,text/csv" multiple hidden onChange={(event) => { void importFiles(Array.from(event.target.files ?? [])); event.target.value = ""; }} />
            <input ref={directoryInputRef} type="file" accept=".csv,text/csv" multiple hidden onChange={(event) => {
              const files = Array.from(event.target.files ?? []);
              const relative = (files[0] as File & { webkitRelativePath?: string } | undefined)?.webkitRelativePath;
              setFolderName(relative?.split("/")[0] ?? "Selected folder");
              setWatching(false);
              void importFiles(files);
              event.target.value = "";
            }} />
          </div>
          <dl>
            <div><dt>Export folder</dt><dd>{folderName ?? "Not selected"}</dd></div>
            <div><dt>Folder watcher</dt><dd>{watching ? "Refreshing every 5 seconds" : folderName ? "Snapshot import · reselect to refresh" : "Inactive"}</dd></div>
            <div><dt>Files detected</dt><dd>{visibleDiagnostics.files.length}</dd></div>
            <div><dt>Last sync</dt><dd>{visibleDiagnostics.importedAt ? new Date(visibleDiagnostics.importedAt).toLocaleTimeString() : "Never"}</dd></div>
            <div><dt>Players imported</dt><dd>{snapshot.dataSource === "export-watcher" ? snapshot.players.length : 0}</dd></div>
            <div><dt>Columns matched</dt><dd>{visibleDiagnostics.columnsMatched} / {visibleDiagnostics.sourceColumns}</dd></div>
          </dl>
          <p>Reads user-exported visible FM26 columns. Hidden-value headers are discarded before players enter GlassScout.</p>
        </article>
      </section>

      <section className="entity-diagnostic-strip">
        <header><Database /><h2>Entity Map Diagnostic</h2></header>
        <div>
          <span><small>Module base</small><strong>{status.moduleBase ?? "Unavailable"}</strong></span>
          <span><small>Executable architecture</small><strong>{status.architecture ?? "Unavailable"}</strong></span>
          <span><small>Profile match</small><strong>{status.entityMapProfileId ?? "No"}</strong></span>
          <span><small>Pointer validation</small><strong>{status.pointerValidation?.replaceAll("_", " ") ?? "Not run"}</strong></span>
          <span><small>Executable SHA-256</small><strong title={status.executableSha256 ?? ""}>{status.executableSha256 ? `${status.executableSha256.slice(0, 12)}…` : "Unavailable"}</strong></span>
        </div>
      </section>

      {importError ? <div className="sync-error"><AlertTriangle />{importError}</div> : null}
      {visibleDiagnostics.warnings.map((warning) => <div className="sync-warning" key={warning}><ShieldCheck />{warning}</div>)}

      <section className="export-files-table">
        <header><div><FileSpreadsheet /><h2>Detected export files</h2></div><span>{visibleDiagnostics.blockedColumns.length} blocked columns</span></header>
        <div className="export-table-head"><span>File</span><span>Type</span><span>Modified</span><span>Rows</span><span>Status</span></div>
        {visibleDiagnostics.files.length ? visibleDiagnostics.files.map((file) => (
          <article key={`${file.name}-${file.modified}`}>
            <strong>{file.name}</strong>
            <span>{file.type.replace("-", " ")}</span>
            <span>{new Date(file.modified).toLocaleString()}</span>
            <span>{file.rows}</span>
            <span className={`file-status-${file.status}`}>{file.status === "ready" ? <Check /> : <AlertTriangle />}{file.status}</span>
          </article>
        )) : (
          <div className="export-empty"><FolderOpen /><strong>No export files detected</strong><span>Choose an export folder or import CSV files to get started.</span></div>
        )}
      </section>
    </main>
  );
}
