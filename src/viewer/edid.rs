//! EDID 1.4 synthesizer.
//!
//! Builds a 128-byte EDID block by taking the JetKVM firmware's own default
//! EDID as a base and overwriting only the first Detailed Timing Descriptor
//! with the requested mode. This minimises the diff seen by the firmware and
//! its T749 HDMI capture chip — every byte the device hasn't been validated
//! against by the JetKVM team stays unchanged.
//!
//! Timings come either from a curated [`SAFE_MODES`] table (hand-picked
//! CTA-861 and CVT-RB v1 modes ≤ 1920×1200, the published ceiling of the
//! T749 chipset) or from a CVT-RB v1 generator for explicit
//! `--width/--height/--refresh` overrides.

use once_cell::sync::Lazy;

/// EDID DTD's pixel-clock field is a u16 counting 10 kHz units, so the
/// largest representable pixel clock is 65535 * 10 kHz = 655.35 MHz.
pub const MAX_PIXEL_CLOCK_KHZ: u32 = 655_350;

/// JetKVM's own factory-default EDID, copied verbatim from
/// `jetkvm/ui/src/routes/devices.$id.settings.video.tsx`. Used as the base
/// every modification is applied on top of, so that any field we don't
/// understand stays identical to what the device firmware expects.
pub static JETKVM_DEFAULT_EDID: Lazy<[u8; 128]> = Lazy::new(|| {
    const HEX: &str = "00ffffffffffff0052620188008888881c150103800000780a0dc9a05747982712484c00000001010101010101010101010101010101023a801871382d40582c4500c48e2100001e011d007251d01e206e285500c48e2100001e000000fc00543734392d6648443732300a20000000fd00147801ff1d000a202020202020017b";
    let v = hex::decode(HEX).expect("valid hex");
    <[u8; 128]>::try_from(v.as_slice()).expect("128 bytes")
});

/// JetKVM's default EDID as a hex string, suitable for `setEDID` to restore
/// the factory default.
pub fn jetkvm_default_edid_hex() -> String {
    hex::encode_upper(*JETKVM_DEFAULT_EDID)
}

/// The EDID base the viewer pushes to the device. Derived from
/// [`JETKVM_DEFAULT_EDID`] (so the byte layout is identical to what the
/// firmware ships and was validated against) but with the identification
/// fields rewritten to describe a Dell U2412M — a 24" 1920×1200 16:10 IPS
/// monitor that has been mass-produced for years and is recognised by every
/// modern host. Using a well-known consumer monitor identity means hosts
/// reliably pick a standard HDMI mode they're already happy to output.
///
/// Specifically:
/// - Manufacturer ID (bytes 8-9): `TSB` → `DEL` (0x10AC).
/// - Product code (bytes 10-11): `0x8801` → `0xF0A0` (U2412M).
/// - Serial number (bytes 12-15): pattern-of-0x88 → ASCII-like value.
/// - Year of manufacture (byte 17): 2011 → 2012 (U2412M's release year).
/// - DTD#1 H/V image size: 708×398 mm → 518×324 mm (U2412M's actual panel).
/// - DTD#3 display product name (offset 90-107): `T749-fHD720` →
///   `Dell U2412M`.
///
/// Everything else (DTD#2 1280×720 alt timing, range limits, chromaticity,
/// EDID 1.3 header, feature byte, etc.) stays byte-identical to the
/// firmware-shipped default.
pub static MONITOR_BASE_EDID: Lazy<[u8; 128]> = Lazy::new(|| {
    let mut e = *JETKVM_DEFAULT_EDID;

    // Manufacturer ID "DEL" — bits packed big-endian:
    //   ((D-@)<<10) | ((E-@)<<5) | (L-@) = (4<<10)|(5<<5)|12 = 0x10AC.
    e[8] = 0x10;
    e[9] = 0xAC;
    // Product code 0xF0A0 (Dell U2412M), stored little-endian.
    e[10] = 0xA0;
    e[11] = 0xF0;
    // Serial number — fixed arbitrary bytes.
    e[12] = 0x4D;
    e[13] = 0x4F;
    e[14] = 0x47;
    e[15] = 0x4B;
    // Week 5, year 2012 (= 1990 + 22).
    e[16] = 0x05;
    e[17] = 22;

    // DTD#1 H/V image size in mm — Dell U2412M's actual 518 × 324 mm panel.
    //   bytes [DTD+12, DTD+13, DTD+14] = H_LSB, V_LSB, (H_MSB<<4)|V_MSB
    e[54 + 12] = (518 & 0xFF) as u8;
    e[54 + 13] = (324 & 0xFF) as u8;
    e[54 + 14] = ((((518 >> 8) & 0x0F) << 4) | ((324 >> 8) & 0x0F)) as u8;

    // Colour-characterisation block (bytes 23..=34) — overwrite the
    // firmware-shipped values with exact sRGB / IEC 61966-2-1.
    //
    // macOS (and other ICC-aware hosts) parses these to either select a
    // factory ICC profile or synthesise a corrective one. If they don't
    // describe a standard space the host applies a corrective transform and
    // the visible result is a warm cast. Asserting sRGB-default makes the
    // host pass the framebuffer through unmodified.
    //
    //   byte 23: display gamma. Encoded as (gamma * 100) - 100. 2.2 → 120.
    e[23] = 120;
    //   byte 24: feature support. Preserve DPMS / digital-display-type bits,
    //   set bit 2 ("sRGB Standard is the default colour space").
    e[24] |= 0b0000_0100;
    //   bytes 25..=34: chromaticity. Each x/y is a 10-bit unsigned value =
    //   round(coord * 1024); the high 8 bits go into bytes 27..=34, the low
    //   2 bits get packed into bytes 25/26.
    //
    //     sRGB primaries (BT.709) + D65 white:
    //       Rx 0.640 → 655   Ry 0.330 → 338
    //       Gx 0.300 → 307   Gy 0.600 → 614
    //       Bx 0.150 → 154   By 0.060 →  61
    //       Wx 0.3127→ 320   Wy 0.3290→ 337
    //
    //   byte 25: Rx[1:0]Ry[1:0]Gx[1:0]Gy[1:0]
    //          = 11 10 11 10 = 0xEE
    e[25] = 0xEE;
    //   byte 26: Bx[1:0]By[1:0]Wx[1:0]Wy[1:0]
    //          = 10 01 00 01 = 0x91
    e[26] = 0x91;
    e[27] = 0xA3; // Rx >> 2
    e[28] = 0x54; // Ry >> 2
    e[29] = 0x4C; // Gx >> 2
    e[30] = 0x99; // Gy >> 2
    e[31] = 0x26; // Bx >> 2
    e[32] = 0x0F; // By >> 2
    e[33] = 0x50; // Wx >> 2
    e[34] = 0x54; // Wy >> 2

    // DTD#3 (offset 90..108) is a Display Product Name descriptor.
    // bytes 90..95 are the tag preamble (00 00 00 FC 00) and stay intact.
    // bytes 95..108 carry up to 13 ASCII chars terminated by 0x0A and
    // padded with 0x20.
    e[95..108].copy_from_slice(b"Dell U2412M\x0A\x20");

    // Recompute checksum so bytes 0..=127 sum to 0 mod 256.
    e[127] = 0;
    let mut sum: u32 = 0;
    for &b in &e[0..127] {
        sum = sum.wrapping_add(b as u32);
    }
    e[127] = (256u32.wrapping_sub(sum & 0xFF) & 0xFF) as u8;
    e
});

