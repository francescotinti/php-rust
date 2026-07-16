//! ext/exif subset: `exif_imagetype`, `exif_read_data`, `iptcparse`
//! (ext/exif/exif.c + ext/standard/iptc.c), oracle-pinned by the WP-9 gd
//! probes (p09/p10). The consumer of record is WordPress'
//! `wp_read_image_metadata`: JPEG/TIFF EXIF (IFD0 + EXIF sub-IFD + IFD1
//! thumbnail + GPS), the COMPUTED pseudo-section, COM comments, and IPTC
//! APP13 parsing. MakerNote/INTEROP/FPIX/WINXP are not decoded (their
//! pointer tags surface as plain values), and multibyte `EXIF_USE_MBSTRING`
//! re-encoding is out of scope.

use php_runtime::Ctx;
use php_types::{convert, Diag, Key, PhpArray, PhpError, PhpStr, Zval};
use std::rc::Rc;

/// exif.c `tag_table_IFD` (TIFF/EXIF tag names, both IFD0 and the EXIF
/// sub-IFD share it).
const TAG_IFD: &[(u16, &str)] = &[
    (0x000B, "ACDComment"),
    (0x00FE, "NewSubFile"),
    (0x00FF, "SubFile"),
    (0x0100, "ImageWidth"),
    (0x0101, "ImageLength"),
    (0x0102, "BitsPerSample"),
    (0x0103, "Compression"),
    (0x0106, "PhotometricInterpretation"),
    (0x010A, "FillOrder"),
    (0x010D, "DocumentName"),
    (0x010E, "ImageDescription"),
    (0x010F, "Make"),
    (0x0110, "Model"),
    (0x0111, "StripOffsets"),
    (0x0112, "Orientation"),
    (0x0115, "SamplesPerPixel"),
    (0x0116, "RowsPerStrip"),
    (0x0117, "StripByteCounts"),
    (0x0118, "MinSampleValue"),
    (0x0119, "MaxSampleValue"),
    (0x011A, "XResolution"),
    (0x011B, "YResolution"),
    (0x011C, "PlanarConfiguration"),
    (0x011D, "PageName"),
    (0x011E, "XPosition"),
    (0x011F, "YPosition"),
    (0x0120, "FreeOffsets"),
    (0x0121, "FreeByteCounts"),
    (0x0122, "GrayResponseUnit"),
    (0x0123, "GrayResponseCurve"),
    (0x0124, "T4Options"),
    (0x0125, "T6Options"),
    (0x0128, "ResolutionUnit"),
    (0x0129, "PageNumber"),
    (0x012D, "TransferFunction"),
    (0x0131, "Software"),
    (0x0132, "DateTime"),
    (0x013B, "Artist"),
    (0x013C, "HostComputer"),
    (0x013D, "Predictor"),
    (0x013E, "WhitePoint"),
    (0x013F, "PrimaryChromaticities"),
    (0x0140, "ColorMap"),
    (0x0141, "HalfToneHints"),
    (0x0142, "TileWidth"),
    (0x0143, "TileLength"),
    (0x0144, "TileOffsets"),
    (0x0145, "TileByteCounts"),
    (0x014A, "SubIFD"),
    (0x014C, "InkSet"),
    (0x014D, "InkNames"),
    (0x014E, "NumberOfInks"),
    (0x0150, "DotRange"),
    (0x0151, "TargetPrinter"),
    (0x0152, "ExtraSample"),
    (0x0153, "SampleFormat"),
    (0x0154, "SMinSampleValue"),
    (0x0155, "SMaxSampleValue"),
    (0x0156, "TransferRange"),
    (0x0157, "ClipPath"),
    (0x0158, "XClipPathUnits"),
    (0x0159, "YClipPathUnits"),
    (0x015A, "Indexed"),
    (0x015B, "JPEGTables"),
    (0x015F, "OPIProxy"),
    (0x0200, "JPEGProc"),
    (0x0201, "JPEGInterchangeFormat"),
    (0x0202, "JPEGInterchangeFormatLength"),
    (0x0203, "JPEGRestartInterval"),
    (0x0205, "JPEGLosslessPredictors"),
    (0x0206, "JPEGPointTransforms"),
    (0x0207, "JPEGQTables"),
    (0x0208, "JPEGDCTables"),
    (0x0209, "JPEGACTables"),
    (0x0211, "YCbCrCoefficients"),
    (0x0212, "YCbCrSubSampling"),
    (0x0213, "YCbCrPositioning"),
    (0x0214, "ReferenceBlackWhite"),
    (0x02BC, "ExtensibleMetadataPlatform"),
    (0x0301, "Gamma"),
    (0x0302, "ICCProfileDescriptor"),
    (0x0303, "SRGBRenderingIntent"),
    (0x0320, "ImageTitle"),
    (0x5001, "ResolutionXUnit"),
    (0x5002, "ResolutionYUnit"),
    (0x5003, "ResolutionXLengthUnit"),
    (0x5004, "ResolutionYLengthUnit"),
    (0x5005, "PrintFlags"),
    (0x5006, "PrintFlagsVersion"),
    (0x5007, "PrintFlagsCrop"),
    (0x5008, "PrintFlagsBleedWidth"),
    (0x5009, "PrintFlagsBleedWidthScale"),
    (0x500A, "HalftoneLPI"),
    (0x500B, "HalftoneLPIUnit"),
    (0x500C, "HalftoneDegree"),
    (0x500D, "HalftoneShape"),
    (0x500E, "HalftoneMisc"),
    (0x500F, "HalftoneScreen"),
    (0x5010, "JPEGQuality"),
    (0x5011, "GridSize"),
    (0x5012, "ThumbnailFormat"),
    (0x5013, "ThumbnailWidth"),
    (0x5014, "ThumbnailHeight"),
    (0x5015, "ThumbnailColorDepth"),
    (0x5016, "ThumbnailPlanes"),
    (0x5017, "ThumbnailRawBytes"),
    (0x5018, "ThumbnailSize"),
    (0x5019, "ThumbnailCompressedSize"),
    (0x501A, "ColorTransferFunction"),
    (0x501B, "ThumbnailData"),
    (0x5020, "ThumbnailImageWidth"),
    (0x5021, "ThumbnailImageHeight"),
    (0x5022, "ThumbnailBitsPerSample"),
    (0x5023, "ThumbnailCompression"),
    (0x5024, "ThumbnailPhotometricInterp"),
    (0x5025, "ThumbnailImageDescription"),
    (0x5026, "ThumbnailEquipMake"),
    (0x5027, "ThumbnailEquipModel"),
    (0x5028, "ThumbnailStripOffsets"),
    (0x5029, "ThumbnailOrientation"),
    (0x502A, "ThumbnailSamplesPerPixel"),
    (0x502B, "ThumbnailRowsPerStrip"),
    (0x502C, "ThumbnailStripBytesCount"),
    (0x502D, "ThumbnailResolutionX"),
    (0x502E, "ThumbnailResolutionY"),
    (0x502F, "ThumbnailPlanarConfig"),
    (0x5030, "ThumbnailResolutionUnit"),
    (0x5031, "ThumbnailTransferFunction"),
    (0x5032, "ThumbnailSoftwareUsed"),
    (0x5033, "ThumbnailDateTime"),
    (0x5034, "ThumbnailArtist"),
    (0x5035, "ThumbnailWhitePoint"),
    (0x5036, "ThumbnailPrimaryChromaticities"),
    (0x5037, "ThumbnailYCbCrCoefficients"),
    (0x5038, "ThumbnailYCbCrSubsampling"),
    (0x5039, "ThumbnailYCbCrPositioning"),
    (0x503A, "ThumbnailRefBlackWhite"),
    (0x503B, "ThumbnailCopyRight"),
    (0x5090, "LuminanceTable"),
    (0x5091, "ChrominanceTable"),
    (0x5100, "FrameDelay"),
    (0x5101, "LoopCount"),
    (0x5110, "PixelUnit"),
    (0x5111, "PixelPerUnitX"),
    (0x5112, "PixelPerUnitY"),
    (0x5113, "PaletteHistogram"),
    (0x1000, "RelatedImageFileFormat"),
    (0x800D, "ImageID"),
    (0x80E3, "Matteing"),
    (0x80E4, "DataType"),
    (0x80E5, "ImageDepth"),
    (0x80E6, "TileDepth"),
    (0x828D, "CFARepeatPatternDim"),
    (0x828E, "CFAPattern"),
    (0x828F, "BatteryLevel"),
    (0x8298, "Copyright"),
    (0x829A, "ExposureTime"),
    (0x829D, "FNumber"),
    (0x83BB, "IPTC/NAA"),
    (0x84E3, "IT8RasterPadding"),
    (0x84E5, "IT8ColorTable"),
    (0x8649, "ImageResourceInformation"),
    (0x8769, "Exif_IFD_Pointer"),
    (0x8773, "ICC_Profile"),
    (0x8822, "ExposureProgram"),
    (0x8824, "SpectralSensitivity"),
    (0x8825, "GPS_IFD_Pointer"),
    (0x8827, "ISOSpeedRatings"),
    (0x8828, "OECF"),
    (0x9000, "ExifVersion"),
    (0x9003, "DateTimeOriginal"),
    (0x9004, "DateTimeDigitized"),
    (0x9010, "OffsetTime"),
    (0x9011, "OffsetTimeOriginal"),
    (0x9012, "OffsetTimeDigitized"),
    (0x9101, "ComponentsConfiguration"),
    (0x9102, "CompressedBitsPerPixel"),
    (0x9201, "ShutterSpeedValue"),
    (0x9202, "ApertureValue"),
    (0x9203, "BrightnessValue"),
    (0x9204, "ExposureBiasValue"),
    (0x9205, "MaxApertureValue"),
    (0x9206, "SubjectDistance"),
    (0x9207, "MeteringMode"),
    (0x9208, "LightSource"),
    (0x9209, "Flash"),
    (0x920A, "FocalLength"),
    (0x920B, "FlashEnergy"),
    (0x920C, "SpatialFrequencyResponse"),
    (0x920D, "Noise"),
    (0x920E, "FocalPlaneXResolution"),
    (0x920F, "FocalPlaneYResolution"),
    (0x9210, "FocalPlaneResolutionUnit"),
    (0x9211, "ImageNumber"),
    (0x9212, "SecurityClassification"),
    (0x9213, "ImageHistory"),
    (0x9214, "SubjectLocation"),
    (0x9215, "ExposureIndex"),
    (0x9216, "TIFF/EPStandardID"),
    (0x9217, "SensingMethod"),
    (0x923F, "StoNits"),
    (0x927C, "MakerNote"),
    (0x9286, "UserComment"),
    (0x9290, "SubSecTime"),
    (0x9291, "SubSecTimeOriginal"),
    (0x9292, "SubSecTimeDigitized"),
    (0x935C, "ImageSourceData"),
    (0x9C9B, "Title"),
    (0x9C9C, "Comments"),
    (0x9C9D, "Author"),
    (0x9C9E, "Keywords"),
    (0x9C9F, "Subject"),
    (0xA000, "FlashPixVersion"),
    (0xA001, "ColorSpace"),
    (0xA002, "ExifImageWidth"),
    (0xA003, "ExifImageLength"),
    (0xA004, "RelatedSoundFile"),
    (0xA005, "InteroperabilityOffset"),
    (0xA20B, "FlashEnergy"),
    (0xA20C, "SpatialFrequencyResponse"),
    (0xA20D, "Noise"),
    (0xA20E, "FocalPlaneXResolution"),
    (0xA20F, "FocalPlaneYResolution"),
    (0xA210, "FocalPlaneResolutionUnit"),
    (0xA211, "ImageNumber"),
    (0xA212, "SecurityClassification"),
    (0xA213, "ImageHistory"),
    (0xA214, "SubjectLocation"),
    (0xA215, "ExposureIndex"),
    (0xA216, "TIFF/EPStandardID"),
    (0xA217, "SensingMethod"),
    (0xA300, "FileSource"),
    (0xA301, "SceneType"),
    (0xA302, "CFAPattern"),
    (0xA401, "CustomRendered"),
    (0xA402, "ExposureMode"),
    (0xA403, "WhiteBalance"),
    (0xA404, "DigitalZoomRatio"),
    (0xA405, "FocalLengthIn35mmFilm"),
    (0xA406, "SceneCaptureType"),
    (0xA407, "GainControl"),
    (0xA408, "Contrast"),
    (0xA409, "Saturation"),
    (0xA40A, "Sharpness"),
    (0xA40B, "DeviceSettingDescription"),
    (0xA40C, "SubjectDistanceRange"),
    (0xA420, "ImageUniqueID"),
];

