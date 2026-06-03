//! ISOBMFF (MP4/MOV/fMP4) atom-tree walker.
//!
//! This is the forensic/QC differentiator: ffprobe summarizes streams but
//! hides container structure. Here we walk every box, record its offset,
//! size, and (for container boxes) recurse into children. The tree shape
//! is what the UI's Atom Tree view renders verbatim.
//!
//! Memory-safe on huge files: we never read more than each box's header
//! plus, for known leaf boxes, just enough payload to extract the fields
//! we surface. Container payloads are streamed past.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use serde::Serialize;

use crate::error::AppError;

/// Known container box types — we descend into their payloads.
const CONTAINERS: &[&[u8; 4]] = &[
    b"moov", b"trak", b"mdia", b"minf", b"dinf", b"stbl", b"edts", b"udta",
    b"mvex", b"moof", b"traf", b"mfra", b"sinf", b"schi", b"tref", b"iprp",
    b"ipco", b"ilst", b"\xa9too", b"\xa9enc",
];

/// Containers that carry a 4-byte version+flags header before children.
const FULL_CONTAINERS: &[&[u8; 4]] = &[b"meta"];

#[derive(Debug, Serialize, Clone)]
pub struct AtomNode {
    /// FourCC, e.g. "moov".
    pub kind: String,
    /// Absolute byte offset of the box header in the file.
    pub offset: u64,
    /// Total box size (header + payload). 0 means "to EOF".
    pub size: u64,
    /// For containers, child boxes in file order.
    pub children: Vec<AtomNode>,
    /// Decoded fields for known leaf boxes (e.g., `ftyp` -> {brand, version, compatible}).
    pub fields: serde_json::Value,
}

pub fn walk(path: &Path) -> Result<Option<AtomNode>, AppError> {
    let mut f = File::open(path).map_err(|e| AppError::Io(format!("open: {e}")))?;
    let file_len = f
        .metadata()
        .map_err(|e| AppError::Io(format!("metadata: {e}")))?
        .len();

    // Quick sniff: a real ISOBMFF file starts with ftyp (or skip/free/etc.
    // followed by ftyp). If first 4 bytes after the size aren't a known
    // box type, treat as "not ISOBMFF".
    let mut head = [0u8; 16];
    let n = f
        .read(&mut head)
        .map_err(|e| AppError::Io(format!("sniff: {e}")))?;
    if n < 8 {
        return Ok(None);
    }
    if !looks_like_isobmff(&head[..n]) {
        return Ok(None);
    }
    f.seek(SeekFrom::Start(0))
        .map_err(|e| AppError::Io(format!("rewind: {e}")))?;

    let children = walk_range(&mut f, 0, file_len, 0)?;

    Ok(Some(AtomNode {
        kind: "file".into(),
        offset: 0,
        size: file_len,
        children,
        fields: serde_json::Value::Null,
    }))
}

fn looks_like_isobmff(buf: &[u8]) -> bool {
    if buf.len() < 8 {
        return false;
    }
    let kind = &buf[4..8];
    matches!(
        kind,
        b"ftyp" | b"styp" | b"skip" | b"free" | b"wide" | b"moov" | b"mdat"
    )
}

