//! Encoding and decoding of Shift-JIS variant used by the shin engine.
//!
//! (copied from shin: https://github.com/DCNick3/shin/blob/master/shin-core/src/format/text/mod.rs)
// Maybe it should be polished and made into a separate library?

use std::io;

use bumpalo::Bump;

include!("decode_tables.rs");
include!("encode_tables.rs");

const ASCII_START: u8 = 0x20;
const ASCII_END: u8 = 0x80;
const KATAKANA_START: u8 = 0xa0;
const KATAKANA_END: u8 = 0xe0;

#[inline]
fn decode_single_sjis_char(c: u8, fixup: bool) -> char {
    if c < 0x20 {
        // SAFETY: c < 0x20, so it is safe to construct such a char
        unsafe { char::from_u32_unchecked(c as u32) }
    } else if (ASCII_START..ASCII_END).contains(&c) {
        let index = (c - 0x20) as usize;
        // SAFETY: index < 0x60, so it is safe to access the table
        unsafe { *ASCII_TABLE.get_unchecked(index) }
    } else if (KATAKANA_START..KATAKANA_END).contains(&c) {
        let index = (c - 0xa0) as usize;
        if fixup {
            // SAFETY: index < 0x40, so it is safe to access the table
            unsafe { *FIXUP_KATAKANA_TABLE.get_unchecked(index) }
        } else {
            // SAFETY: index < 0x40, so it is safe to access the table
            unsafe { *KATAKANA_TABLE.get_unchecked(index) }
        }
    } else {
        // unmapped, no such first byte
        '\0'
    }
}

#[inline]
fn decode_double_sjis_char(first: u8, second: u8) -> char {
    // column actually spans two JIS rows
    // so, it's in range 0-193
    let column = if matches!(second, 0x40..=0x7e | 0x80..=0xfc) {
        if (0x40..=0x7e).contains(&second) {
            second - 0x40
        } else {
            second - 0x41
        }
    } else {
        return '\0';
    } as usize;

    let row = match first {
        0x81..=0xa0 => (first - 0x81) * 2, // 64 JIS rows (each HI byte value spans 2 rows)
        0xe0..=0xfc => (first - 0xe0) * 2 + 62, // 58 JIS rows (each HI byte value spans 2 rows)
        _ => return '\0',
    } as usize;

    // row \in [0; 121]
    // column \in [0; 193]
    // addr \in [0; 121*94 + 193] = [0; 11567]
    let addr = row * 94 + column;

    // SAFETY: addr < 11567, so it is safe to access the table
    unsafe { *JIS_TABLE.get_unchecked(addr) }
}

fn is_extended(c: u8) -> bool {
    matches!(c, 0x81..=0x9f | 0xe0..=0xfc)
}

/// The game engine files are encoded in (a variant of) Shift-JIS
/// But the game engine itself uses UTF-8
/// This function converts (a variant of) Shift-JIS to UTF-8
/// This function stops reading either at the first null byte or when byte_size bytes have been read
pub fn decode_sjis_zstring<'bump>(
    bump: &'bump Bump,
    mut s: &[u8],
    fixup: bool,
) -> io::Result<bumpalo::collections::String<'bump>> {
    let mut res = bumpalo::collections::String::new_in(bump);
    // TODO: maybe there is a better estimation
    res.reserve(s.len());

    while s.last() == Some(&0) {
        s = &s[..s.len() - 1];
    }

    let mut b = s.iter().cloned();
    while let Some(c1) = b.next() {
        let utf8_c = if is_extended(c1) {
            let c2 = b.next().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "unexpected end of string when reading double-byte char",
                )
            })?;
            let utf8_c = decode_double_sjis_char(c1, c2);

            if utf8_c == '\0' {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("unmappable sjis char: 0x{:02x}, 0x{:02x}", c1, c2),
                ));
            }
            utf8_c
        } else {
            let utf8_c = decode_single_sjis_char(c1, fixup);
            if utf8_c == '\0' {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("invalid single-byte char: 0x{:02x}", c1),
                ));
            }
            utf8_c
        };

        res.push(utf8_c);
    }

    Ok(res)
}

fn map_char_to_sjis(c: char) -> Option<u16> {
    if c < '\u{0020}' {
        return Some(c as u16);
    }

    if c >= '\u{10000}' {
        return None;
    }
    let c = c as u16;
    let lo = (c & 0x1f) as usize;
    let hi = (c >> 5) as usize;

    let block_index = UNICODE_SJIS_COARSE_MAP[hi];
    if block_index < 0 {
        return None;
    }

    let mapped_char = UNICODE_SJIS_FINE_MAP[block_index as usize][lo];
    if mapped_char == 0 {
        return None;
    }

    Some(mapped_char)
}

/// Calculate the size of a string in Shift-JIS
pub fn measure_sjis_string(s: &str) -> io::Result<usize> {
    let mut result = 0;

    for c in s.chars() {
        let sjis = map_char_to_sjis(c).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unmappable char: {} (U+{:04X})", c, c as u32),
            )
        })?;

        match sjis {
            0x00..=0xff => {
                // single-byte
                result += 1;
            }
            0x100..=0xffff => {
                // double-byte
                result += 2;
            }
            // work around rust-intellij bug
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        }
    }

    Ok(result)
}

pub fn measure_sjis_zstring(s: &str) -> io::Result<usize> {
    let mut result = measure_sjis_string(s)?;
    result += 1;
    Ok(result)
}

pub fn encode_sjis_string<'bump>(
    bump: &'bump Bump,
    s: &str,
    fixup: bool,
) -> io::Result<bumpalo::collections::Vec<'bump, u8>> {
    let mut output = bumpalo::collections::Vec::with_capacity_in(s.len(), bump);

    for c in s.chars() {
        // NOTE: the game impl emits ※ (81A6 in Shift-JIS) for unmappable chars
        // we are more conservative and just error out
        let mut sjis = map_char_to_sjis(c).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unmappable char: {} (U+{:04X})", c, c as u32),
            )
        })?;

        // apply fixup
        // TODO: this might be slow
        if fixup {
            if let Some(position) = SJIS_FIXUP_ENTRIES.iter().position(|&c| c == sjis) {
                sjis = (KATAKANA_START + position as u8) as u16;
            }
        }

        match sjis {
            0x00..=0xff => {
                // single-byte
                output.push(sjis as u8);
            }
            0x100..=0xffff => {
                // double-byte
                let hi = (sjis >> 8) as u8;
                let lo = (sjis & 0xff) as u8;
                output.push(hi);
                output.push(lo);
            }
            // work around rust-intellij bug
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        }
    }

    Ok(output)
}

/// Encode a string in Shift-JIS
pub fn encode_sjis_zstring<'bump>(
    bump: &'bump Bump,
    s: &str,
    fixup: bool,
) -> io::Result<bumpalo::collections::Vec<'bump, u8>> {
    let mut output = encode_sjis_string(bump, s, fixup)?;

    output.push(0);

    Ok(output)
}

#[cfg(test)]
mod tests {
    #[allow(unused)]
    use super::*;

    #[test]
    fn test_sjis() {
        let bump = Bump::new();
        let s = b"\x82\xa0\x82\xa2\x82\xa4\x82\xa6\x82\xa8\x00";
        let s = decode_sjis_zstring(&bump, s, false).unwrap();
        assert_eq!(s, "あいうえお");
        let encoded = encode_sjis_zstring(&bump, &s, false).unwrap();
        assert_eq!(encoded, b"\x82\xa0\x82\xa2\x82\xa4\x82\xa6\x82\xa8\x00");
    }
}