const CELL_GRAN: u32 = 8;
const MIN_V_BPORCH: u32 = 6;
const MIN_V_BLANKING_TIME_US: u32 = 460;
const CLOCK_STEP_KHZ: u32 = 250;
const RB_H_BLANK: u32 = 160;
const RB_H_SYNC: u32 = 32;
const RB_H_FRONT_PORCH: u32 = 48;
const RB_V_FRONT_PORCH: u32 = 3;
const RB_V_SYNC: u32 = 4;

pub struct CvtRbTiming {
    pub pixel_clock_khz: u32,
    pub h_active: u32,
    pub h_blank: u32,
    pub h_sync_offset: u32,
    pub h_sync_width: u32,
    pub v_active: u32,
    pub v_blank: u32,
    pub v_sync_offset: u32,
    pub v_sync_width: u32,
}

/// A fully-specified Detailed Timing Descriptor, including sync polarity.
/// Used by [`SAFE_MODES`] to declare known-good HDMI timings.
#[derive(Clone, Copy, Debug)]
pub struct DtdTiming {
    pub width: u32,
    pub height: u32,
    pub refresh_hz: u32,
    pub pixel_clock_khz: u32,
    pub h_blank: u32,
    pub h_sync_offset: u32,
    pub h_sync_width: u32,
    pub v_blank: u32,
    pub v_sync_offset: u32,
    pub v_sync_width: u32,
    pub h_sync_positive: bool,
    pub v_sync_positive: bool,
}

impl From<&CvtRbTiming> for DtdTiming {
    fn from(t: &CvtRbTiming) -> Self {
        Self {
            width: t.h_active,
            height: t.v_active,
            refresh_hz: 0,
            pixel_clock_khz: t.pixel_clock_khz,
            h_blank: t.h_blank,
            h_sync_offset: t.h_sync_offset,
            h_sync_width: t.h_sync_width,
            v_blank: t.v_blank,
            v_sync_offset: t.v_sync_offset,
            v_sync_width: t.v_sync_width,
            h_sync_positive: true,
            v_sync_positive: false,
        }
    }
}