/// exif.c `tag_table_GPS`.
const TAG_GPS: &[(u16, &str)] = &[
    (0x0000, "GPSVersion"),
    (0x0001, "GPSLatitudeRef"),
    (0x0002, "GPSLatitude"),
    (0x0003, "GPSLongitudeRef"),
    (0x0004, "GPSLongitude"),
    (0x0005, "GPSAltitudeRef"),
    (0x0006, "GPSAltitude"),
    (0x0007, "GPSTimeStamp"),
    (0x0008, "GPSSatellites"),
    (0x0009, "GPSStatus"),
    (0x000A, "GPSMeasureMode"),
    (0x000B, "GPSDOP"),
    (0x000C, "GPSSpeedRef"),
    (0x000D, "GPSSpeed"),
    (0x000E, "GPSTrackRef"),
    (0x000F, "GPSTrack"),
    (0x0010, "GPSImgDirectionRef"),
    (0x0011, "GPSImgDirection"),
    (0x0012, "GPSMapDatum"),
    (0x0013, "GPSDestLatitudeRef"),
    (0x0014, "GPSDestLatitude"),
    (0x0015, "GPSDestLongitudeRef"),
    (0x0016, "GPSDestLongitude"),
    (0x0017, "GPSDestBearingRef"),
    (0x0018, "GPSDestBearing"),
    (0x0019, "GPSDestDistanceRef"),
    (0x001A, "GPSDestDistance"),
    (0x001B, "GPSProcessingMode"),
    (0x001C, "GPSAreaInformation"),
    (0x001D, "GPSDateStamp"),
    (0x001E, "GPSDifferential"),
];

