export type ViewKey =
  | "overview"
  | "streams"
  | "atoms"
  | "loudness"
  | "compliance"
  | "raw";

interface SidebarProps {
  current: ViewKey;
  onSelect: (v: ViewKey) => void;
  disabled?: Partial<Record<ViewKey, boolean>>;
}

const ITEMS: { key: ViewKey; label: string; hint: string }[] = [
  { key: "overview", label: "Overview", hint: "Headline + scorecard" },
  { key: "streams", label: "Streams", hint: "Per-stream parameters" },
  { key: "atoms", label: "Atom tree", hint: "ISOBMFF/MOV boxes" },
  { key: "loudness", label: "Loudness", hint: "EBU R128, true peak" },
  { key: "compliance", label: "Compliance", hint: "Spec rules" },
  { key: "raw", label: "Raw probe", hint: "Source-of-truth output" },
];

export function Sidebar({ current, onSelect, disabled }: SidebarProps) {
  return (
    <nav className="w-52 shrink-0 border-r border-line bg-panel/40 p-3">
      <ul className="flex flex-col gap-1">
        {ITEMS.map((item) => {
          const isDisabled = disabled?.[item.key];
          const isCurrent = item.key === current;
          return (
            <li key={item.key}>
              <button
                disabled={isDisabled}
                onClick={() => onSelect(item.key)}
                className={[
                  "w-full text-left rounded px-2 py-1.5 transition",
                  isCurrent ? "bg-accent/10 text-accent" : "text-zinc-300 hover:bg-line",
                  isDisabled ? "opacity-40 cursor-not-allowed" : "",
                ].join(" ")}
              >
                <div className="text-sm font-medium">{item.label}</div>
                <div className="text-[11px] text-muted">{item.hint}</div>
              </button>
            </li>
          );
        })}
      </ul>
    </nav>
  );
}