fn walk_range<R: Read + Seek>(
    f: &mut R,
    start: u64,
    end: u64,
    depth: u32,
) -> Result<Vec<AtomNode>, AppError> {
    // Hard cap on recursion depth to defend against pathological files.
    if depth > 16 {
        return Ok(Vec::new());
    }

    let mut out = Vec::new();
    let mut pos = start;
    f.seek(SeekFrom::Start(pos))
        .map_err(|e| AppError::Io(format!("seek: {e}")))?;

    while pos + 8 <= end {
        let mut hdr = [0u8; 8];
        if f.read_exact(&mut hdr).is_err() {
            break;
        }
        let mut box_size = u32::from_be_bytes([hdr[0], hdr[1], hdr[2], hdr[3]]) as u64;
        let kind_bytes: [u8; 4] = [hdr[4], hdr[5], hdr[6], hdr[7]];
        let mut header_len: u64 = 8;

        if box_size == 1 {
            let mut large = [0u8; 8];
            f.read_exact(&mut large)
                .map_err(|e| AppError::Io(format!("largesize: {e}")))?;
            box_size = u64::from_be_bytes(large);
            header_len = 16;
        } else if box_size == 0 {
            box_size = end - pos;
        }

        if box_size < header_len || pos + box_size > end {
            // Malformed — record what we can and stop.
            out.push(AtomNode {
                kind: format!("{}!", fourcc(&kind_bytes)),
                offset: pos,
                size: box_size,
                children: Vec::new(),
                fields: serde_json::json!({"error": "size exceeds parent"}),
            });
            break;
        }

        let payload_start = pos + header_len;
        let payload_end = pos + box_size;

        let is_container = CONTAINERS.iter().any(|c| **c == kind_bytes);
        let is_full_container = FULL_CONTAINERS.iter().any(|c| **c == kind_bytes);

        let (children, fields) = if is_container {
            let child_start = payload_start;
            let children = walk_range(f, child_start, payload_end, depth + 1)?;
            (children, serde_json::Value::Null)
        } else if is_full_container {
            // Skip 4-byte version+flags header, then recurse.
            let child_start = payload_start + 4;
            if child_start <= payload_end {
                let children = walk_range(f, child_start, payload_end, depth + 1)?;
                (children, serde_json::Value::Null)
            } else {
                (Vec::new(), serde_json::Value::Null)
            }
        } else {
            let fields = decode_leaf(f, &kind_bytes, payload_start, payload_end)
                .unwrap_or(serde_json::Value::Null);
            (Vec::new(), fields)
        };

        out.push(AtomNode {
            kind: fourcc(&kind_bytes),
            offset: pos,
            size: box_size,
            children,
            fields,
        });

        pos = payload_end;
        f.seek(SeekFrom::Start(pos))
            .map_err(|e| AppError::Io(format!("seek end: {e}")))?;
    }

    Ok(out)
}

fn fourcc(b: &[u8; 4]) -> String {
    let mut s = String::with_capacity(4);
    for &c in b {
        if c >= 0x20 && c < 0x7F {
            s.push(c as char);
        } else {
            s.push_str(&format!("\\x{:02x}", c));
        }
    }
    s
}

/// Decode a handful of high-value leaf boxes. This is intentionally narrow
/// for MVP — the goal is to surface fields experts actually look at, not
/// re-implement every spec.
fn decode_leaf<R: Read + Seek>(
    f: &mut R,
    kind: &[u8; 4],
    start: u64,
    end: u64,
) -> Option<serde_json::Value> {
    let payload_len = end.saturating_sub(start);
    // Skip payloads larger than 4 KiB for leaves — we only want metadata.
    if payload_len == 0 || payload_len > 4096 {
        return None;
    }
    f.seek(SeekFrom::Start(start)).ok()?;
    let mut buf = vec![0u8; payload_len as usize];
    f.read_exact(&mut buf).ok()?;

    match kind {
        b"ftyp" | b"styp" => decode_ftyp(&buf),
        b"mvhd" => decode_mvhd(&buf),
        b"tkhd" => decode_tkhd(&buf),
        b"mdhd" => decode_mdhd(&buf),
        b"hdlr" => decode_hdlr(&buf),
        b"elst" => decode_elst(&buf),
        _ => None,
    }
}

fn read_u32(b: &[u8], o: usize) -> Option<u32> {
    Some(u32::from_be_bytes(b.get(o..o + 4)?.try_into().ok()?))
}
fn read_u64(b: &[u8], o: usize) -> Option<u64> {
    Some(u64::from_be_bytes(b.get(o..o + 8)?.try_into().ok()?))
}
fn fcc(b: &[u8], o: usize) -> Option<String> {
    let s: [u8; 4] = b.get(o..o + 4)?.try_into().ok()?;
    Some(fourcc(&s))
}

fn decode_ftyp(b: &[u8]) -> Option<serde_json::Value> {
    let major = fcc(b, 0)?;
    let minor = read_u32(b, 4)?;
    let mut compat = Vec::new();
    let mut i = 8;
    while i + 4 <= b.len() {
        if let Some(s) = fcc(b, i) {
            compat.push(s);
        }
        i += 4;
    }
    Some(serde_json::json!({
        "major_brand": major,
        "minor_version": minor,
        "compatible_brands": compat,
    }))
}