/// exif.c `tag_table_IOP` (the Interoperability IFD, reached via
/// TAG_INTEROP_IFD_POINTER 0xA005).
const TAG_IOP: &[(u16, &str)] = &[
    (0x0001, "InterOperabilityIndex"),
    (0x0002, "InterOperabilityVersion"),
    (0x1000, "RelatedFileFormat"),
    (0x1001, "RelatedImageWidth"),
    (0x1002, "RelatedImageHeight"),
];

fn tag_name(table: &[(u16, &str)], tag: u16) -> String {
    for &(t, n) in table {
        if t == tag {
            return n.to_string();
        }
    }
    format!("UndefinedTag:0x{tag:04X}")
}

fn zstr(s: impl AsRef<[u8]>) -> Zval {
    Zval::Str(PhpStr::new(s.as_ref().to_vec()))
}

fn basename(path: &[u8]) -> &[u8] {
    path.rsplit(|&b| b == b'/').next().unwrap_or(path)
}

/// PHP's IMAGETYPE_* detection order (php_getimagetype). Shared by
/// `exif_imagetype` and image.rs.
pub(crate) fn detect_imagetype(d: &[u8]) -> Option<i64> {
    if d.len() >= 3 && &d[..3] == b"GIF" {
        return Some(1);
    }
    if d.len() >= 3 && d[0] == 0xFF && d[1] == 0xD8 && d[2] == 0xFF {
        return Some(2);
    }
    if d.len() >= 8 && d[..8] == *b"\x89PNG\r\n\x1a\n" {
        return Some(3);
    }
    if d.len() >= 3 && &d[..3] == b"FWS" {
        return Some(4);
    }
    if d.len() >= 3 && &d[..3] == b"CWS" {
        return Some(13);
    }
    if d.len() >= 4 && &d[..4] == b"8BPS" {
        return Some(5);
    }
    if d.len() >= 2 && &d[..2] == b"BM" {
        return Some(6);
    }
    if d.len() >= 4 && &d[..4] == b"II\x2a\x00" {
        return Some(7);
    }
    if d.len() >= 4 && &d[..4] == b"MM\x00\x2a" {
        return Some(8);
    }
    if d.len() >= 4 && &d[..4] == b"\xff\x4f\xff\x51" {
        return Some(9);
    }
    if d.len() >= 12 && &d[..12] == b"\x00\x00\x00\x0cjP  \x0d\x0a\x87\x0a" {
        return Some(10);
    }
    if d.len() >= 4 && &d[..4] == b"FORM" {
        return Some(14);
    }
    if d.len() >= 4 && &d[..4] == b"\x00\x00\x01\x00" {
        return Some(17);
    }
    if d.len() >= 12 && &d[..4] == b"RIFF" && &d[8..12] == b"WEBP" {
        return Some(18);
    }
    if is_avif(d) {
        return Some(19);
    }
    None
}

/// AVIF: an ISOBMFF `ftyp` box whose major or compatible brands include
/// `avif`/`avis` (php_is_image_avif via libavifinfo).
fn is_avif(d: &[u8]) -> bool {
    if d.len() < 12 || &d[4..8] != b"ftyp" {
        return false;
    }
    let size = be32(d, 0) as usize;
    let end = size.clamp(12, d.len().min(144));
    let mut i = 8;
    while i + 4 <= end {
        if &d[i..i + 4] == b"avif" || &d[i..i + 4] == b"avis" {
            return true;
        }
        i += 4;
    }
    false
}

/// Minimal libavifinfo work-alike for `getimagesize`: width/height from the
/// first `ispe` property, bits/channels from the first `pixi`, plus one for
/// an alpha auxiliary (`auxC` … `alpha`).
pub(crate) struct AvifInfo {
    pub width: u32,
    pub height: u32,
    pub bits: u32,
    pub channels: u32,
}

pub(crate) fn parse_avif_info(d: &[u8]) -> Option<AvifInfo> {
    if !is_avif(d) {
        return None;
    }
    let mut ispe: Option<(u32, u32)> = None;
    let mut pixi: Option<(u32, u32)> = None; // (channels, bits)
    let mut has_alpha = false;
    scan_boxes(d, 0, d.len(), 0, &mut ispe, &mut pixi, &mut has_alpha);
    let (width, height) = ispe?;
    let (mut channels, bits) = pixi.unwrap_or((3, 8));
    if has_alpha {
        channels += 1;
    }
    Some(AvifInfo { width, height, bits, channels })
}

