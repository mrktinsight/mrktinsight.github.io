import { Field } from "../components/Field";
import { bytes, kbps, lufs, seconds } from "../lib/format";
import type { InspectReport, Verdict } from "../lib/types";

interface OverviewProps {
  report: InspectReport;
}

export function Overview({ report }: OverviewProps) {
  const fmt = (report.ffprobe as any)?.format ?? {};
  const fmtName = fmt.format_long_name ?? fmt.format_name ?? "—";
  const duration = parseFloat(fmt.duration ?? "0") || null;
  const fmtBitrate = report.bitrate_timeline?.format_kbps ?? null;
  const streams = (report.ffprobe as any)?.streams ?? [];
  const videoStream = streams.find((s: any) => s.codec_type === "video");
  const audioStream = streams.find((s: any) => s.codec_type === "audio");

  const passes = report.compliance.filter((r) => r.verdict === "pass").length;
  const warns = report.compliance.filter((r) => r.verdict === "warn").length;
  const fails = report.compliance.filter((r) => r.verdict === "fail").length;

  return (
    <div className="flex flex-col gap-6">
      <section>
        <h2 className="text-xs uppercase tracking-wide text-muted mb-2">File</h2>
        <div className="text-sm font-mono break-all text-zinc-200">{report.path}</div>
        <div className="grid grid-cols-4 gap-x-8 gap-y-1 mt-3">
          <Field label="Container" value={fmtName} source="ffprobe" />
          <Field label="Size" value={bytes(report.size_bytes)} />
          <Field label="Duration" value={seconds(duration)} source="ffprobe" />
          <Field label="Format bitrate" value={kbps(fmtBitrate)} source="ffprobe" />
        </div>
      </section>

      {videoStream && (
        <section>
          <h2 className="text-xs uppercase tracking-wide text-muted mb-2">Video</h2>
          <div className="grid grid-cols-4 gap-x-8 gap-y-1">
            <Field label="Codec" value={videoStream.codec_long_name ?? videoStream.codec_name} source="ffprobe" />
            <Field label="Profile" value={videoStream.profile ?? "—"} source="ffprobe" />
            <Field label="Pixel format" value={videoStream.pix_fmt ?? "—"} source="ffprobe" />
            <Field
              label="Resolution"
              value={
                videoStream.width && videoStream.height
                  ? `${videoStream.width} × ${videoStream.height}`
                  : "—"
              }
              source="ffprobe"
            />
            <Field label="FPS" value={videoStream.avg_frame_rate ?? "—"} source="ffprobe" />
            <Field label="Bit rate" value={kbps(parseInt(videoStream.bit_rate ?? "0", 10) / 1000)} source="ffprobe" />
            <Field
              label="Color primaries"
              value={videoStream.color_primaries ?? "untagged"}
              source="ffprobe"
              citation="ISO 23001-8 / ITU-T H.273"
            />
            <Field
              label="Transfer"
              value={videoStream.color_transfer ?? "untagged"}
              source="ffprobe"
              citation="ISO 23001-8 / ITU-T H.273"
            />
          </div>
        </section>
      )}

      {audioStream && (
        <section>
          <h2 className="text-xs uppercase tracking-wide text-muted mb-2">Audio</h2>
          <div className="grid grid-cols-4 gap-x-8 gap-y-1">
            <Field label="Codec" value={audioStream.codec_long_name ?? audioStream.codec_name} source="ffprobe" />
            <Field label="Sample rate" value={`${audioStream.sample_rate ?? "—"} Hz`} source="ffprobe" />
            <Field label="Channels" value={audioStream.channels ?? "—"} source="ffprobe" />
            <Field label="Channel layout" value={audioStream.channel_layout ?? "—"} source="ffprobe" />
            <Field
              label="Bit depth (declared)"
              value={audioStream.bits_per_raw_sample ?? audioStream.bits_per_sample ?? "—"}
              source="ffprobe"
            />
            <Field
              label="Integrated loudness"
              value={lufs(report.loudness?.integrated_lufs)}
              source="ebur128"
              citation="ITU-R BS.1770-4 / EBU R128"
            />
            <Field
              label="True peak (max)"
              value={
                report.loudness?.true_peak_dbtp.length
                  ? `${Math.max(...report.loudness.true_peak_dbtp).toFixed(2)} dBTP`
                  : "—"
              }
              source="ebur128"
              citation="EBU R128 s2 §3.3"
            />
            <Field
              label="Loudness range"
              value={
                report.loudness ? `${report.loudness.loudness_range_lu.toFixed(2)} LU` : "—"
              }
              source="ebur128"
            />
          </div>
        </section>
      )}

      <section>
        <h2 className="text-xs uppercase tracking-wide text-muted mb-2">Compliance scorecard</h2>
        <div className="flex gap-3">
          <Badge verdict="pass" label="Pass" count={passes} />
          <Badge verdict="warn" label="Warn" count={warns} />
          <Badge verdict="fail" label="Fail" count={fails} />
        </div>
      </section>

      {report.warnings.length > 0 && (
        <section>
          <h2 className="text-xs uppercase tracking-wide text-muted mb-2">Warnings</h2>
          <ul className="text-sm font-mono text-warn space-y-1">
            {report.warnings.map((w, i) => (
              <li key={i}>{w}</li>
            ))}
          </ul>
        </section>
      )}
    </div>
  );
}

function Badge({
  verdict,
  label,
  count,
}: {
  verdict: Verdict;
  label: string;
  count: number;
}) {
  const colorClass =
    verdict === "pass"
      ? "border-pass/40 text-pass"
      : verdict === "warn"
        ? "border-warn/40 text-warn"
        : "border-fail/40 text-fail";
  return (
    <div className={`rounded border ${colorClass} px-3 py-2 font-mono text-sm`}>
      <div className="text-[11px] uppercase tracking-wide opacity-80">{label}</div>
      <div className="text-xl">{count}</div>
    </div>
  );
}
