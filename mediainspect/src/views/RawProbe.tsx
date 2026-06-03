import { useState } from "react";
import type { InspectReport } from "../lib/types";

interface RawProbeProps {
  report: InspectReport;
}

type Tab = "ffprobe" | "mediainfo" | "report";

export function RawProbe({ report }: RawProbeProps) {
  const [tab, setTab] = useState<Tab>("ffprobe");

  const data =
    tab === "ffprobe"
      ? report.ffprobe
      : tab === "mediainfo"
        ? report.mediainfo
        : report;

  return (
    <div className="flex flex-col gap-3">
      <p className="text-sm text-muted">
        The escape hatch: unmodified probe output. If a value in another view
        looks suspect, find its raw equivalent here.
      </p>
      <div className="flex gap-1 text-xs font-mono">
        {(["ffprobe", "mediainfo", "report"] as Tab[]).map((t) => (
          <button
            key={t}
            onClick={() => setTab(t)}
            className={`rounded px-3 py-1 border ${
              t === tab
                ? "border-accent text-accent bg-accent/10"
                : "border-line text-muted hover:bg-line/50"
            }`}
          >
            {t}
          </button>
        ))}
      </div>
      {data == null ? (
        <div className="text-sm text-muted">
          No output. {tab === "ffprobe" && "Install ffprobe or bundle it."}
          {tab === "mediainfo" && "Install MediaInfo or bundle it."}
        </div>
      ) : (
        <pre className="text-xs font-mono bg-panel rounded p-3 overflow-auto max-h-[70vh] whitespace-pre-wrap">
          {JSON.stringify(data, null, 2)}
        </pre>
      )}
    </div>
  );
}