fn scan_boxes(
    d: &[u8],
    mut i: usize,
    end: usize,
    depth: u32,
    ispe: &mut Option<(u32, u32)>,
    pixi: &mut Option<(u32, u32)>,
    has_alpha: &mut bool,
) {
    if depth > 6 {
        return;
    }
    while i + 8 <= end {
        let mut size = be32(d, i) as usize;
        let typ = &d[i + 4..i + 8];
        let mut hdr = 8;
        if size == 1 {
            if i + 16 > end {
                return;
            }
            let hi = be32(d, i + 8) as u64;
            let lo = be32(d, i + 12) as u64;
            size = ((hi << 32) | lo) as usize;
            hdr = 16;
        } else if size == 0 {
            size = end - i;
        }
        if size < hdr || i + size > end {
            return;
        }
        let body = i + hdr;
        match typ {
            // `meta` is a FullBox (4 bytes version/flags), the rest are
            // plain containers.
            b"meta" => {
                if body + 4 <= i + size {
                    scan_boxes(d, body + 4, i + size, depth + 1, ispe, pixi, has_alpha);
                }
            }
            b"iprp" | b"ipco" => {
                scan_boxes(d, body, i + size, depth + 1, ispe, pixi, has_alpha);
            }
            b"ispe" => {
                if ispe.is_none() && body + 12 <= i + size {
                    *ispe = Some((be32(d, body + 4), be32(d, body + 8)));
                }
            }
            b"pixi" => {
                if pixi.is_none() && body + 5 <= i + size {
                    let n = d[body + 4] as u32;
                    let bits = if body + 5 < i + size { d[body + 5] as u32 } else { 8 };
                    *pixi = Some((n, bits));
                }
            }
            b"auxC" => {
                let s = &d[body..i + size];
                if s.windows(5).any(|w| w == b"alpha") {
                    *has_alpha = true;
                }
            }
            _ => {}
        }
        i += size;
    }
}

fn be32(b: &[u8], i: usize) -> u32 {
    ((b[i] as u32) << 24) | ((b[i + 1] as u32) << 16) | ((b[i + 2] as u32) << 8) | b[i + 3] as u32
}

/// `exif_imagetype(string $filename): int|false`.
pub fn exif_imagetype(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let name = convert::to_zstr(
        args.first().ok_or_else(|| {
            PhpError::ArgumentCountError(
                "exif_imagetype() expects exactly 1 argument, 0 given".to_string(),
            )
        })?,
        ctx.diags,
    );
    let Some(data) = crate::file::read_for_builtin(name.as_bytes(), "exif_imagetype", ctx) else {
        return Ok(Zval::Bool(false));
    };
    Ok(match detect_imagetype(&data) {
        Some(t) => Zval::Long(t),
        None => Zval::Bool(false),
    })
}

/// `iptcparse(string $data): array|false` (ext/standard/iptc.c).
pub fn iptcparse(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = convert::to_zstr(args.first().unwrap_or(&Zval::Null), ctx.diags);
    let d = s.as_bytes();
    let mut out = PhpArray::new();
    let mut found = false;
    let mut i = 0usize;
    while i + 5 <= d.len() {
        // Seek the 0x1C dataset marker.
        if d[i] != 0x1C {
            i += 1;
            continue;
        }
        if i + 5 > d.len() {
            break;
        }
        let rec = d[i + 1];
        let ds = d[i + 2];
        let mut len = ((d[i + 3] as usize) << 8) | d[i + 4] as usize;
        i += 5;
        if len & 0x8000 != 0 {
            // Extended dataset: 4-byte length.
            if i + 4 > d.len() {
                break;
            }
            len = ((d[i] as usize) << 24)
                | ((d[i + 1] as usize) << 16)
                | ((d[i + 2] as usize) << 8)
                | d[i + 3] as usize;
            i += 4;
        }
        if i + len > d.len() {
            break;
        }
        let key = format!("{}#{:03}", rec, ds);
        let val = Zval::Str(PhpStr::new(d[i..i + len].to_vec()));
        let k = Key::from_bytes(key.as_bytes());
        match out.get_mut(&k) {
            Some(Zval::Array(list)) => {
                let l = Rc::make_mut(list);
                let _ = l.append(val);
            }
            _ => {
                let mut list = PhpArray::new();
                let _ = list.append(val);
                out.insert(k, Zval::Array(Rc::new(list)));
            }
        }
        found = true;
        i += len;
    }
    if !found {
        return Ok(Zval::Bool(false));
    }
    Ok(Zval::Array(Rc::new(out)))
}

// ---------------------------------------------------------------------------
// exif_read_data
// ---------------------------------------------------------------------------

/// One decoded tag (order-preserving).
struct TagVal {
    name: String,
    value: Zval,
}

struct TiffParse {
    motorola: bool,
    ifd0: Vec<TagVal>,
    exif: Vec<TagVal>,
    gps: Vec<TagVal>,
    thumbnail: Vec<TagVal>,
    /// (offset, length) of the IFD1 embedded thumbnail, tiff-relative.
    thumb_data: Option<(usize, usize)>,
    /// Running COMPUTED.ApertureFNumber, exif.c order semantics: TAG_FNUMBER
    /// overwrites, TAG_APERTURE/TAG_MAX_APERTURE apply only while it is 0
    /// (f32 like ImageInfo.ApertureFNumber, so the "f/%.1F" render matches).
    aperture_f: f32,
    /// Raw TAG_USER_COMMENT bytes (8-byte encoding header included), for
    /// COMPUTED.UserComment / UserCommentEncoding.
    user_comment: Option<Vec<u8>>,
    /// TAG_INTEROP_IFD_POINTER, then the parsed INTEROP section.
    interop_ptr: Option<usize>,
    interop: Vec<TagVal>,
    copyright: Option<Vec<u8>>,
}

struct Rd<'a> {
    d: &'a [u8],
    motorola: bool,
}

impl<'a> Rd<'a> {
    fn u16(&self, i: usize) -> Option<u16> {
        let b = self.d.get(i..i + 2)?;
        Some(if self.motorola {
            u16::from_be_bytes([b[0], b[1]])
        } else {
            u16::from_le_bytes([b[0], b[1]])
        })
    }
    fn u32(&self, i: usize) -> Option<u32> {
        let b = self.d.get(i..i + 4)?;
        Some(if self.motorola {
            u32::from_be_bytes([b[0], b[1], b[2], b[3]])
        } else {
            u32::from_le_bytes([b[0], b[1], b[2], b[3]])
        })
    }
}

