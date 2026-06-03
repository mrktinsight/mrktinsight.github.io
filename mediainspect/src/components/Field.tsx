interface FieldProps {
  label: string;
  value: React.ReactNode;
  /** Where the value came from. Shown subtly so experts can trust the number. */
  source?: string;
  /** Spec citation shown in title attribute. */
  citation?: string;
  className?: string;
}

export function Field({ label, value, source, citation, className }: FieldProps) {
  return (
    <div
      className={`flex flex-col gap-0.5 py-1.5 ${className ?? ""}`}
      title={citation}
    >
      <div className="text-[11px] uppercase tracking-wide text-muted">
        {label}
        {source && <span className="ml-2 normal-case tracking-normal text-[10px] text-zinc-600">via {source}</span>}
      </div>
      <div className="font-mono text-sm text-zinc-100">{value}</div>
    </div>
  );
}