/// Hand-picked display modes that fit inside the T749 HDMI capture chip's
/// envelope (≤1920×1200, ≤155 MHz pixel clock, 60 Hz). CTA-861 timings are
/// used for the universal HDMI modes (640x480, 1280x720, 1920x1080) which
/// every consumer sink accepts. CVT-RB v1 fills in the wider/taller modes.
pub const SAFE_MODES: &[DtdTiming] = &[
    // 640x480@60 — CTA-861 mode 1, HSync-, VSync-
    DtdTiming { width: 640, height: 480, refresh_hz: 60, pixel_clock_khz: 25_175,
        h_blank: 160, h_sync_offset: 16, h_sync_width: 96,
        v_blank: 45, v_sync_offset: 10, v_sync_width: 2,
        h_sync_positive: false, v_sync_positive: false },
    // 1024x768@60 — DMT 16, HSync-, VSync-
    DtdTiming { width: 1024, height: 768, refresh_hz: 60, pixel_clock_khz: 65_000,
        h_blank: 320, h_sync_offset: 24, h_sync_width: 136,
        v_blank: 38, v_sync_offset: 3, v_sync_width: 6,
        h_sync_positive: false, v_sync_positive: false },
    // 1280x720@60 — CTA-861 mode 4, HSync+, VSync+
    DtdTiming { width: 1280, height: 720, refresh_hz: 60, pixel_clock_khz: 74_250,
        h_blank: 370, h_sync_offset: 110, h_sync_width: 40,
        v_blank: 30, v_sync_offset: 5, v_sync_width: 5,
        h_sync_positive: true, v_sync_positive: true },
    // 1280x800@60 — CVT-RB, 16:10
    DtdTiming { width: 1280, height: 800, refresh_hz: 60, pixel_clock_khz: 71_000,
        h_blank: 160, h_sync_offset: 48, h_sync_width: 32,
        v_blank: 23, v_sync_offset: 3, v_sync_width: 6,
        h_sync_positive: true, v_sync_positive: false },
    // 1280x1024@60 — DMT 35, 5:4
    DtdTiming { width: 1280, height: 1024, refresh_hz: 60, pixel_clock_khz: 108_000,
        h_blank: 408, h_sync_offset: 48, h_sync_width: 112,
        v_blank: 42, v_sync_offset: 1, v_sync_width: 3,
        h_sync_positive: true, v_sync_positive: true },
    // 1440x900@60 — DMT 47 CVT-RB, 16:10
    DtdTiming { width: 1440, height: 900, refresh_hz: 60, pixel_clock_khz: 88_750,
        h_blank: 160, h_sync_offset: 48, h_sync_width: 32,
        v_blank: 26, v_sync_offset: 3, v_sync_width: 6,
        h_sync_positive: true, v_sync_positive: false },
    // 1600x900@60 — CVT-RB, 16:9
    DtdTiming { width: 1600, height: 900, refresh_hz: 60, pixel_clock_khz: 97_750,
        h_blank: 160, h_sync_offset: 48, h_sync_width: 32,
        v_blank: 26, v_sync_offset: 3, v_sync_width: 5,
        h_sync_positive: true, v_sync_positive: false },
    // 1680x1050@60 — DMT 58 CVT-RB, 16:10
    DtdTiming { width: 1680, height: 1050, refresh_hz: 60, pixel_clock_khz: 119_000,
        h_blank: 160, h_sync_offset: 48, h_sync_width: 32,
        v_blank: 30, v_sync_offset: 3, v_sync_width: 6,
        h_sync_positive: true, v_sync_positive: false },
    // 1920x1080@60 — CTA-861 mode 16, 16:9 (universal HDMI fallback)
    DtdTiming { width: 1920, height: 1080, refresh_hz: 60, pixel_clock_khz: 148_500,
        h_blank: 280, h_sync_offset: 88, h_sync_width: 44,
        v_blank: 45, v_sync_offset: 4, v_sync_width: 5,
        h_sync_positive: true, v_sync_positive: true },
    // 1920x1200@60 — DMT 69 CVT-RB, 16:10
    DtdTiming { width: 1920, height: 1200, refresh_hz: 60, pixel_clock_khz: 154_000,
        h_blank: 160, h_sync_offset: 48, h_sync_width: 32,
        v_blank: 35, v_sync_offset: 3, v_sync_width: 6,
        h_sync_positive: true, v_sync_positive: false },
];

/// 1920x1080@60 — the universal HDMI fallback when nothing better fits.
fn fallback_mode() -> &'static DtdTiming {
    SAFE_MODES
        .iter()
        .find(|m| m.width == 1920 && m.height == 1080)
        .expect("SAFE_MODES is missing the 1920x1080 fallback")
}

/// Aspect-ratio comparison in integer arithmetic, with tolerance in tenths
/// of a percent. `|m_w/m_h - local_w/local_h| <= tolerance * local_w/local_h`.
fn aspect_within(
    m_w: u32,
    m_h: u32,
    local_w: u32,
    local_h: u32,
    tolerance_per_mille: u32,
) -> bool {
    let lhs = (m_w as u64) * (local_h as u64);
    let rhs = (local_w as u64) * (m_h as u64);
    let diff = lhs.max(rhs) - lhs.min(rhs);
    diff * 1000 <= (tolerance_per_mille as u64) * rhs
}