fn format_size(fmt: u16) -> usize {
    match fmt {
        1 | 2 | 6 | 7 => 1,
        3 | 8 => 2,
        4 | 9 | 11 => 4,
        5 | 10 | 12 => 8,
        _ => 0,
    }
}

/// Decode one component at `off`.
fn decode_component(rd: &Rd, fmt: u16, off: usize) -> Option<Zval> {
    Some(match fmt {
        1 => Zval::Long(*rd.d.get(off)? as i64),
        6 => Zval::Long(*rd.d.get(off)? as i8 as i64),
        3 => Zval::Long(rd.u16(off)? as i64),
        8 => Zval::Long(rd.u16(off)? as i16 as i64),
        4 => Zval::Long(rd.u32(off)? as i64),
        9 => Zval::Long(rd.u32(off)? as i32 as i64),
        5 => {
            let n = rd.u32(off)?;
            let dnm = rd.u32(off + 4)?;
            zstr(format!("{n}/{dnm}").into_bytes())
        }
        10 => {
            let n = rd.u32(off)? as i32;
            let dnm = rd.u32(off + 4)? as i32;
            zstr(format!("{n}/{dnm}").into_bytes())
        }
        11 => Zval::Double(f32::from_bits(rd.u32(off)?) as f64),
        12 => {
            let hi = rd.u32(off)? as u64;
            let lo = rd.u32(off + 4)? as u64;
            let bits = if rd.motorola { (hi << 32) | lo } else { (lo << 32) | hi };
            Zval::Double(f64::from_bits(bits))
        }
        _ => return None,
    })
}

/// Decode a whole tag value (fmt 2 = ASCII up to the first NUL, fmt 7 = raw).
fn decode_value(rd: &Rd, fmt: u16, count: usize, off: usize) -> Option<Zval> {
    match fmt {
        2 => {
            let raw = rd.d.get(off..off + count)?;
            let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
            Some(zstr(&raw[..end]))
        }
        7 => {
            let raw = rd.d.get(off..off + count)?;
            Some(zstr(raw))
        }
        _ => {
            let sz = format_size(fmt);
            if sz == 0 {
                return None;
            }
            if count == 1 {
                decode_component(rd, fmt, off)
            } else {
                let mut a = PhpArray::new();
                for c in 0..count {
                    let _ = a.append(decode_component(rd, fmt, off + c * sz)?);
                }
                Some(Zval::Array(Rc::new(a)))
            }
        }
    }
}

/// Walk one IFD, returning its decoded tags in file order plus the
/// next-IFD offset.
fn parse_ifd(
    rd: &Rd,
    ifd_off: usize,
    table: &[(u16, &str)],
    out: &mut Vec<TagVal>,
    exif_ptr: &mut Option<usize>,
    gps_ptr: &mut Option<usize>,
    tp: &mut TiffParse,
) -> Option<usize> {
    let n = rd.u16(ifd_off)? as usize;
    for e in 0..n {
        let ent = ifd_off + 2 + e * 12;
        let tag = rd.u16(ent)?;
        let fmt = rd.u16(ent + 2)?;
        let count = rd.u32(ent + 4)? as usize;
        let total = format_size(fmt).max(if fmt == 2 || fmt == 7 { 1 } else { 0 }) * count;
        let voff = if total <= 4 { ent + 8 } else { rd.u32(ent + 8)? as usize };
        let Some(value) = decode_value(rd, fmt, count, voff) else {
            continue;
        };
        match tag {
            0x8769 => *exif_ptr = Some(rd.u32(ent + 8)? as usize),
            0x8825 => *gps_ptr = Some(rd.u32(ent + 8)? as usize),
            0x829D => {
                // TAG_FNUMBER: "simplest way of expressing aperture" —
                // overwrites any previously computed value (exif.c).
                if let Zval::Str(s) = &value {
                    if let Some((a, b)) = parse_rational_str(s.as_bytes()) {
                        if b != 0 {
                            tp.aperture_f = a as f32 / b as f32;
                        }
                    }
                }
            }
            0x9202 | 0x9205 => {
                // TAG_APERTURE / TAG_MAX_APERTURE (APEX): only while no better
                // aperture information was seen yet.
                if let Zval::Str(s) = &value {
                    if let Some((a, b)) = parse_rational_str(s.as_bytes()) {
                        if tp.aperture_f == 0.0 && b != 0 {
                            tp.aperture_f =
                                ((a as f32 / b as f32) * std::f32::consts::LN_2 * 0.5).exp();
                        }
                    }
                }
            }
            0x9286 => {
                // TAG_USER_COMMENT: raw bytes, decoded into COMPUTED at
                // assembly time (encoding header included).
                if let Zval::Str(s) = &value {
                    tp.user_comment = Some(s.as_bytes().to_vec());
                }
            }
            0xA005 => tp.interop_ptr = rd.u32(ent + 8).map(|v| v as usize),
            0x8298 => {
                if let Zval::Str(s) = &value {
                    tp.copyright = Some(s.as_bytes().to_vec());
                }
            }
            _ => {}
        }
        out.push(TagVal { name: tag_name(table, tag), value });
    }
    rd.u32(ifd_off + 2 + n * 12).map(|v| v as usize)
}

fn parse_rational_str(s: &[u8]) -> Option<(i64, i64)> {
    let s = std::str::from_utf8(s).ok()?;
    let (a, b) = s.split_once('/')?;
    Some((a.parse().ok()?, b.parse().ok()?))
}

