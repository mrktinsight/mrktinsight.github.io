import { Table } from "../components/Table";
import type { InspectReport } from "../lib/types";

interface StreamsProps {
  report: InspectReport;
}

interface StreamRow {
  index: number;
  type: string;
  codec: string;
  profile: string;
  detail: string;
  bitrate: string;
}

export function Streams({ report }: StreamsProps) {
  const streams: any[] = (report.ffprobe as any)?.streams ?? [];

  const rows: StreamRow[] = streams.map((s) => ({
    index: s.index,
    type: s.codec_type ?? "—",
    codec: s.codec_long_name ?? s.codec_name ?? "—",
    profile: s.profile ?? "—",
    detail:
      s.codec_type === "video"
        ? `${s.width}×${s.height} ${s.pix_fmt} @ ${s.avg_frame_rate} (${s.color_primaries ?? "untagged"} / ${s.color_transfer ?? "untagged"})`
        : s.codec_type === "audio"
          ? `${s.sample_rate} Hz ${s.channels}ch ${s.channel_layout ?? ""} (${s.bits_per_raw_sample ?? s.bits_per_sample ?? "?"} bit)`
          : (s.tags?.language ? `lang=${s.tags.language}` : ""),
    bitrate: s.bit_rate ? `${(parseInt(s.bit_rate, 10) / 1000).toFixed(0)} kbps` : "—",
  }));

  return (
    <div className="flex flex-col gap-4">
      <p className="text-sm text-muted">
        Direct parse of <code>ffprobe -show_streams</code> output. Cross-check fields against
        the Raw probe view if you don't trust them.
      </p>
      <Table
        rows={rows}
        columns={[
          { header: "#", cell: (r) => r.index, className: "w-10 text-muted" },
          { header: "Type", cell: (r) => r.type, className: "w-20" },
          { header: "Codec", cell: (r) => r.codec },
          { header: "Profile", cell: (r) => r.profile, className: "w-40" },
          { header: "Detail", cell: (r) => r.detail },
          { header: "Bitrate", cell: (r) => r.bitrate, className: "w-28 text-right" },
        ]}
      />
    </div>
  );
}
