use std::{
    collections::{HashMap, hash_map::Entry},
    io,
    io::{Seek, SeekFrom},
};

use binrw::{BinRead, BinResult, BinWrite, Endian};
use bytes::Buf;
use image::GrayImage;

#[derive(BinRead, BinWrite, Debug)]
#[brw(little, magic = b"FNT4")]
struct Font0Header {
    pub size: u32,
    pub ascent: u16,
    pub descent: u16,
    pub padding: u32,
}

#[derive(BinRead, BinWrite, Debug)]
#[brw(little)]
struct Font0GlyphHeader {
    pub bearing_x: i8,
    pub bearing_y: i8,
    pub width: u8,
    pub height: u8,
    pub advance_width: u8,
    pub unk: u8,
    pub compressed_size: u16,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct GlyphMetrics {
    pub bearing_x: i8,
    pub bearing_y: i8,
    pub advance_width: u8,
    pub width: u8,
    pub height: u8,
}

impl From<Font0GlyphHeader> for GlyphMetrics {
    fn from(value: Font0GlyphHeader) -> Self {
        Self {
            bearing_x: value.bearing_x,
            bearing_y: value.bearing_y,
            advance_width: value.advance_width,
            width: value.width,
            height: value.height,
        }
    }
}

// this is similar to lz77 used on newer versions, but it stores the backseek spec in a single byte, has length and offset swapped and has different length bias
pub fn fnt0_lz77_decompress(input: &[u8], output: &mut Vec<u8>) {
    let mut input = io::Cursor::new(input);

    while input.has_remaining() {
        let map = input.get_u8();
        for i in 0..8 {
            if !input.has_remaining() {
                break;
            }

            if ((map >> i) & 1) == 0 {
                /* literal value */
                output.push(input.get_u8());
            } else {
                /* back seek */
                let backseek_spec = input.get_u8();

                /*  MSB  XXXXXXXX          YYYYYYYY    LSB
                    val  backOffset               len
                    size (16-OFFSET_BITS)  OFFSET_BITS
                */

                const LEN_BITS: u32 = 3;

                let len_mask = (1 << LEN_BITS) - 1; // magic to get the last OFFSET_BITS bits

                let len = (backseek_spec & len_mask) + 2;
                let back_offset = (backseek_spec >> LEN_BITS) + 1;

                for _ in 0..len {
                    let last = output.len() - back_offset as usize;
                    // TODO: make this fallible?
                    // TODO: this might be optimized by stopping the bounds checking after we have enough data to guarantee that it's in bounds
                    output.push(output[last]);
                }
            }
        }
    }
}

/// A newtype representing an ID of the glyph within the FNT file
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GlyphId(pub u32);

pub struct Font0Glyph {
    pub info: GlyphMetrics,
    pub image: GrayImage,
}

impl BinRead for Font0Glyph {
    type Args<'a> = ();

