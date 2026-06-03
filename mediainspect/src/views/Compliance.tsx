import type { RuleResult, Verdict } from "../lib/types";

interface ComplianceProps {
  results: RuleResult[];
}

export function Compliance({ results }: ComplianceProps) {
  if (results.length === 0) {
    return <div className="text-sm text-muted">No compliance rules ran.</div>;
  }

  // Group by spec
  const bySpec = new Map<string, RuleResult[]>();
  for (const r of results) {
    const arr = bySpec.get(r.spec) ?? [];
    arr.push(r);
    bySpec.set(r.spec, arr);
  }

  return (
    <div className="flex flex-col gap-6">
      <p className="text-sm text-muted">
        Each rule is evaluated against the parsed report. Hover the citation chip
        for the spec reference. <span className="text-warn">Warn</span> means
        "look at this" — not necessarily a delivery blocker.
      </p>
      {[...bySpec.entries()].map(([spec, rules]) => (
        <section key={spec}>
          <h2 className="text-xs uppercase tracking-wide text-muted mb-2">{spec}</h2>
          <ul className="rounded border border-line divide-y divide-line">
            {rules.map((r, i) => (
              <li key={i} className="px-3 py-2 flex items-start gap-3">
                <VerdictPill verdict={r.verdict} />
                <div className="flex-1">
                  <div className="text-sm text-zinc-100">{r.rule}</div>
                  <div className="text-xs text-muted font-mono mt-0.5">{r.detail}</div>
                </div>
                <span
                  className="text-[11px] text-muted bg-panel border border-line rounded px-2 py-0.5 self-start"
                  title={r.citation}
                >
                  {r.citation}
                </span>
              </li>
            ))}
          </ul>
        </section>
      ))}
    </div>
  );
}

function VerdictPill({ verdict }: { verdict: Verdict }) {
  const map: Record<Verdict, { label: string; cls: string }> = {
    pass: { label: "pass", cls: "bg-pass/10 text-pass border-pass/30" },
    warn: { label: "warn", cls: "bg-warn/10 text-warn border-warn/30" },
    fail: { label: "fail", cls: "bg-fail/10 text-fail border-fail/30" },
    notapplicable: { label: "n/a", cls: "bg-na/10 text-na border-na/30" },
  };
  const v = map[verdict];
  return (
    <span
      className={`text-[10px] uppercase tracking-wide border rounded px-1.5 py-0.5 font-mono ${v.cls}`}
    >
      {v.label}
    </span>
  );
}