/// Parse a TIFF blob (the Exif APP1 payload after `Exif\0\0`, or a whole
/// .tif file).
fn parse_tiff(d: &[u8]) -> Option<TiffParse> {
    let motorola = match d.get(..4)? {
        b"MM\x00\x2a" => true,
        b"II\x2a\x00" => false,
        _ => return None,
    };
    let rd = Rd { d, motorola };
    let ifd0_off = rd.u32(4)? as usize;
    let mut tp = TiffParse {
        motorola,
        ifd0: Vec::new(),
        exif: Vec::new(),
        gps: Vec::new(),
        thumbnail: Vec::new(),
        thumb_data: None,
        aperture_f: 0.0,
        user_comment: None,
        interop_ptr: None,
        interop: Vec::new(),
        copyright: None,
    };
    let mut exif_ptr = None;
    let mut gps_ptr = None;
    let mut ifd0 = Vec::new();
    let next = parse_ifd(&rd, ifd0_off, TAG_IFD, &mut ifd0, &mut exif_ptr, &mut gps_ptr, &mut tp);
    tp.ifd0 = ifd0;
    // IFD1 = the thumbnail directory.
    if let Some(next) = next {
        if next != 0 {
            let mut t = Vec::new();
            let mut e2 = None;
            let mut g2 = None;
            let _ = parse_ifd(&rd, next, TAG_IFD, &mut t, &mut e2, &mut g2, &mut tp);
            let mut off = None;
            let mut len = None;
            for tv in &t {
                if tv.name == "JPEGInterchangeFormat" {
                    if let Zval::Long(v) = tv.value {
                        off = Some(v as usize);
                    }
                }
                if tv.name == "JPEGInterchangeFormatLength" {
                    if let Zval::Long(v) = tv.value {
                        len = Some(v as usize);
                    }
                }
            }
            if let (Some(o), Some(l)) = (off, len) {
                if o + l <= d.len() {
                    tp.thumb_data = Some((o, l));
                }
            }
            tp.thumbnail = t;
        }
    }
    if let Some(p) = exif_ptr {
        let mut e = Vec::new();
        let mut e2 = None;
        let mut g2 = None;
        let _ = parse_ifd(&rd, p, TAG_IFD, &mut e, &mut e2, &mut g2, &mut tp);
        tp.exif = e;
    }
    if let Some(p) = gps_ptr {
        let mut g = Vec::new();
        let mut e2 = None;
        let mut g2 = None;
        let _ = parse_ifd(&rd, p, TAG_GPS, &mut g, &mut e2, &mut g2, &mut tp);
        tp.gps = g;
    }
    // The Interoperability IFD (pointer tag 0xA005, usually inside EXIF).
    if let Some(p) = tp.interop_ptr {
        let mut io = Vec::new();
        let mut e2 = None;
        let mut g2 = None;
        let _ = parse_ifd(&rd, p, TAG_IOP, &mut io, &mut e2, &mut g2, &mut tp);
        tp.interop = io;
    }
    Some(tp)
}

/// exif.c `exif_process_user_comment`: strip the 8-byte encoding header;
/// ASCII/UNDEFINED bodies lose their Olympus trailing-space padding and stop
/// at the first NUL; UNICODE decodes UTF-16 (BOM if present, else the TIFF
/// byte order) into the `exif.encode_unicode` default ISO-8859-15, '?' for
/// unmappable code points like the mbstring converter; JIS stays raw (the
/// empty `exif.encode_jis` default makes the C conversion fail into the raw
/// copy). No recognisable header → no encoding, whole value as string.
fn decode_user_comment(raw: &[u8], motorola: bool) -> (Option<&'static [u8]>, Vec<u8>) {
    fn strip_and_cut(s: &[u8]) -> Vec<u8> {
        let mut end = s.len();
        while end > 1 && s[end - 1] == b' ' {
            end -= 1; // exif.c never blanks index 0 (`for (a = ByteCount-1; a && ...`)
        }
        let t = &s[..end];
        let cut = t.iter().position(|&b| b == 0).unwrap_or(t.len());
        t[..cut].to_vec()
    }
    fn ucs2_to_8859_15(u: u16) -> u8 {
        match u {
            0x20AC => 0xA4,
            0x0160 => 0xA6,
            0x0161 => 0xA8,
            0x017D => 0xB4,
            0x017E => 0xB8,
            0x0152 => 0xBC,
            0x0153 => 0xBD,
            0x0178 => 0xBE,
            // The Latin-1 slots ISO-8859-15 repurposes are unmappable.
            0xA4 | 0xA6 | 0xA8 | 0xB4 | 0xB8 | 0xBC | 0xBD | 0xBE => b'?',
            u if u <= 0xFF => u as u8,
            _ => b'?',
        }
    }
    if raw.len() >= 8 {
        match &raw[..8] {
            b"UNICODE\0" => {
                let mut body = &raw[8..];
                let be = if body.len() >= 2 && &body[..2] == b"\xFE\xFF" {
                    body = &body[2..];
                    true
                } else if body.len() >= 2 && &body[..2] == b"\xFF\xFE" {
                    body = &body[2..];
                    false
                } else {
                    motorola
                };
                let out = body
                    .chunks_exact(2)
                    .map(|c| {
                        let u =
                            if be { u16::from_be_bytes([c[0], c[1]]) } else { u16::from_le_bytes([c[0], c[1]]) };
                        ucs2_to_8859_15(u)
                    })
                    .collect();
                return (Some(b"UNICODE"), out);
            }
            b"ASCII\0\0\0" => return (Some(b"ASCII"), strip_and_cut(&raw[8..])),
            b"JIS\0\0\0\0\0" => return (Some(b"JIS"), raw[8..].to_vec()),
            b"\0\0\0\0\0\0\0\0" => return (Some(b"UNDEFINED"), strip_and_cut(&raw[8..])),
            _ => {}
        }
    }
    (None, strip_and_cut(raw))
}

/// The pieces scanned out of a JPEG (or TIFF) for exif_read_data.
struct ExifScan {
    file_type: i64,
    width: u32,
    height: u32,
    channels: u32,
    comments: Vec<Vec<u8>>,
    tiff: Option<TiffParse>,
    tiff_slice: (usize, usize),
}

