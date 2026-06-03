import { useCallback, useEffect, useState } from "react";
import { Sidebar, type ViewKey } from "./components/Sidebar";
import { AtomTree } from "./views/AtomTree";
import { Compliance } from "./views/Compliance";
import { Loudness } from "./views/Loudness";
import { Overview } from "./views/Overview";
import { RawProbe } from "./views/RawProbe";
import { Streams } from "./views/Streams";
import { inspect, pickFile, toolStatus } from "./lib/ipc";
import type { InspectReport, ToolStatus } from "./lib/types";

export default function App() {
  const [view, setView] = useState<ViewKey>("overview");
  const [report, setReport] = useState<InspectReport | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [tools, setTools] = useState<ToolStatus | null>(null);

  useEffect(() => {
    toolStatus().then(setTools).catch(() => setTools(null));
  }, []);

  const open = useCallback(async () => {
    setError(null);
    const path = await pickFile();
    if (!path) return;
    setLoading(true);
    try {
      const r = await inspect(path);
      setReport(r);
      setView("overview");
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  return (
    <div className="h-full flex flex-col">
      <Header tools={tools} onOpen={open} loading={loading} />
      <div className="flex-1 flex min-h-0">
        <Sidebar
          current={view}
          onSelect={setView}
          disabled={{
            atoms: !report?.atoms,
            loudness: !report?.loudness,
            compliance: !report || report.compliance.length === 0,
            streams: !report?.ffprobe,
            raw: !report,
            overview: !report,
          }}
        />
        <main className="flex-1 overflow-auto p-6">
          {error && (
            <div className="mb-4 rounded border border-fail/40 bg-fail/10 text-fail px-3 py-2 text-sm font-mono">
              {error}
            </div>
          )}
          {!report && !loading && !error && <EmptyState onOpen={open} />}
          {loading && (
            <div className="text-sm text-muted">Analyzing… this can take a moment on large masters.</div>
          )}
          {report && view === "overview" && <Overview report={report} />}
          {report && view === "streams" && <Streams report={report} />}
          {report && view === "atoms" && <AtomTree root={report.atoms} />}
          {report && view === "loudness" && <Loudness loudness={report.loudness} />}
          {report && view === "compliance" && <Compliance results={report.compliance} />}
          {report && view === "raw" && <RawProbe report={report} />}
        </main>
      </div>
    </div>
  );
}

function Header({
  tools,
  onOpen,
  loading,
}: {
  tools: ToolStatus | null;
  onOpen: () => void;
  loading: boolean;
}) {
  return (
    <header className="flex items-center gap-4 border-b border-line px-6 py-3 bg-panel/40">
      <div className="font-semibold tracking-tight text-zinc-100">
        Media<span className="text-accent">Inspect</span>
      </div>
      <button
        onClick={onOpen}
        disabled={loading}
        className="rounded bg-accent/15 text-accent border border-accent/40 px-3 py-1 text-sm hover:bg-accent/25 disabled:opacity-50"
      >
        {loading ? "Analyzing…" : "Open file"}
      </button>
      <div className="ml-auto flex items-center gap-3 text-xs font-mono">
        <ToolDot label="ffprobe" ok={tools?.ffprobe ?? false} path={tools?.ffprobe_path} />
        <ToolDot label="mediainfo" ok={tools?.mediainfo ?? false} path={tools?.mediainfo_path} />
      </div>
    </header>
  );
}

function ToolDot({ label, ok, path }: { label: string; ok: boolean; path?: string | null }) {
  return (
    <span
      title={path ?? `${label} not found on PATH`}
      className={`flex items-center gap-1 ${ok ? "text-pass" : "text-muted"}`}
    >
      <span
        className={`inline-block w-2 h-2 rounded-full ${ok ? "bg-pass" : "bg-muted"}`}
      />
      {label}
    </span>
  );
}

function EmptyState({ onOpen }: { onOpen: () => void }) {
  return (
    <div className="max-w-xl space-y-4">
      <h1 className="text-2xl font-semibold text-zinc-100">
        MediaInspect
      </h1>
      <p className="text-sm text-zinc-300 leading-relaxed">
        A hyper-detailed media analyzer for encoding/streaming engineers and
        archivists. Open any video, audio, or container file and see every parameter
        ffprobe and MediaInfo know about — cross-checked, organized, and audited
        against the specs your audience actually delivers under.
      </p>
      <p className="text-sm text-muted leading-relaxed">
        On open, MediaInspect runs ffprobe + MediaInfo, walks the ISOBMFF atom tree
        natively, measures EBU R128 loudness with true peak, and evaluates a starter
        rule set (Apple HLS, EBU R128 / ATSC A/85, Netflix delivery basics).
      </p>
      <button
        onClick={onOpen}
        className="rounded bg-accent/15 text-accent border border-accent/40 px-4 py-2 text-sm hover:bg-accent/25"
      >
        Open a media file
      </button>
    </div>
  );
}
