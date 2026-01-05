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

pub const UNFIXED_UP_CHARACTERS: [char; 64] = KATAKANA_TABLE;

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
    // FIXME: not actually safe, since the table's size is 11468 < 11567 :/
    // unsafe { *JIS_TABLE.get_unchecked(addr) }
    JIS_TABLE[addr]
}

fn is_extended(c: u8) -> bool {
    matches!(c, 0x81..=0x9f | 0xe0..=0xfc)
}

pub fn decode_sjis_codepoint(codepoint: u16, fixup: bool) -> char {
    if codepoint < 0x100 {
        // single-byte
        let codepoint = codepoint as u8;
        assert!(!is_extended(codepoint));
        decode_single_sjis_char(codepoint, fixup)
    } else {
        // double-byte
        let [first, second] = codepoint.to_be_bytes();
        assert!(is_extended(first));
        decode_double_sjis_char(first, second)
    }
}

/// Decode a Shift-JIS encoded string to UTF-8
///
/// Despite the zstring in the name, the function can work with either zero-terminated or non-zero terminated Shift-JIS strings.
/// It will simply ignore all zero bytes at the end of the string.
///
/// If `fixup` is true, an alternative table will be used to decode half-width katakana characters, reading them as full-width hiragana instead.
pub fn decode_sjis_zstring<'bump>(
    bump: &'bump Bump,
    mut s: &[u8],
    fixup: bool,
) -> io::Result<&'bump str> {
    // TODO: maybe there is a better capacity estimation
    let mut res = bumpalo::collections::String::with_capacity_in(s.len(), bump);

    while s.last() == Some(&0) {
        s = &s[..s.len() - 1];
    }

    let mut b = s.iter().copied();
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

    Ok(res.into_bump_str())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixupDetectResult {
    /// No fix-uppable characters were detected, so can't determine if fixup was used
    NoFixupCharacters,
    /// A fix-uppable character was detected and it was fixed up
    FixedUp,
    /// A fix-uppable character was detected and it was not fixed up
    UnfixedUp,
    /// Some fix-uppable characters were fixed up and some were not
    Inconsistent,
}

impl FixupDetectResult {
    pub fn merge_all(results: &[FixupDetectResult]) -> Self {
        let mut result = Self::NoFixupCharacters;
        for &r in results {
            result.merge(r);
        }
        result
    }

    pub fn merge(&mut self, other: FixupDetectResult) {
        match (*self, other) {
            (Self::NoFixupCharacters, something) => {
                *self = something;
            }
            (Self::FixedUp, Self::UnfixedUp) | (Self::UnfixedUp, Self::FixedUp) => {
                *self = Self::Inconsistent;
            }
            _ => {
                // do nothing
            }
        }
    }

    pub fn resolve(self) -> Option<bool> {
        match self {
            FixupDetectResult::NoFixupCharacters | FixupDetectResult::UnfixedUp => Some(false),
            FixupDetectResult::FixedUp => Some(true),
            FixupDetectResult::Inconsistent => None,
        }
    }
}

pub trait DetectFixupSink {
    fn merge(&mut self, other: FixupDetectResult);
}

impl DetectFixupSink for FixupDetectResult {
    fn merge(&mut self, other: FixupDetectResult) {
        FixupDetectResult::merge(self, other);
    }
}

impl DetectFixupSink for bumpalo::collections::Vec<'_, FixupDetectResult> {
    fn merge(&mut self, other: FixupDetectResult) {
        self.push(other);
    }
}