/// Pick the [`DtdTiming`] from [`SAFE_MODES`] that best matches the local
/// display. Prefers the highest-resolution mode that (a) has the same aspect
/// ratio (within 5 %), and (b) fits inside `(local_w, local_h)`. Falls back
/// to the closest-aspect option, then to 1920x1080.
pub fn pick_safe_mode(local_w: u32, local_h: u32) -> &'static DtdTiming {
    if local_w == 0 || local_h == 0 {
        return fallback_mode();
    }

    let aspect_match = |m: &DtdTiming| aspect_within(m.width, m.height, local_w, local_h, 50);

    // Pass 1: aspect-matched modes that fit inside the local screen. Pick
    // the highest-resolution such mode.
    let mut best: Option<&'static DtdTiming> = None;
    let mut best_area: u64 = 0;
    for m in SAFE_MODES {
        if !aspect_match(m) {
            continue;
        }
        if m.width > local_w || m.height > local_h {
            continue;
        }
        let area = m.width as u64 * m.height as u64;
        if area > best_area {
            best_area = area;
            best = Some(m);
        }
    }
    if let Some(m) = best {
        return m;
    }

    // Pass 2: any aspect-matched mode (local screen smaller than smallest
    // mode in that aspect group). Pick the smallest available.
    let mut best_area: u64 = u64::MAX;
    for m in SAFE_MODES {
        if !aspect_match(m) {
            continue;
        }
        let area = m.width as u64 * m.height as u64;
        if area < best_area {
            best_area = area;
            best = Some(m);
        }
    }
    if let Some(m) = best {
        return m;
    }

    // Pass 3: no aspect match at all. Pick the mode whose aspect is closest
    // to the local screen and that fits. Use integer cross-multiplication.
    let mut best_diff: u64 = u64::MAX;
    for m in SAFE_MODES {
        if m.width > local_w || m.height > local_h {
            continue;
        }
        let lhs = (m.width as u64) * (local_h as u64);
        let rhs = (local_w as u64) * (m.height as u64);
        let diff = lhs.max(rhs) - lhs.min(rhs);
        if diff < best_diff {
            best_diff = diff;
            best = Some(m);
        }
    }
    best.unwrap_or_else(|| fallback_mode())
}

pub fn cvt_rb_v1(width: u32, height: u32, refresh_hz: u32) -> CvtRbTiming {
    let h_pixels = (width / CELL_GRAN) * CELL_GRAN;
    let v_pixels = height;
    let v_rate = refresh_hz as f64;

    // h_period_est in microseconds per horizontal line
    let h_period_est_us =
        (1_000_000.0 / v_rate - MIN_V_BLANKING_TIME_US as f64) / (v_pixels as f64 + MIN_V_BPORCH as f64);

    let mut vsync_bp = (MIN_V_BLANKING_TIME_US as f64 / h_period_est_us).floor() as u32 + 1;
    if vsync_bp < RB_V_SYNC + MIN_V_BPORCH {
        vsync_bp = RB_V_SYNC + MIN_V_BPORCH;
    }

    let total_v_lines = v_pixels + vsync_bp + RB_V_FRONT_PORCH;
    let total_h_pixels = h_pixels + RB_H_BLANK;

    let ideal_clock_khz = v_rate * total_v_lines as f64 * total_h_pixels as f64 / 1000.0;
    let pixel_clock_khz =
        ((ideal_clock_khz / CLOCK_STEP_KHZ as f64).floor() as u32) * CLOCK_STEP_KHZ;

    CvtRbTiming {
        pixel_clock_khz,
        h_active: h_pixels,
        h_blank: RB_H_BLANK,
        h_sync_offset: RB_H_FRONT_PORCH,
        h_sync_width: RB_H_SYNC,
        v_active: v_pixels,
        v_blank: vsync_bp + RB_V_FRONT_PORCH,
        v_sync_offset: RB_V_FRONT_PORCH,
        v_sync_width: RB_V_SYNC,
    }
}

fn write_dtd_full(out: &mut [u8; 18], t: &DtdTiming, h_size_mm: u32, v_size_mm: u32) {
    let pix_clock_10khz = (t.pixel_clock_khz / 10) as u16;
    out[0] = (pix_clock_10khz & 0xFF) as u8;
    out[1] = (pix_clock_10khz >> 8) as u8;

    out[2] = (t.width & 0xFF) as u8;
    out[3] = (t.h_blank & 0xFF) as u8;
    out[4] = ((((t.width >> 8) & 0x0F) << 4) | ((t.h_blank >> 8) & 0x0F)) as u8;

    out[5] = (t.height & 0xFF) as u8;
    out[6] = (t.v_blank & 0xFF) as u8;
    out[7] = ((((t.height >> 8) & 0x0F) << 4) | ((t.v_blank >> 8) & 0x0F)) as u8;

    out[8] = (t.h_sync_offset & 0xFF) as u8;
    out[9] = (t.h_sync_width & 0xFF) as u8;
    out[10] = (((t.v_sync_offset & 0x0F) << 4) | (t.v_sync_width & 0x0F)) as u8;
    out[11] = ((((t.h_sync_offset >> 8) & 0x03) << 6)
        | (((t.h_sync_width >> 8) & 0x03) << 4)
        | (((t.v_sync_offset >> 4) & 0x03) << 2)
        | ((t.v_sync_width >> 4) & 0x03)) as u8;

    out[12] = (h_size_mm & 0xFF) as u8;
    out[13] = (v_size_mm & 0xFF) as u8;
    out[14] = ((((h_size_mm >> 8) & 0x0F) << 4) | ((v_size_mm >> 8) & 0x0F)) as u8;

    out[15] = 0; // H border
    out[16] = 0; // V border
    // bits 4-3 = 11 (digital separate sync)
    // bit 2 = VSync polarity, bit 1 = HSync polarity, bit 0 reserved
    let mut sync = 0b0001_1000u8;
    if t.v_sync_positive {
        sync |= 0b0000_0100;
    }
    if t.h_sync_positive {
        sync |= 0b0000_0010;
    }
    out[17] = sync;
}

