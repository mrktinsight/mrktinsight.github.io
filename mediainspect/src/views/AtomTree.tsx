import { useState } from "react";
import { bytes, hexOffset } from "../lib/format";
import type { AtomNode } from "../lib/types";

interface AtomTreeProps {
  root: AtomNode | null;
}

export function AtomTree({ root }: AtomTreeProps) {
  if (!root) {
    return (
      <div className="text-sm text-muted">
        No ISOBMFF structure found. This view supports MP4, MOV, M4A, fragmented MP4
        and similar containers. For Matroska, MPEG-TS, MXF, etc., the parser is
        on the roadmap.
      </div>
    );
  }

  return (
    <div className="font-mono text-sm">
      <div className="text-muted mb-3 text-xs">
        Click a row to drill in. Each box shows its FourCC, file offset, total size,
        and decoded fields where MediaInspect knows the layout. Container payloads
        recurse; leaves print decoded fields inline.
      </div>
      <ul>
        {root.children.map((child, i) => (
          <Node key={i} node={child} depth={0} />
        ))}
      </ul>
    </div>
  );
}

function Node({ node, depth }: { node: AtomNode; depth: number }) {
  const [open, setOpen] = useState(depth < 1);
  const isContainer = node.children.length > 0;
  const hasFields =
    node.fields !== null &&
    typeof node.fields === "object" &&
    Object.keys(node.fields as object).length > 0;

  return (
    <li>
      <div
        className={`flex items-baseline gap-3 py-0.5 cursor-pointer hover:bg-line/40 rounded px-1 ${
          isContainer ? "text-zinc-100" : "text-zinc-300"
        }`}
        onClick={() => setOpen((v) => !v)}
        style={{ paddingLeft: `${depth * 16}px` }}
      >
        <span className="text-muted w-4">{isContainer ? (open ? "▾" : "▸") : "·"}</span>
        <span className="font-semibold w-24">{node.kind}</span>
        <span className="text-muted text-xs w-28">{hexOffset(node.offset)}</span>
        <span className="text-muted text-xs w-20 text-right">{bytes(node.size)}</span>
        {hasFields && !open && (
          <span className="text-xs text-zinc-500 truncate">
            {summarize(node.fields)}
          </span>
        )}
      </div>
      {open && hasFields && (
        <pre className="ml-12 my-1 px-3 py-2 bg-panel rounded text-xs text-zinc-300 whitespace-pre-wrap break-all">
          {JSON.stringify(node.fields, null, 2)}
        </pre>
      )}
      {open && isContainer && (
        <ul>
          {node.children.map((child, i) => (
            <Node key={i} node={child} depth={depth + 1} />
          ))}
        </ul>
      )}
    </li>
  );
}

function summarize(fields: unknown): string {
  if (typeof fields !== "object" || fields === null) return "";
  const entries = Object.entries(fields as Record<string, unknown>).slice(0, 3);
  return entries
    .map(([k, v]) => `${k}=${typeof v === "object" ? "…" : String(v)}`)
    .join("  ");
}