fn scan_jpeg(d: &[u8]) -> ExifScan {
    let mut scan = ExifScan {
        file_type: 2,
        width: 0,
        height: 0,
        channels: 0,
        comments: Vec::new(),
        tiff: None,
        tiff_slice: (0, 0),
    };
    let mut i = 2usize;
    while i + 4 <= d.len() {
        if d[i] != 0xFF {
            i += 1;
            continue;
        }
        let mut j = i;
        while j < d.len() && d[j] == 0xFF {
            j += 1;
        }
        if j >= d.len() {
            break;
        }
        let marker = d[j];
        let seg = j + 1;
        match marker {
            0xD8 | 0x01 | 0xD0..=0xD7 => {
                i = j + 1;
                continue;
            }
            0xD9 | 0xDA => break,
            _ => {}
        }
        if seg + 2 > d.len() {
            break;
        }
        let len = ((d[seg] as usize) << 8) | d[seg + 1] as usize;
        if len < 2 || seg + len > d.len() {
            break;
        }
        let body = &d[seg + 2..seg + len];
        match marker {
            0xC0 | 0xC1 | 0xC2 | 0xC3 | 0xC5 | 0xC6 | 0xC7 | 0xC9 | 0xCA | 0xCB | 0xCD | 0xCE
            | 0xCF => {
                if scan.width == 0 && body.len() >= 6 {
                    scan.height = ((body[1] as u32) << 8) | body[2] as u32;
                    scan.width = ((body[3] as u32) << 8) | body[4] as u32;
                    scan.channels = body[5] as u32;
                }
            }
            0xFE => {
                scan.comments.push(body.to_vec());
            }
            0xE1 => {
                if scan.tiff.is_none() && body.len() > 6 && &body[..6] == b"Exif\0\0" {
                    scan.tiff = parse_tiff(&body[6..]);
                    scan.tiff_slice = (seg + 2 + 6, len - 2 - 6);
                }
            }
            _ => {}
        }
        i = seg + len;
    }
    scan
}

fn push_tags(arr: &mut PhpArray, tags: &[TagVal]) {
    for t in tags {
        arr.insert(Key::from_bytes(t.name.as_bytes()), t.value.clone());
    }
}

fn tags_array(tags: &[TagVal]) -> Zval {
    let mut a = PhpArray::new();
    push_tags(&mut a, tags);
    Zval::Array(Rc::new(a))
}

/// `exif_read_data(string $file, ?string $required_sections = null,
///  bool $as_arrays = false, bool $read_thumbnail = false): array|false`.
pub fn exif_read_data(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    use std::os::unix::ffi::OsStrExt;
    let name = convert::to_zstr(
        args.first().ok_or_else(|| {
            PhpError::ArgumentCountError(
                "exif_read_data() expects at least 1 argument, 0 given".to_string(),
            )
        })?,
        ctx.diags,
    );
    let as_arrays = args.get(2).map(|v| convert::to_bool(v, ctx.diags)).unwrap_or(false);
    let base = basename(name.as_bytes()).to_vec();
    let path = std::ffi::OsStr::from_bytes(crate::file::strip_file_wrapper(name.as_bytes()));
    // The warning names the basename, not the full path (exif.c docref).
    let Some(data) =
        crate::file::read_for_builtin_named(name.as_bytes(), &base, "exif_read_data", ctx)
    else {
        return Ok(Zval::Bool(false));
    };
    let mtime = std::fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    exif_core(&data, &base, mtime, as_arrays, ctx)
}

/// `__exif_read_data_bytes($data, $display_path, $mtime, ...$rest)` —
/// VM-internal twin of `exif_read_data` for bytes already read through a
/// userland stream wrapper (the VM does the open/read/close; `$rest` are the
/// original call's trailing arguments, so `$as_arrays` sits at index 4).
pub fn exif_read_data_bytes(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let data = convert::to_zstr(args.first().unwrap_or(&Zval::Null), ctx.diags);
    let name = convert::to_zstr(args.get(1).unwrap_or(&Zval::Null), ctx.diags);
    let mtime = args.get(2).map(|v| convert::to_long_cast(v, ctx.diags)).unwrap_or(0);
    let as_arrays = args.get(4).map(|v| convert::to_bool(v, ctx.diags)).unwrap_or(false);
    let base = basename(name.as_bytes()).to_vec();
    exif_core(data.as_bytes(), &base, mtime, as_arrays, ctx)
}

/// `__exif_imagetype_bytes($data)` — VM-internal twin of `exif_imagetype` for
/// wrapper-read bytes: the IMAGETYPE_* constant, or `false` when unrecognised.
pub fn exif_imagetype_bytes(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let data = convert::to_zstr(args.first().unwrap_or(&Zval::Null), ctx.diags);
    Ok(detect_imagetype(data.as_bytes()).map(Zval::Long).unwrap_or(Zval::Bool(false)))
}