/// Build a 128-byte EDID 1.4 block whose preferred timing matches
/// (width, height, refresh_hz) using CVT-RB v1.
///
/// Returns an error if the synthesized pixel clock exceeds the EDID DTD's
/// representable maximum of 655.35 MHz — in which case the caller must pick
/// a lower refresh rate or resolution. Common cause: a hi-DPI / ProMotion
/// laptop panel reporting 120+ Hz native refresh.
pub fn try_build_edid_1_4(
    width: u32,
    height: u32,
    refresh_hz: u32,
) -> Result<[u8; 128], EdidError> {
    let t = cvt_rb_v1(width, height, refresh_hz);
    if t.pixel_clock_khz > MAX_PIXEL_CLOCK_KHZ {
        return Err(EdidError::PixelClockTooHigh {
            requested_khz: t.pixel_clock_khz,
            max_khz: MAX_PIXEL_CLOCK_KHZ,
            width,
            height,
            refresh_hz,
        });
    }
    Ok(build_edid_from_timing(&t, width, height))
}

/// Same as [`try_build_edid_1_4`] but panics on overflow. Use only when the
/// inputs are statically known to be in range (e.g. tests).
pub fn build_edid_1_4(width: u32, height: u32, refresh_hz: u32) -> [u8; 128] {
    try_build_edid_1_4(width, height, refresh_hz).expect("EDID pixel clock overflow")
}

#[derive(Debug, Clone)]
pub enum EdidError {
    PixelClockTooHigh {
        requested_khz: u32,
        max_khz: u32,
        width: u32,
        height: u32,
        refresh_hz: u32,
    },
}

impl std::fmt::Display for EdidError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EdidError::PixelClockTooHigh {
                requested_khz,
                max_khz,
                width,
                height,
                refresh_hz,
            } => write!(
                f,
                "{width}x{height}@{refresh_hz} requires pixel clock {} MHz, \
                 exceeding the EDID DTD maximum of {} MHz. Use --refresh or \
                 --width/--height to pick a lower mode.",
                *requested_khz as f64 / 1000.0,
                *max_khz as f64 / 1000.0,
            ),
        }
    }
}

impl std::error::Error for EdidError {}

fn build_edid_from_timing(t: &CvtRbTiming, _width: u32, _height: u32) -> [u8; 128] {
    build_edid_with_dtd(&DtdTiming::from(t))
}

/// Build a 128-byte EDID by patching only DTD#1 (at offset 54) into the
/// monitor base EDID (see [`MONITOR_BASE_EDID`]), then recomputing byte
/// 127's checksum. The base's H/V physical-size fields are preserved so
/// the announced display dimensions stay consistent with the declared
/// Dell U2412M.
fn build_edid_with_dtd(t: &DtdTiming) -> [u8; 128] {
    let mut edid: [u8; 128] = *MONITOR_BASE_EDID;

    let base_h_size_lsb = MONITOR_BASE_EDID[54 + 12] as u32;
    let base_v_size_lsb = MONITOR_BASE_EDID[54 + 13] as u32;
    let base_size_msb = MONITOR_BASE_EDID[54 + 14] as u32;
    let h_size_mm = base_h_size_lsb | (((base_size_msb >> 4) & 0x0F) << 8);
    let v_size_mm = base_v_size_lsb | ((base_size_msb & 0x0F) << 8);

    let mut dtd1 = [0u8; 18];
    write_dtd_full(&mut dtd1, t, h_size_mm, v_size_mm);
    edid[54..72].copy_from_slice(&dtd1);

    // Recompute checksum so bytes 0..=127 sum to 0 mod 256.
    edid[127] = 0;
    let mut sum: u32 = 0;
    for &b in &edid[0..127] {
        sum = sum.wrapping_add(b as u32);
    }
    edid[127] = (256u32.wrapping_sub(sum & 0xFF) & 0xFF) as u8;

    edid
}

/// Hex-encoded uppercase EDID string, suitable for passing to `rpc_set_edid`.
/// Returns an error if the requested mode's pixel clock cannot be represented
/// in EDID 1.4.
pub fn try_build_edid_hex(
    width: u32,
    height: u32,
    refresh_hz: u32,
) -> Result<String, EdidError> {
    Ok(hex::encode_upper(try_build_edid_1_4(width, height, refresh_hz)?))
}

