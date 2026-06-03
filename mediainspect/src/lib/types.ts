export type Verdict = "pass" | "warn" | "fail" | "notapplicable";

export interface RuleResult {
  spec: string;
  rule: string;
  verdict: Verdict;
  detail: string;
  evidence_view: string;
  citation: string;
}

export interface AtomNode {
  kind: string;
  offset: number;
  size: number;
  children: AtomNode[];
  fields: unknown;
}

export interface StreamBitrate {
  index: number;
  codec: string;
  kind: string;
  declared_kbps: number | null;
  avg_kbps: number | null;
  duration_seconds: number | null;
}

export interface BitrateTimeline {
  format_kbps: number | null;
  streams: StreamBitrate[];
}

export interface LoudnessReport {
  integrated_lufs: number;
  loudness_range_lu: number;
  true_peak_dbtp: number[];
  channels: number;
  sample_rate: number;
  seconds_measured: number;
}

export interface InspectReport {
  path: string;
  size_bytes: number;
  ffprobe: unknown | null;
  mediainfo: unknown | null;
  atoms: AtomNode | null;
  bitrate_timeline: BitrateTimeline | null;
  loudness: LoudnessReport | null;
  compliance: RuleResult[];
  warnings: string[];
}

export interface ToolStatus {
  ffprobe: boolean;
  mediainfo: boolean;
  ffprobe_path: string | null;
  mediainfo_path: string | null;
}