/// The shared back half of `exif_read_data`: scan `data`, assemble the
/// FILE/COMPUTED/sections output. `base` is the display name for warnings and
/// FileName, `mtime` fills FileDateTime.
fn exif_core(
    data: &[u8],
    base: &[u8],
    mtime: i64,
    as_arrays: bool,
    ctx: &mut Ctx,
) -> Result<Zval, PhpError> {
    let scan = match detect_imagetype(data) {
        Some(2) => scan_jpeg(&data),
        Some(t @ (7 | 8)) => {
            let tiff = parse_tiff(&data);
            let mut s = ExifScan {
                file_type: t,
                width: 0,
                height: 0,
                channels: 0,
                comments: Vec::new(),
                tiff,
                tiff_slice: (0, data.len()),
            };
            if let Some(tp) = &s.tiff {
                for t in &tp.ifd0 {
                    if let Zval::Long(v) = t.value {
                        if t.name == "ImageWidth" {
                            s.width = v as u32;
                        } else if t.name == "ImageLength" {
                            s.height = v as u32;
                        }
                    }
                }
            }
            s
        }
        _ => {
            ctx.diags.push(Diag::Warning(format!(
                "exif_read_data({}): File not supported",
                String::from_utf8_lossy(&base)
            )));
            return Ok(Zval::Bool(false));
        }
    };

    // SectionsFound, in exif.c's fixed section order.
    let has_tags = scan
        .tiff
        .as_ref()
        .is_some_and(|t| !t.ifd0.is_empty() || !t.exif.is_empty() || !t.interop.is_empty());
    let mut sections: Vec<&str> = Vec::new();
    if has_tags {
        sections.push("ANY_TAG");
    }
    if scan.tiff.as_ref().is_some_and(|t| !t.ifd0.is_empty()) {
        sections.push("IFD0");
    }
    if scan.tiff.as_ref().is_some_and(|t| !t.thumbnail.is_empty()) {
        sections.push("THUMBNAIL");
    }
    if !scan.comments.is_empty() {
        sections.push("COMMENT");
    }
    if scan.tiff.as_ref().is_some_and(|t| !t.exif.is_empty()) {
        sections.push("EXIF");
    }
    if scan.tiff.as_ref().is_some_and(|t| !t.gps.is_empty()) {
        sections.push("GPS");
    }
    if scan.tiff.as_ref().is_some_and(|t| !t.interop.is_empty()) {
        sections.push("INTEROP");
    }
    let sections_found = sections.join(", ");

    // COMPUTED.
    let mut computed = PhpArray::new();
    if scan.width > 0 {
        computed.insert(
            Key::from_bytes(b"html"),
            zstr(format!("width=\"{}\" height=\"{}\"", scan.width, scan.height).into_bytes()),
        );
        computed.insert(Key::from_bytes(b"Height"), Zval::Long(scan.height as i64));
        computed.insert(Key::from_bytes(b"Width"), Zval::Long(scan.width as i64));
    }
    computed.insert(
        Key::from_bytes(b"IsColor"),
        Zval::Long(if scan.channels >= 3 || scan.channels == 0 { 1 } else { 0 }),
    );
    if let Some(tp) = &scan.tiff {
        computed.insert(
            Key::from_bytes(b"ByteOrderMotorola"),
            Zval::Long(if tp.motorola { 1 } else { 0 }),
        );
        if tp.aperture_f != 0.0 {
            computed.insert(
                Key::from_bytes(b"ApertureFNumber"),
                zstr(format!("f/{:.1}", tp.aperture_f).into_bytes()),
            );
        }
        if let Some(uc) = &tp.user_comment {
            let (enc, text) = decode_user_comment(uc, tp.motorola);
            computed.insert(Key::from_bytes(b"UserComment"), zstr(&text));
            if let Some(e) = enc {
                computed.insert(Key::from_bytes(b"UserCommentEncoding"), zstr(e));
            }
        }
        if let Some(c) = &tp.copyright {
            if let Some(pos) = c.iter().position(|&b| b == 0) {
                let (ph, ed) = (&c[..pos], &c[pos + 1..]);
                computed.insert(Key::from_bytes(b"Copyright"), zstr(c.as_slice()));
                computed.insert(Key::from_bytes(b"Copyright.Photographer"), zstr(ph));
                computed.insert(Key::from_bytes(b"Copyright.Editor"), zstr(ed));
            } else {
                computed.insert(Key::from_bytes(b"Copyright"), zstr(c.as_slice()));
            }
        }
        if let Some((o, l)) = tp.thumb_data {
            let (ts, _) = scan.tiff_slice;
            let abs = ts + o;
            if abs + l <= data.len() {
                if let Some(t) = detect_imagetype(&data[abs..abs + l]) {
                    computed.insert(Key::from_bytes(b"Thumbnail.FileType"), Zval::Long(t));
                    computed.insert(
                        Key::from_bytes(b"Thumbnail.MimeType"),
                        zstr(crate::image::mime_for_type(t)),
                    );
                }
            }
        }
    }

    // Assemble.
    let mut out = PhpArray::new();
    let mime: &[u8] = match scan.file_type {
        2 => b"image/jpeg",
        7 | 8 => b"image/tiff",
        _ => b"application/octet-stream",
    };
    if as_arrays {
        let mut file = PhpArray::new();
        file.insert(Key::from_bytes(b"FileName"), zstr(&base));
        file.insert(Key::from_bytes(b"FileDateTime"), Zval::Long(mtime));
        file.insert(Key::from_bytes(b"FileSize"), Zval::Long(data.len() as i64));
        file.insert(Key::from_bytes(b"FileType"), Zval::Long(scan.file_type));
        file.insert(Key::from_bytes(b"MimeType"), zstr(mime));
        file.insert(Key::from_bytes(b"SectionsFound"), zstr(sections_found.as_bytes()));
        out.insert(Key::from_bytes(b"FILE"), Zval::Array(Rc::new(file)));
        out.insert(Key::from_bytes(b"COMPUTED"), Zval::Array(Rc::new(computed)));
        if let Some(tp) = &scan.tiff {
            if !tp.ifd0.is_empty() {
                out.insert(Key::from_bytes(b"IFD0"), tags_array(&tp.ifd0));
            }
            if !tp.thumbnail.is_empty() {
                out.insert(Key::from_bytes(b"THUMBNAIL"), tags_array(&tp.thumbnail));
            }
            if !tp.exif.is_empty() {
                out.insert(Key::from_bytes(b"EXIF"), tags_array(&tp.exif));
            }
            if !tp.gps.is_empty() {
                out.insert(Key::from_bytes(b"GPS"), tags_array(&tp.gps));
            }
            if !tp.interop.is_empty() {
                out.insert(Key::from_bytes(b"INTEROP"), tags_array(&tp.interop));
            }
        }
        if !scan.comments.is_empty() {
            let mut c = PhpArray::new();
            for com in &scan.comments {
                let _ = c.append(zstr(com));
            }
            out.insert(Key::from_bytes(b"COMMENT"), Zval::Array(Rc::new(c)));
        }
    } else {
        out.insert(Key::from_bytes(b"FileName"), zstr(&base));
        out.insert(Key::from_bytes(b"FileDateTime"), Zval::Long(mtime));
        out.insert(Key::from_bytes(b"FileSize"), Zval::Long(data.len() as i64));
        out.insert(Key::from_bytes(b"FileType"), Zval::Long(scan.file_type));
        out.insert(Key::from_bytes(b"MimeType"), zstr(mime));
        out.insert(Key::from_bytes(b"SectionsFound"), zstr(sections_found.as_bytes()));
        out.insert(Key::from_bytes(b"COMPUTED"), Zval::Array(Rc::new(computed)));
        if let Some(tp) = &scan.tiff {
            push_tags(&mut out, &tp.ifd0);
            if !tp.thumbnail.is_empty() {
                out.insert(Key::from_bytes(b"THUMBNAIL"), tags_array(&tp.thumbnail));
            }
            push_tags(&mut out, &tp.exif);
            if !tp.gps.is_empty() {
                push_tags(&mut out, &tp.gps);
            }
            push_tags(&mut out, &tp.interop);
        }
        if !scan.comments.is_empty() {
            let mut c = PhpArray::new();
            for com in &scan.comments {
                let _ = c.append(zstr(com));
            }
            out.insert(Key::from_bytes(b"COMMENT"), Zval::Array(Rc::new(c)));
        }
    }
    Ok(Zval::Array(Rc::new(out)))
}
