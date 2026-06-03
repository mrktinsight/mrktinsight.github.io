# MediaInspect

A hyper-detailed media analyzer for encoding/streaming engineers and
archivists/forensic-QC. Standalone desktop app. **ffprobe++**: everything
`ffprobe` and `MediaInfo` give you, parsed, cross-checked, visualized, and
audited against the specs your audience actually delivers to.

> Scope: this directory is a self-contained Tauri 2 project under the
> MRKT Insight repo, on branch `claude/media-analysis-app-nDkmH`. It is
> independent of the marketing site at the repo root.

## What it does today (MVP scaffold)

- Open a media file from disk.
- Spawn bundled `ffprobe` + `MediaInfo` (or use system binaries during dev),
  parse their JSON, present every field labeled and cross-linked.
- Walk MP4/MOV ISOBMFF atom trees with byte offsets via a pure-Rust
  streaming box walker (no `ffprobe` required) — surfaces `ftyp`, `mvhd`,
  `tkhd`, `mdhd`, `hdlr`, and **`elst` empty-edit detection** so QC catches
  edit lists that hide content.
- Compliance rules engine with starter rule sets for **Apple HLS Authoring
  Spec**, **EBU R128 / ATSC A/85** loudness, and a **Netflix delivery**
  subset. Each rule cites the spec and links back to the measurement.
- Bitrate-over-time analysis derived from packet sizes/durations.
- EBU R128 loudness wrapper (integrated, momentary, short-term, LRA, true
  peak) via the `ebur128` crate.
- A "Raw Probe" view that shows the unmodified `ffprobe`/`MediaInfo` output
  for trust-building.

## What's intentionally deferred

VMAF/SSIM, per-frame QP, AV1 OBU parser, scene detection, spectrogram canvas,
compare-two-files diff view, Windows/macOS code signing/notarization,
DASH/HLS network manifest fetching. The architecture has hook points for all
of these; see the plan.

## Architecture

```
Tauri 2 shell
├── Rust core (src-tauri/src)
│   ├── probe/        native parsers (ISOBMFF; Matroska/TS to come)
│   ├── sidecar/      ffprobe + MediaInfo wrappers
│   ├── analysis/     bitrate, R128 loudness, …
│   └── compliance/   spec rule engine (Apple HLS, EBU R128, Netflix)
└── React + TS frontend (src)
    ├── views/        Overview, Streams, AtomTree, Compliance, RawProbe
    └── components/   Field (labeled+spec-cited), Table, Sidebar
```

Every measurement displayed in the UI is tagged with its **source**
(`mp4parse` / `ffprobe` / `MediaInfo` / `ebur128`) and, where applicable,
a **spec citation** on hover. This is the trust-building hook for experts.

## Running it

You need:
- Rust 1.77+ (`rustup`)
- Node 20+ and pnpm 10+
- Tauri 2 CLI prerequisites for your OS — see
  https://tauri.app/start/prerequisites/
- `ffprobe` and `mediainfo` on your `PATH` for the sidecar features
  (production builds will bundle them; the dev shell looks up `PATH` first)

```bash
cd mediainspect
pnpm install
pnpm tauri dev
```

For the Rust-only smoke test (no UI), the library crate has unit tests:

```bash
cd mediainspect/src-tauri
cargo test
```

## Why this exists

`ffprobe` is the de-facto inspection tool but it's a CLI that dumps JSON,
and `MediaInfo` GUI presents a flat key/value list. Neither shows you GOP
structure, IDR alignment across HLS renditions, declared-vs-measured HDR
metadata, edit-list consequences, encoder fingerprints, or loudness
compliance against the spec a streaming platform actually ships under.
MediaInspect is the GUI that does.

## License

The bundled FFmpeg and MediaInfoLib binaries are LGPL / BSD-adjacent.
Track upstream licenses in `THIRD_PARTY.md` (to be added at first release
build) and ship the license texts alongside binaries. The MediaInspect
code itself is © MRKT Insight, LLC.