    fn read_options<R: io::Read + Seek>(
        reader: &mut R,
        endian: Endian,
        (): Self::Args<'_>,
    ) -> BinResult<Self> {
        let header = Font0GlyphHeader::read_options(reader, endian, ())?;
        let compressed_size = header.compressed_size as usize;

        let stride = header.width.div_ceil(2) as usize;

        let uncompressed_size = stride * header.height as usize;
        let info: GlyphMetrics = header.into();

        let data = if compressed_size == 0 {
            let mut data = vec![0; uncompressed_size];
            reader.read_exact(&mut data)?;
            data
        } else {
            let mut data = vec![0; compressed_size];
            reader.read_exact(&mut data)?;
            let mut decompressed_data = Vec::with_capacity(uncompressed_size);
            fnt0_lz77_decompress(&data, &mut decompressed_data);
            assert_eq!(decompressed_data.len(), uncompressed_size);
            decompressed_data
        };

        // convert 4bpp to 8bpp
        let mut pixel_data = Vec::with_capacity(info.width as usize * info.height as usize);
        for row_data in data.chunks_exact(stride) {
            for (x, v) in (0..info.width).zip(
                row_data
                    .iter()
                    .copied()
                    .flat_map(|v| std::iter::repeat_n(v, 2)),
            ) {
                let v = if x & 1 == 0 { v >> 4 } else { v & 0xf };
                let v = v << 4;
                pixel_data.push(v);
            }
        }

        let image = GrayImage::from_raw(info.width as u32, info.height as u32, pixel_data).unwrap();

        Ok(Self { info, image })
    }
}

fn stream_size(reader: &mut impl Seek) -> BinResult<u64> {
    let pos = reader.stream_position()?;
    let size = reader.seek(SeekFrom::End(0))?;
    reader.seek(SeekFrom::Start(pos))?;
    Ok(size)
}

fn font0_map_sjis_to_index(sjis_codepoint: u32) -> Option<u32> {
    struct SjisDoubleRange {
        size: u32,
        begin: u32,
        end: u32,
    }
    const RANGES: [SjisDoubleRange; 3] = [
        SjisDoubleRange {
            size: 0x16c4,
            begin: 0x8100,
            end: 0x9fff,
        },
        SjisDoubleRange {
            size: 0x814,
            begin: 0xe000,
            end: 0xeaff,
        },
        SjisDoubleRange {
            size: 0xbc,
            begin: 0xf000,
            end: 0xf0ff,
        },
    ];

    Some(match sjis_codepoint {
        0x20..=0x7f => sjis_codepoint - 0x20,        // ascii
        0xa0..=0xdf => sjis_codepoint - 0xa0 + 0x60, // single-byte sjis
        _ => {
            let mut range_size_accum = 0x0;
            let mut ranges_iter = RANGES.iter();
            let range = loop {
                let candidate_range = ranges_iter.next()?;
                if (candidate_range.begin..=candidate_range.end).contains(&sjis_codepoint) {
                    break candidate_range;
                }
                range_size_accum += candidate_range.size;
            };
            // each shift-jis high byte represents two shift-jis rows 96 characters wide each
            let mut hi = (((sjis_codepoint - range.begin) >> 8) * 188 + range_size_accum) as i32;
            let lo = (sjis_codepoint & 0xff) as i32;

            // check for invalid lo values
            if !(0x40..=0xfc).contains(&lo) || lo == 0x7f {
                return None;
            }

            // some sjis magic idk
            if sjis_codepoint & 0x80 != 0 {
                hi -= 1;
            }

            (hi + lo + 0x20 + 0x60) as u32
        }
    })
}

/// A macro similar to `vec![$elem; $size]` which returns a boxed array.
///
/// ```rustc
///     let _: Box<[u8; 1024]> = box_array![0; 1024];
/// ```
macro_rules! box_array {
    ($val:expr ; $len:expr) => {{
        // Use a generic function so that the pointer cast remains type-safe
        fn vec_to_boxed_array<T>(vec: Vec<T>) -> Box<[T; $len]> {
            let boxed_slice = vec.into_boxed_slice();

            let ptr = ::std::boxed::Box::into_raw(boxed_slice) as *mut [T; $len];

            unsafe { Box::from_raw(ptr) }
        }

        vec_to_boxed_array(vec![$val; $len])
    }};
}

const FONT0_GLYPH_COUNT: usize = 8244;

#[expect(unused)] // extraction of actual glyph data is implemented, but is not exposed currently (because it's been hacked together)
fn read_font0<R: io::Read + io::Seek>(reader: &mut R) -> BinResult<()> {
    let endian = binrw::Endian::Little;

    let stream_position = reader.stream_position()?;
    let header = Font0Header::read_options(reader, endian, ())?;
    let size = stream_size(reader)?;

    let mut character_table = box_array![0u32; FONT0_GLYPH_COUNT];
    for c in character_table.iter_mut() {
        *c = u32::read_options(reader, endian, ())?;
    }

    let mut known_glyph_offsets = HashMap::new();
    let mut characters = box_array![GlyphId(0); FONT0_GLYPH_COUNT];
    let mut glyphs = HashMap::new();

    for (character_index, &glyph_offset) in character_table.iter().enumerate() {
        let next_glyph_id = GlyphId(known_glyph_offsets.len() as u32);
        let glyph_id = *known_glyph_offsets
            .entry(glyph_offset)
            .or_insert(next_glyph_id);
        characters[character_index] = glyph_id;

        match glyphs.entry(glyph_id) {
            Entry::Occupied(_) => continue,
            Entry::Vacant(entry) => {
                reader.seek(SeekFrom::Start(glyph_offset as u64))?;
                entry.insert(Font0Glyph::read_options(
                    reader,
                    endian,
                    Default::default(),
                )?);
            }
        }
    }

    for character in 0..FONT0_GLYPH_COUNT {
        let glyph_id = characters[character];
        let glyph = glyphs.get(&glyph_id).unwrap();
        glyph
            .image
            .save(&format!("glyphs/{character:05}.png"))
            .unwrap();
    }

    for sjis in 0..0xffff {
        if let Some(flat) = font0_map_sjis_to_index(sjis) {
            println!("{sjis:04x}: {flat}")
        }
    }

    Ok(())
}

enum FontType {
    Font0,
    #[expect(unused)] // FNTv1 support not yet fully implemented, but planned
    Font1,
}

#[derive(Copy, Clone, Debug)]
pub struct FontInfo {
    /// Distance between the baseline and the top of the font
    pub ascent: u16,
    /// Distance between the baseline and the bottom of the font
    pub descent: u16,
}

pub struct FontMetrics {
    r#type: FontType,
    metrics: Box<[GlyphMetrics]>,
    info: FontInfo,
}

impl FontMetrics {
    pub fn from_font0<R: io::Read + io::Seek>(reader: &mut R) -> BinResult<Self> {
        let endian = binrw::Endian::Little;

        let header = Font0Header::read_options(reader, endian, ())?;

        let mut character_table = box_array![0u32; FONT0_GLYPH_COUNT];
        for c in character_table.iter_mut() {
            *c = u32::read_options(reader, endian, ())?;
        }

        let mut metrics = Vec::with_capacity(FONT0_GLYPH_COUNT);

        for &glyph_offset in character_table.iter() {
            reader.seek(SeekFrom::Start(glyph_offset as u64))?;
            let glyph_header = Font0GlyphHeader::read_options(reader, endian, Default::default())?;

            metrics.push(glyph_header.into());
        }

        Ok(Self {
            r#type: FontType::Font0,
            metrics: metrics.into_boxed_slice(),
            info: FontInfo {
                ascent: header.ascent,
                descent: header.descent,
            },
        })
    }

    pub fn get_info(&self) -> FontInfo {
        self.info
    }

    pub fn get_glyph_metrics(&self, codepoint: char) -> Option<GlyphMetrics> {
        let index = match self.r#type {
            FontType::Font0 => {
                let sjis = shin_text::encode_sjis_codepoint(codepoint, false)?;
                font0_map_sjis_to_index(sjis as u32)
                    .expect("BUG: a just-encoded sjis codepoint is unmapped to font")
                    as usize
            }
            FontType::Font1 => {
                todo!()
            }
        };

        Some(self.metrics[index])
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    #[test]
    #[ignore]
    pub fn smoke() {
        let font_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../test-assets/higurashi-sui/font.fnt");
        let font_data = std::fs::read(font_path).unwrap();

        super::read_font0(&mut io::Cursor::new(font_data.as_slice())).unwrap();
    }
}
