//! EBU R128 loudness measurement.
//!
//! Wraps the `ebur128` crate. We decode the file's first audio track via
//! Symphonia (pure Rust, no FFmpeg dependency), push samples through the
//! meter in 100 ms blocks, then report integrated/short-term/momentary
//! loudness, LRA, and true peak (dBTP) per channel.
//!
//! Returns `Ok(None)` if there's no audio track we can decode — that's
//! not an error condition.

use std::path::Path;

use ebur128::{EbuR128, Mode};
use serde::Serialize;
use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use crate::error::AppError;

#[derive(Debug, Serialize)]
pub struct LoudnessReport {
    pub integrated_lufs: f64,
    pub loudness_range_lu: f64,
    pub true_peak_dbtp: Vec<f64>,
    pub channels: u32,
    pub sample_rate: u32,
    pub seconds_measured: f64,
}

pub fn measure(path: &Path) -> Result<Option<LoudnessReport>, AppError> {
    let file = std::fs::File::open(path).map_err(|e| AppError::Io(format!("open: {e}")))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        hint.with_extension(ext);
    }

    let probed = match symphonia::default::get_probe().format(
        &hint,
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    ) {
        Ok(p) => p,
        Err(SymphoniaError::Unsupported(_)) => return Ok(None),
        Err(e) => return Err(AppError::Parse(format!("symphonia probe: {e}"))),
    };

    let mut format = probed.format;
    let track = match format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
    {
        Some(t) => t.clone(),
        None => return Ok(None),
    };

    let codec_params = &track.codec_params;
    let channels = match codec_params.channels {
        Some(c) => c.count() as u32,
        None => return Ok(None),
    };
    let sample_rate = match codec_params.sample_rate {
        Some(r) => r,
        None => return Ok(None),
    };

    let track_id = track.id;
    let mut decoder = symphonia::default::get_codecs()
        .make(codec_params, &DecoderOptions::default())
        .map_err(|e| AppError::Parse(format!("symphonia decoder: {e}")))?;

    let mut meter = EbuR128::new(
        channels,
        sample_rate,
        Mode::I | Mode::LRA | Mode::TRUE_PEAK,
    )
    .map_err(|e| AppError::Parse(format!("ebur128: {e}")))?;

    let mut total_frames: u64 = 0;

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(SymphoniaError::IoError(e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(SymphoniaError::ResetRequired) => break,
            Err(e) => return Err(AppError::Parse(format!("packet: {e}"))),
        };
        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(SymphoniaError::IoError(_)) => break,
            Err(e) => return Err(AppError::Parse(format!("decode: {e}"))),
        };

        let frames = decoded.frames() as u64;
        total_frames += frames;
        push_planar_f32(&mut meter, decoded)?;
    }

    if total_frames == 0 {
        return Ok(None);
    }

    let integrated = meter
        .loudness_global()
        .map_err(|e| AppError::Parse(format!("ebur128 integrated: {e}")))?;
    let lra = meter
        .loudness_range()
        .map_err(|e| AppError::Parse(format!("ebur128 lra: {e}")))?;
    let mut tp = Vec::with_capacity(channels as usize);
    for ch in 0..channels {
        let v = meter
            .true_peak(ch)
            .map_err(|e| AppError::Parse(format!("ebur128 tp: {e}")))?;
        // ebur128 returns true peak as linear; convert to dBTP.
        let dbtp = if v > 0.0 { 20.0 * v.log10() } else { f64::NEG_INFINITY };
        tp.push(dbtp);
    }

    Ok(Some(LoudnessReport {
        integrated_lufs: integrated,
        loudness_range_lu: lra,
        true_peak_dbtp: tp,
        channels,
        sample_rate,
        seconds_measured: total_frames as f64 / sample_rate as f64,
    }))
}

fn push_planar_f32(meter: &mut EbuR128, decoded: AudioBufferRef) -> Result<(), AppError> {
    // Convert any sample format Symphonia decoded to f32 and interleave for the meter.
    let spec = *decoded.spec();
    let n_channels = spec.channels.count();
    let n_frames = decoded.frames();
    let mut interleaved: Vec<f32> = Vec::with_capacity(n_frames * n_channels);
    interleaved.resize(n_frames * n_channels, 0.0);

    macro_rules! pull {
        ($buf:expr, $convert:expr) => {{
            for ch in 0..n_channels {
                let plane = $buf.chan(ch);
                for (i, s) in plane.iter().enumerate() {
                    interleaved[i * n_channels + ch] = $convert(*s);
                }
            }
        }};
    }

    match decoded {
        AudioBufferRef::F32(b) => pull!(b, |s: f32| s),
        AudioBufferRef::F64(b) => pull!(b, |s: f64| s as f32),
        AudioBufferRef::S32(b) => pull!(b, |s: i32| s as f32 / i32::MAX as f32),
        AudioBufferRef::S16(b) => pull!(b, |s: i16| s as f32 / i16::MAX as f32),
        AudioBufferRef::S24(b) => pull!(b, |s: symphonia::core::sample::i24| {
            s.inner() as f32 / 8_388_607.0
        }),
        AudioBufferRef::U8(b) => pull!(b, |s: u8| (s as f32 - 128.0) / 127.0),
        AudioBufferRef::U16(b) => pull!(b, |s: u16| (s as f32 - 32768.0) / 32767.0),
        AudioBufferRef::U24(b) => pull!(b, |s: symphonia::core::sample::u24| {
            (s.inner() as f32 - 8_388_608.0) / 8_388_607.0
        }),
        AudioBufferRef::U32(b) => pull!(b, |s: u32| (s as f32 - 2_147_483_648.0) / 2_147_483_647.0),
        AudioBufferRef::S8(b) => pull!(b, |s: i8| s as f32 / 127.0),
    }

    meter
        .add_frames_f32(&interleaved)
        .map_err(|e| AppError::Parse(format!("ebur128 push: {e}")))?;
    Ok(())
}
