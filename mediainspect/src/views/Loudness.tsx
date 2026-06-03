import { Field } from "../components/Field";
import { dbtp, lufs } from "../lib/format";
import type { LoudnessReport } from "../lib/types";

interface LoudnessProps {
  loudness: LoudnessReport | null;
}

export function Loudness({ loudness }: LoudnessProps) {
  if (!loudness) {
    return (
      <div className="text-sm text-muted">
        No decodable audio. MediaInspect measures EBU R128 loudness on any audio
        track Symphonia can decode (FLAC, ALAC, AAC, MP3, PCM, Vorbis, Opus…).
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-6">
      <section>
        <h2 className="text-xs uppercase tracking-wide text-muted mb-2">
          EBU R128 / ITU-R BS.1770-4
        </h2>
        <div className="grid grid-cols-3 gap-x-8 gap-y-1">
          <Field
            label="Integrated"
            value={lufs(loudness.integrated_lufs)}
            source="ebur128"
            citation="ITU-R BS.1770-4 §3"
          />
          <Field
            label="Loudness range (LRA)"
            value={`${loudness.loudness_range_lu.toFixed(2)} LU`}
            source="ebur128"
            citation="EBU Tech 3342"
          />
          <Field
            label="Seconds measured"
            value={loudness.seconds_measured.toFixed(2)}
            source="ebur128"
          />
        </div>
      </section>

      <section>
        <h2 className="text-xs uppercase tracking-wide text-muted mb-2">
          True peak per channel
        </h2>
        <div className="grid grid-cols-4 gap-x-8 gap-y-1">
          {loudness.true_peak_dbtp.map((p, i) => (
            <Field
              key={i}
              label={`Channel ${i + 1}`}
              value={dbtp(p)}
              source="ebur128"
              citation="EBU R128 s2 §3.3 (oversampled true peak)"
            />
          ))}
        </div>
      </section>

      <section>
        <h2 className="text-xs uppercase tracking-wide text-muted mb-2">
          Quick reference
        </h2>
        <div className="text-sm text-muted space-y-1">
          <div>EBU R128 broadcast target: <span className="text-zinc-200">-23 LUFS ±1</span>, true peak ≤ -1 dBTP</div>
          <div>ATSC A/85 (US broadcast): <span className="text-zinc-200">-24 LKFS ±2</span></div>
          <div>Spotify: <span className="text-zinc-200">-14 LUFS</span>, Apple Music: <span className="text-zinc-200">-16 LUFS</span>, YouTube: <span className="text-zinc-200">-14 LUFS</span></div>
        </div>
      </section>
    </div>
  );
}