fn decode_mvhd(b: &[u8]) -> Option<serde_json::Value> {
    let version = *b.first()?;
    let (timescale, duration) = if version == 1 {
        // version=1: 8+8+4+8 → creation, modification, timescale, duration
        let timescale = read_u32(b, 4 + 8 + 8)?;
        let duration = read_u64(b, 4 + 8 + 8 + 4)?;
        (timescale, duration as f64)
    } else {
        // version=0: 4+4+4+4
        let timescale = read_u32(b, 4 + 4 + 4)?;
        let duration = read_u32(b, 4 + 4 + 4 + 4)? as f64;
        (timescale, duration)
    };
    let dur_seconds = if timescale > 0 { duration / timescale as f64 } else { 0.0 };
    Some(serde_json::json!({
        "version": version,
        "timescale": timescale,
        "duration_units": duration,
        "duration_seconds": dur_seconds,
    }))
}

fn decode_tkhd(b: &[u8]) -> Option<serde_json::Value> {
    let version = *b.first()?;
    let flags = u32::from_be_bytes([0, b[1], b[2], b[3]]);
    let enabled = (flags & 0x1) != 0;
    let in_movie = (flags & 0x2) != 0;
    let in_preview = (flags & 0x4) != 0;

    let track_id_off = if version == 1 { 4 + 8 + 8 } else { 4 + 4 + 4 };
    let track_id = read_u32(b, track_id_off)?;

    // Width/height are the last 8 bytes (16.16 fixed point).
    let n = b.len();
    if n < 8 {
        return None;
    }
    let w = read_u32(b, n - 8)? as f64 / 65536.0;
    let h = read_u32(b, n - 4)? as f64 / 65536.0;
    Some(serde_json::json!({
        "version": version,
        "track_id": track_id,
        "enabled": enabled,
        "in_movie": in_movie,
        "in_preview": in_preview,
        "width": w,
        "height": h,
    }))
}

fn decode_mdhd(b: &[u8]) -> Option<serde_json::Value> {
    let version = *b.first()?;
    let (timescale, duration, lang_off) = if version == 1 {
        let ts = read_u32(b, 4 + 8 + 8)?;
        let du = read_u64(b, 4 + 8 + 8 + 4)? as f64;
        (ts, du, 4 + 8 + 8 + 4 + 8)
    } else {
        let ts = read_u32(b, 4 + 4 + 4)?;
        let du = read_u32(b, 4 + 4 + 4 + 4)? as f64;
        (ts, du, 4 + 4 + 4 + 4 + 4)
    };
    let dur_seconds = if timescale > 0 { duration / timescale as f64 } else { 0.0 };
    // Language is a packed 5-bit-per-char ISO 639-2 code, offset+1 to skip pad.
    let lang = b
        .get(lang_off..lang_off + 2)
        .and_then(|s| Some(decode_iso639(u16::from_be_bytes([s[0], s[1]]))));
    Some(serde_json::json!({
        "version": version,
        "timescale": timescale,
        "duration_units": duration,
        "duration_seconds": dur_seconds,
        "language": lang,
    }))
}

fn decode_iso639(packed: u16) -> String {
    let a = (((packed >> 10) & 0x1F) as u8 + 0x60) as char;
    let b = (((packed >> 5) & 0x1F) as u8 + 0x60) as char;
    let c = ((packed & 0x1F) as u8 + 0x60) as char;
    format!("{a}{b}{c}")
}

fn decode_hdlr(b: &[u8]) -> Option<serde_json::Value> {
    // version+flags(4) + pre_defined(4) + handler_type(4) + reserved(12) + name(string)
    let handler_type = fcc(b, 8)?;
    let name_start = 24;
    let name = if b.len() > name_start {
        let bytes = &b[name_start..];
        // c-string terminated, with optional pascal-style len prefix on QuickTime.
        let nul = bytes.iter().position(|&c| c == 0).unwrap_or(bytes.len());
        String::from_utf8_lossy(&bytes[..nul]).to_string()
    } else {
        String::new()
    };
    Some(serde_json::json!({
        "handler_type": handler_type,
        "name": name,
    }))
}