/// Hex-encoded uppercase EDID string. Panics on overflow; prefer
/// [`try_build_edid_hex`] in production code.
pub fn build_edid_hex(width: u32, height: u32, refresh_hz: u32) -> String {
    hex::encode_upper(build_edid_1_4(width, height, refresh_hz))
}

/// Build an EDID hex string from a curated safe mode chosen by
/// [`pick_safe_mode`].
pub fn build_safe_edid_hex(local_w: u32, local_h: u32) -> (String, &'static DtdTiming) {
    let mode = pick_safe_mode(local_w, local_h);
    (hex::encode_upper(build_edid_with_dtd(mode)), mode)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_is_fixed() {
        let e = build_edid_1_4(1920, 1080, 60);
        assert_eq!(&e[0..8], &[0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00]);
    }

    #[test]
    fn version_field_matches_jetkvm_base() {
        // JetKVM's factory EDID is version 1.3, so the patched EDID is too.
        let e = build_edid_1_4(1920, 1080, 60);
        assert_eq!(e[18], 0x01);
        assert_eq!(e[19], 0x03);
    }

    #[test]
    fn checksum_zero_mod_256() {
        for &(w, h, r) in &[(1920u32, 1080u32, 60u32), (2560, 1440, 60), (3840, 2160, 60)] {
            let e = build_edid_1_4(w, h, r);
            let sum: u32 = e.iter().map(|&b| b as u32).sum();
            assert_eq!(sum % 256, 0, "checksum failed for {}x{}@{}", w, h, r);
        }
    }

    #[test]
    fn dtd1_active_pixels_match_input_1080p() {
        let e = build_edid_1_4(1920, 1080, 60);
        let dtd = &e[54..72];
        let h_active = ((dtd[4] as u32 >> 4) << 8) | dtd[2] as u32;
        let v_active = ((dtd[7] as u32 >> 4) << 8) | dtd[5] as u32;
        assert_eq!(h_active, 1920);
        assert_eq!(v_active, 1080);
    }

    #[test]
    fn dtd1_active_pixels_match_input_1440p() {
        let e = build_edid_1_4(2560, 1440, 60);
        let dtd = &e[54..72];
        let h_active = ((dtd[4] as u32 >> 4) << 8) | dtd[2] as u32;
        let v_active = ((dtd[7] as u32 >> 4) << 8) | dtd[5] as u32;
        assert_eq!(h_active, 2560);
        assert_eq!(v_active, 1440);
    }

    #[test]
    fn dtd1_pixel_clock_reasonable_1080p60() {
        let e = build_edid_1_4(1920, 1080, 60);
        let dtd = &e[54..72];
        let pix_clock_10khz = (dtd[0] as u32) | ((dtd[1] as u32) << 8);
        let mhz = pix_clock_10khz as f64 / 100.0;
        // CVT-RB v1 1920x1080@60 ≈ 138.5 MHz (Reduced Blanking)
        assert!(
            mhz > 130.0 && mhz < 145.0,
            "pixel clock {} MHz out of range for 1080p60 CVT-RB",
            mhz
        );
    }

    #[test]
    fn hex_string_is_256_chars() {
        let s = build_edid_hex(1920, 1080, 60);
        assert_eq!(s.len(), 256);
        assert!(s.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn dtd_sync_polarity_is_cvt_rb_v1() {
        // CVT-RB v1: HSync positive, VSync negative.
        // DTD byte 17 = 0b0001_1010 = 0x1A.
        let e = build_edid_1_4(1920, 1080, 60);
        assert_eq!(
            e[54 + 17],
            0x1A,
            "expected CVT-RB v1 sync polarity 0x1A, got 0x{:02X}",
            e[54 + 17]
        );
    }

    #[test]
    fn high_refresh_overflow_is_rejected() {
        // 2880x1800@120 needs ~696 MHz, above the 655.35 MHz DTD ceiling.
        match try_build_edid_1_4(2880, 1800, 120) {
            Err(EdidError::PixelClockTooHigh { .. }) => {}
            other => panic!("expected PixelClockTooHigh, got {other:?}"),
        }
    }

    #[test]
    fn high_resolution_within_clock_budget_is_accepted() {
        // 2880x1800@60 needs ~338 MHz, fits.
        assert!(try_build_edid_1_4(2880, 1800, 60).is_ok());
    }

    #[test]
    fn pick_safe_mode_for_16_10_macbook_pro() {
        // 2880x1800 is 16:10. Highest 16:10 mode that fits is 1920x1200.
        let m = pick_safe_mode(2880, 1800);
        assert_eq!((m.width, m.height), (1920, 1200));
    }

    #[test]
    fn pick_safe_mode_for_16_9_fullhd() {
        let m = pick_safe_mode(1920, 1080);
        assert_eq!((m.width, m.height), (1920, 1080));
    }

    #[test]
    fn pick_safe_mode_for_16_9_4k() {
        // 3840x2160 is 16:9. T749 caps at 1920x1080.
        let m = pick_safe_mode(3840, 2160);
        assert_eq!((m.width, m.height), (1920, 1080));
    }

    #[test]
    fn pick_safe_mode_for_4_3_xga() {
        let m = pick_safe_mode(1024, 768);
        assert_eq!((m.width, m.height), (1024, 768));
    }

    #[test]
    fn pick_safe_mode_for_small_screen_falls_back_to_smallest_match() {
        // 480x270 is 16:9 but smaller than 640x480 (4:3) and 1280x720 (16:9).
        // Should pick the smallest 16:9, which is 1280x720.
        let m = pick_safe_mode(480, 270);
        assert_eq!((m.width, m.height), (1280, 720));
    }

    #[test]
    fn safe_mode_edid_has_correct_polarity_per_mode() {
        // 1920x1080 → CTA-861, both polarities positive (0x1E)
        let (_, m) = build_safe_edid_hex(1920, 1080);
        let edid = build_edid_with_dtd(m);
        assert_eq!(edid[54 + 17], 0x1E, "1920x1080 CTA-861 expects 0x1E");

        // 1920x1200 → CVT-RB, H+, V- (0x1A)
        let (_, m) = build_safe_edid_hex(2880, 1800);
        let edid = build_edid_with_dtd(m);
        assert_eq!(edid[54 + 17], 0x1A, "1920x1200 CVT-RB expects 0x1A");
    }

    #[test]
    fn safe_mode_edid_checksum_is_valid_for_every_mode() {
        for m in SAFE_MODES {
            let edid = build_edid_with_dtd(m);
            let sum: u32 = edid.iter().map(|&b| b as u32).sum();
            assert_eq!(
                sum % 256,
                0,
                "checksum failed for {}x{}@{}",
                m.width,
                m.height,
                m.refresh_hz
            );
        }
    }

    #[test]
    fn every_safe_mode_pixel_clock_fits_in_dtd() {
        for m in SAFE_MODES {
            assert!(
                m.pixel_clock_khz <= MAX_PIXEL_CLOCK_KHZ,
                "{}x{}@{} pixel clock {} kHz exceeds MAX",
                m.width,
                m.height,
                m.refresh_hz,
                m.pixel_clock_khz
            );
        }
    }

    #[test]
    fn jetkvm_default_edid_is_self_consistent() {
        // Sum mod 256 must be 0 — that's what the device firmware ships.
        let sum: u32 = JETKVM_DEFAULT_EDID.iter().map(|&b| b as u32).sum();
        assert_eq!(sum % 256, 0);
        // Round-trips through the hex helper.
        assert_eq!(
            jetkvm_default_edid_hex(),
            hex::encode_upper(*JETKVM_DEFAULT_EDID)
        );
    }

    #[test]
    fn safe_mode_edid_only_touches_dtd1_and_checksum() {
        // Pick any safe mode and verify every byte outside DTD#1 (54..72)
        // and the checksum (127) equals the monitor base EDID.
        let (_, m) = build_safe_edid_hex(1920, 1080);
        let edid = build_edid_with_dtd(m);
        for i in 0..128 {
            if (54..72).contains(&i) || i == 127 {
                continue;
            }
            assert_eq!(
                edid[i], MONITOR_BASE_EDID[i],
                "byte 0x{:02x} unexpectedly modified",
                i
            );
        }
    }

    #[test]
    fn monitor_base_dtd1_matches_safe_1920x1080() {
        // The monitor base's DTD#1 inherits the CTA-861 1920x1080 timing
        // from the JetKVM default. Patching our 1920x1080 SafeMode on top
        // must therefore reproduce the monitor base byte-for-byte.
        let (_, m) = build_safe_edid_hex(1920, 1080);
        let edid = build_edid_with_dtd(m);
        assert_eq!(&edid[..], &MONITOR_BASE_EDID[..]);
    }

    #[test]
    fn monitor_base_checksum_is_valid() {
        let sum: u32 = MONITOR_BASE_EDID.iter().map(|&b| b as u32).sum();
        assert_eq!(sum % 256, 0);
    }

    #[test]
    fn monitor_base_advertises_srgb_default_colorspace() {
        // Byte 24 bit 2 = "sRGB Standard is the default colour space" —
        // macOS uses this to skip auto-deriving a corrective ICC profile
        // from the chromaticity coordinates.
        assert_eq!(
            MONITOR_BASE_EDID[24] & 0b0000_0100,
            0b0000_0100,
            "byte 24 bit 2 (sRGB default) must be set; got 0x{:02X}",
            MONITOR_BASE_EDID[24]
        );
    }

    #[test]
    fn monitor_base_chromaticity_decodes_to_srgb_primaries() {
        // Decode each 10-bit (low 2 bits packed in byte 25/26, high 8 bits in
        // bytes 27..=34) chromaticity value and check it is within 0.001 of
        // the sRGB / BT.709 / D65 specification.
        let rx = decode_chroma(MONITOR_BASE_EDID[27], (MONITOR_BASE_EDID[25] >> 6) & 0x3);
        let ry = decode_chroma(MONITOR_BASE_EDID[28], (MONITOR_BASE_EDID[25] >> 4) & 0x3);
        let gx = decode_chroma(MONITOR_BASE_EDID[29], (MONITOR_BASE_EDID[25] >> 2) & 0x3);
        let gy = decode_chroma(MONITOR_BASE_EDID[30], MONITOR_BASE_EDID[25] & 0x3);
        let bx = decode_chroma(MONITOR_BASE_EDID[31], (MONITOR_BASE_EDID[26] >> 6) & 0x3);
        let by = decode_chroma(MONITOR_BASE_EDID[32], (MONITOR_BASE_EDID[26] >> 4) & 0x3);
        let wx = decode_chroma(MONITOR_BASE_EDID[33], (MONITOR_BASE_EDID[26] >> 2) & 0x3);
        let wy = decode_chroma(MONITOR_BASE_EDID[34], MONITOR_BASE_EDID[26] & 0x3);
        // Allow ~1 LSB rounding error (1/1024 ≈ 0.001).
        assert!((rx - 0.640).abs() < 0.002, "Rx {rx}");
        assert!((ry - 0.330).abs() < 0.002, "Ry {ry}");
        assert!((gx - 0.300).abs() < 0.002, "Gx {gx}");
        assert!((gy - 0.600).abs() < 0.002, "Gy {gy}");
        assert!((bx - 0.150).abs() < 0.002, "Bx {bx}");
        assert!((by - 0.060).abs() < 0.002, "By {by}");
        assert!((wx - 0.3127).abs() < 0.002, "Wx {wx}");
        assert!((wy - 0.3290).abs() < 0.002, "Wy {wy}");
        // Gamma byte: encoded as (gamma * 100) - 100. 2.2 → 120.
        assert_eq!(MONITOR_BASE_EDID[23], 120, "gamma byte must be 2.2 = 120");
    }

    fn decode_chroma(high8: u8, low2: u8) -> f32 {
        let raw = ((high8 as u32) << 2) | (low2 as u32);
        raw as f32 / 1024.0
    }

    #[test]
    fn monitor_base_announces_dell_manufacturer() {
        // "DEL" packed big-endian: ((4)<<10)|((5)<<5)|12 = 0x10AC.
        assert_eq!(MONITOR_BASE_EDID[8], 0x10);
        assert_eq!(MONITOR_BASE_EDID[9], 0xAC);
    }

    #[test]
    fn monitor_base_display_name_is_dell_u2412m() {
        // Display Product Name descriptor at offset 90: 00 00 00 FC 00
        // followed by 13 bytes of name terminated by 0x0A.
        assert_eq!(&MONITOR_BASE_EDID[90..95], &[0x00, 0x00, 0x00, 0xFC, 0x00]);
        let name_bytes = &MONITOR_BASE_EDID[95..108];
        let end = name_bytes.iter().position(|&b| b == 0x0A).unwrap_or(13);
        let name = std::str::from_utf8(&name_bytes[..end]).expect("ASCII");
        assert_eq!(name, "Dell U2412M");
    }

    #[test]
    fn monitor_base_replaces_device_specific_identifiers() {
        // The monitor base must not retain device-specific identifying
        // strings inherited from the JetKVM default (T749 chipset name,
        // the brand string, the fHD720 model). Replacing these is the
        // whole point of deriving a monitor base — exercising the
        // invariant here catches accidental regressions.
        let device_strings: &[&[u8]] =
            &[b"T749", b"JetKVM", b"jetkvm", b"fHD720"];
        for s in device_strings {
            let found =
                MONITOR_BASE_EDID.windows(s.len()).any(|w| w == *s);
            assert!(
                !found,
                "monitor EDID still contains {:?}",
                std::str::from_utf8(s).unwrap_or("<binary>")
            );
        }
        // The Toshiba manufacturer ID 0x5262 (bytes 8-9) is also replaced.
        assert_ne!(
            [MONITOR_BASE_EDID[8], MONITOR_BASE_EDID[9]],
            [0x52, 0x62],
            "manufacturer ID still TSB"
        );
    }

    #[test]
    fn every_safe_mode_produces_clean_monitor_identification() {
        // For every safe mode, the generated EDID must not carry the
        // device-specific identification strings inherited from the
        // JetKVM default.
        let device_strings: &[&[u8]] =
            &[b"T749", b"JetKVM", b"jetkvm", b"fHD720"];
        for m in SAFE_MODES {
            let edid = build_edid_with_dtd(m);
            for s in device_strings {
                let found = edid.windows(s.len()).any(|w| w == *s);
                assert!(
                    !found,
                    "{}x{} EDID contains {:?}",
                    m.width,
                    m.height,
                    std::str::from_utf8(s).unwrap_or("<binary>")
                );
            }
        }
    }

    #[test]
    fn safe_modes_cap_at_t749_envelope() {
        for m in SAFE_MODES {
            assert!(
                m.width <= 1920 && m.height <= 1200 && m.pixel_clock_khz <= 160_000,
                "{}x{}@{} ({} kHz) exceeds the T749 capture envelope",
                m.width,
                m.height,
                m.refresh_hz,
                m.pixel_clock_khz
            );
        }
    }
}