/// Try to detect, per-character, whether fixup encoding was used.
///
/// If `sink` is a bumpalo `Vec`, then produces per-character results. If the sink is `FixupDetectResult`, then it will produce a single judgement on the full string.
pub fn detect_fixup<S: DetectFixupSink>(mut s: &[u8], sink: &mut S) -> io::Result<()> {
    while s.last() == Some(&0) {
        s = &s[..s.len() - 1];
    }

    let mut b = s.iter().copied();
    while let Some(c1) = b.next() {
        let is_double = is_extended(c1);

        let char_result = if is_double {
            let c2 = b.next().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "unexpected end of string when reading double-byte char",
                )
            })?;
            let c = ((c1 as u16) << 8) | c2 as u16;
            if SJIS_FIXUP_ENTRIES.contains(&c) {
                // double character from the fixup range
                FixupDetectResult::UnfixedUp
            } else {
                FixupDetectResult::NoFixupCharacters
            }
        } else if (KATAKANA_START..KATAKANA_END).contains(&c1) {
            // single-char character from the fixup range
            FixupDetectResult::FixedUp
        } else {
            FixupDetectResult::NoFixupCharacters
        };

        sink.merge(char_result);
    }

    Ok(())
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
// TODO: this is missing fixup support
pub fn measure_sjis_zstring(s: &str) -> io::Result<usize> {
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

    // don't forget the zero terminator
    Ok(result + 1)
}

pub trait FixupEncodePolicy {
    fn should_fixup(&mut self) -> bool;
}

impl FixupEncodePolicy for bool {
    fn should_fixup(&mut self) -> bool {
        *self
    }
}

impl FixupEncodePolicy for &[bool] {
    fn should_fixup(&mut self) -> bool {
        let &[result, ref rest @ ..] = *self else {
            panic!("not enough elements in the fixup policy array");
        };
        *self = rest;

        result
    }
}

pub struct IterFixupEncodePolicy<T>(pub T);

impl<T> FixupEncodePolicy for IterFixupEncodePolicy<T>
where
    T: Iterator<Item = bool>,
{
    fn should_fixup(&mut self) -> bool {
        self.0.next().unwrap_or_else(|| {
            panic!("not enough elements in the fixup policy iterator");
        })
    }
}

pub fn encode_sjis_codepoint(codepoint: char, fixup: bool) -> Option<u16> {
    let mut sjis = map_char_to_sjis(codepoint)?;

    if fixup {
        if let Some(position) = SJIS_FIXUP_ENTRIES.iter().position(|&c| c == sjis) {
            sjis = (KATAKANA_START + position as u8) as u16;
        }
    }

    Some(sjis)
}

/// Encode a UTF-8 string into a zero-terminated Shift-JIS string
///
/// The `fixup` argument is a policy used to decide per-character whether to use an alternative encoding for hiragana characters or not.
///
/// This is necessary because when fixup encoding is in use there is a redundancy: the hiragana characters can either be encoded as double-byte Shift-JIS characters or as single-byte Shift-JIS characters in the range 0xA0-0xDF.
/// The authoring tools are somewhat sloppy with this and do end up using both encodings in the same string, so to byte-for-byte reproducibility we need to expose this toggle on character level.
pub fn encode_sjis_zstring<'bump, P: FixupEncodePolicy>(
    bump: &'bump Bump,
    s: &str,
    mut fixup: P,
) -> io::Result<&'bump [u8]> {
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
        // TODO: linearly looking through 64 entries might be too much..
        // maybe add a smartass binary search or something like that?
        if fixup.should_fixup() {
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

    output.push(0);

    Ok(output.into_bump_slice())
}

#[derive(Debug, Clone)]
pub struct StringArrayIter<'a> {
    v: &'a [u8],
}

impl<'a> StringArrayIter<'a> {
    pub fn new(string_array: &'a [u8]) -> Self {
        Self { v: string_array }
    }
}

impl<'a> Iterator for StringArrayIter<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.v.is_empty() {
            return None;
        }

        let idx = self
            .v
            .iter()
            .position(|&x| x == 0)
            .map(|idx| idx + 1)
            .expect("Invalid string array: ended without an explicit zero terminator");
        let (result, rest) = self.v.split_at(idx);
        self.v = rest;

        debug_assert!(!result.is_empty());
        debug_assert_eq!(result.last(), Some(&0));

        // empty string is a signal of the end of the array
        if result == b"\x00" {
            if !rest.is_empty() {
                panic!("Invalid string array: zero terminator not at the end of the array");
            }
            None
        } else {
            Some(result)
        }
    }
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