fn decode_elst(b: &[u8]) -> Option<serde_json::Value> {
    let version = *b.first()?;
    let entry_count = read_u32(b, 4)?;
    let mut entries = Vec::new();
    let mut off = 8;
    let mut hides_content = false;
    for _ in 0..entry_count {
        let (segment_duration, media_time, off_next) = if version == 1 {
            let sd = read_u64(b, off)?;
            let mt = i64::from_be_bytes(b.get(off + 8..off + 16)?.try_into().ok()?);
            (sd, mt, off + 16)
        } else {
            let sd = read_u32(b, off)? as u64;
            let mt = i32::from_be_bytes(b.get(off + 4..off + 8)?.try_into().ok()?) as i64;
            (sd, mt, off + 8)
        };
        let media_rate = read_u32(b, off_next)? as f64 / 65536.0;
        off = off_next + 4;
        // media_time == -1 is an "empty edit" — content is hidden.
        if media_time == -1 {
            hides_content = true;
        }
        entries.push(serde_json::json!({
            "segment_duration": segment_duration,
            "media_time": media_time,
            "media_rate": media_rate,
            "empty_edit": media_time == -1,
        }));
    }
    Some(serde_json::json!({
        "version": version,
        "entries": entries,
        "hides_content": hides_content,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Build a minimal in-memory ISOBMFF: ftyp + moov{mvhd}.
    fn build_minimal_mp4() -> Vec<u8> {
        let mut out = Vec::new();
        // ftyp: size=20, type=ftyp, major=isom, minor=512, compat=isom
        out.extend_from_slice(&20u32.to_be_bytes());
        out.extend_from_slice(b"ftyp");
        out.extend_from_slice(b"isom");
        out.extend_from_slice(&512u32.to_be_bytes());
        out.extend_from_slice(b"isom");

        // mvhd payload (v0): version+flags(4) + ctime(4) + mtime(4) + timescale(4)
        //   + duration(4) + rate(4) + volume(2) + reserved(10) + matrix(36)
        //   + pre_defined(24) + next_track_id(4) = 100 bytes
        let mut mvhd = Vec::new();
        mvhd.extend_from_slice(&[0, 0, 0, 0]); // version+flags
        mvhd.extend_from_slice(&0u32.to_be_bytes()); // creation
        mvhd.extend_from_slice(&0u32.to_be_bytes()); // modification
        mvhd.extend_from_slice(&1000u32.to_be_bytes()); // timescale
        mvhd.extend_from_slice(&5000u32.to_be_bytes()); // duration (5s)
        mvhd.extend_from_slice(&0x00010000u32.to_be_bytes()); // rate 1.0
        mvhd.extend_from_slice(&0x0100u16.to_be_bytes()); // volume 1.0
        mvhd.extend_from_slice(&[0u8; 10]);
        mvhd.extend_from_slice(&[0u8; 36]);
        mvhd.extend_from_slice(&[0u8; 24]);
        mvhd.extend_from_slice(&2u32.to_be_bytes());

        let mvhd_size = 8 + mvhd.len() as u32;
        let moov_size = 8 + mvhd_size;
        out.extend_from_slice(&moov_size.to_be_bytes());
        out.extend_from_slice(b"moov");
        out.extend_from_slice(&mvhd_size.to_be_bytes());
        out.extend_from_slice(b"mvhd");
        out.extend_from_slice(&mvhd);
        out
    }

    #[test]
    fn walks_minimal_mp4() {
        let bytes = build_minimal_mp4();
        let mut cur = Cursor::new(&bytes);
        let children = walk_range(&mut cur, 0, bytes.len() as u64, 0).unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].kind, "ftyp");
        assert_eq!(children[1].kind, "moov");
        assert_eq!(children[1].children.len(), 1);
        assert_eq!(children[1].children[0].kind, "mvhd");
        let fields = &children[1].children[0].fields;
        assert_eq!(fields["timescale"], 1000);
        assert!((fields["duration_seconds"].as_f64().unwrap() - 5.0).abs() < 1e-6);
    }

    #[test]
    fn detects_empty_edit() {
        // build elst with one entry: segment_duration=1000, media_time=-1 (empty)
        let mut elst = Vec::new();
        elst.extend_from_slice(&[0u8, 0, 0, 0]); // version+flags
        elst.extend_from_slice(&1u32.to_be_bytes()); // entry_count
        elst.extend_from_slice(&1000u32.to_be_bytes()); // segment_duration
        elst.extend_from_slice(&(-1i32).to_be_bytes()); // media_time = -1
        elst.extend_from_slice(&0x00010000u32.to_be_bytes()); // media_rate 1.0

        let decoded = decode_elst(&elst).expect("decoded");
        assert_eq!(decoded["hides_content"], true);
        assert_eq!(decoded["entries"][0]["empty_edit"], true);
    }
}
