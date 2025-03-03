use std::iter;

use bumpalo::{collections::Vec, vec, Bump};
use owo_colors::OwoColorize;
use shin_text::{decode_sjis_zstring, detect_fixup, encode_sjis_zstring, FixupDetectResult};
use shin_versions::{MessageCommandStyle, MessageFixupPolicy};
use unicode_width::UnicodeWidthChar as _;

use crate::{
    reactor::{Reactor, StringArraySource, StringSource},
    reader::Reader,
};

pub struct StringRoundtripValidatorReactor<'a> {
    style: MessageCommandStyle,
    policy: MessageFixupPolicy,
    reader: Reader<'a>,
    bump: Bump,
}

impl<'a> StringRoundtripValidatorReactor<'a> {
    pub fn new(style: MessageCommandStyle, policy: MessageFixupPolicy, reader: Reader<'a>) -> Self {
        Self {
            style,
            policy,
            reader,
            bump: Bump::new(),
        }
    }
}

fn format_mismatch(expected: &[u8], found: &[u8], decoded: &str) {
    fn make_iter(bytes: &[u8]) -> impl Iterator<Item = Option<u8>> + use<'_> {
        bytes.iter().copied().map(Some).chain(iter::repeat(None))
    }

    let bytes = make_iter(expected).zip(make_iter(found));

    let mut subline1 = String::new();
    let mut subline2 = String::new();
    for (b1, b2) in bytes {
        match (b1, b2) {
            (Some(b1), Some(b2)) => {
                let hex1 = format!("{:02x}", b1);
                let hex2 = format!("{:02x}", b2);
                if b1 != b2 {
                    subline1 = format!("{} {}", subline1, hex1.red());
                    subline2 = format!("{} {}", subline2, hex2.green());
                } else {
                    subline1 = format!("{} {}", subline1, hex1);
                    subline2 = format!("{} {}", subline2, hex2);
                }
            }
            (Some(b1), None) => {
                let hex1 = format!("{:02x}", b1);
                subline1 = format!("{} {}", subline1, hex1.red());
                subline2 = format!("{}   ", subline2);
            }
            (None, Some(b2)) => {
                let hex2 = format!("{:02x}", b2);
                subline1 = format!("{}   ", subline1);
                subline2 = format!("{} {}", subline2, hex2.green());
            }
            (None, None) => break,
        }
    }
    println!(
        "roundtrip mismatch:\n|{}\n|{}\n|{}\n",
        decoded, subline1, subline2
    );
    panic!("roundtrip mismatch");
}

fn format_invalid_policy(decoded: &str, map: &[FixupDetectResult], policy: &[bool]) {
    let subline1 = decoded;
    let mut subline2 = String::new();
    let mut subline3 = String::new();
    for ((c, &r), &p) in decoded.chars().zip(map).zip(policy) {
        let m = match r {
            FixupDetectResult::NoFixupCharacters => ' '.to_string(),
            FixupDetectResult::FixedUp => 'F'.blue().to_string(),
            FixupDetectResult::UnfixedUp => 'U'.yellow().to_string(),
            FixupDetectResult::Inconsistent => unreachable!(),
        };
        subline2.push_str(&m);
        if c.width().unwrap() == 2 {
            subline2.push_str(&m.to_ascii_lowercase());
        }

        let p = if p {
            'F'.blue().to_string()
        } else {
            'U'.yellow().to_string()
        };
        subline3.push_str(&p);
        if c.width().unwrap() == 2 {
            subline3.push_str(&p.to_ascii_lowercase());
        }
    }

    println!(
        "invalid fixup policy:\n|{}\n|{}\n|{}\n",
        subline1, subline2, subline3
    );
    panic!("invalid fixup policy");
}
fn validate_fixup_policy(decoded: &str, detection_map: &[FixupDetectResult], policy: &[bool]) {
    let mut valid = true;

    for (detection, &policy) in detection_map.iter().zip(policy) {
        match detection {
            FixupDetectResult::FixedUp if !policy => {
                valid = false;
            }
            FixupDetectResult::UnfixedUp if policy => {
                valid = false;
            }
            FixupDetectResult::Inconsistent => unreachable!(),
            _ => {}
        }
    }

    if !valid {
        format_invalid_policy(decoded, detection_map, policy);
    }
}

fn roundrip_string(
    bump: &Bump,
    s: &[u8],
    style: MessageCommandStyle,
    policy: MessageFixupPolicy,
    source: StringSource,
) {
    let decoded = decode_sjis_zstring(bump, s, source.fixup_on_decode()).unwrap();

    let mut fixup_map = Vec::with_capacity_in(s.len(), bump);
    detect_fixup(s, &mut fixup_map).unwrap();
    let fixup_policy = crate::message_parser::infer_string_fixup_policy(
        bump,
        &decoded,
        style,
        policy,
        FixupDetectResult::merge_all(&fixup_map),
        source,
    );
    validate_fixup_policy(&decoded, fixup_map.as_slice(), fixup_policy);

    let reencoded = encode_sjis_zstring(bump, &decoded, fixup_policy).unwrap();

    if s != reencoded.as_slice() {
        format_mismatch(s, reencoded.as_slice(), &decoded);
    }
}

fn roundrip_string_array(
    bump: &Bump,
    ss: &[u8],
    _style: MessageCommandStyle,
    _policy: MessageFixupPolicy,
    source: StringArraySource,
) {
    let mut p = ss;
    while p.last() == Some(&0) {
        p = &p[..p.len() - 1];
    }

    // TODO: infer_string_fixup_policy needs support for sourcing string arrays if we ever encounter string arrays that need fixup
    assert_eq!(source.fixup_on_decode(), false);

    let array_iter = p.split(|&v| v == 0);
    let array_size = array_iter.clone().count();

    let mut decoded = Vec::with_capacity_in(array_size, bump);
    for s in array_iter.clone() {
        let s = decode_sjis_zstring(bump, s, false).unwrap();
        decoded.push(s);
    }

    let mut fixup_policies = Vec::with_capacity_in(array_size, bump);
    for (_s, decoded) in array_iter.clone().zip(&decoded) {
        let policy = vec![in bump; false; decoded.chars().count()];
        fixup_policies.push(policy);
    }

    let mut encoded = Vec::with_capacity_in(ss.len(), bump);
    for (decoded_item, fixup_policy) in decoded.iter().zip(&fixup_policies) {
        let encoded_item =
            encode_sjis_zstring(bump, decoded_item, fixup_policy.as_slice()).unwrap();
        encoded.extend_from_slice(&encoded_item);
    }
    encoded.push(0);

    if ss != encoded.as_slice() {
        format_mismatch(ss, encoded.as_slice(), &format!("{:?}", decoded));
    }
}

impl<'a> Reactor for StringRoundtripValidatorReactor<'a> {
    fn byte(&mut self) -> u8 {
        self.reader.byte()
    }

    fn short(&mut self) -> u16 {
        self.reader.short()
    }

    fn uint(&mut self) -> u32 {
        self.reader.uint()
    }

    fn reg(&mut self) {
        self.reader.reg();
    }

    fn offset(&mut self) {
        self.reader.offset();
    }

    fn u8string(&mut self, source: StringSource) {
        let s = self.reader.u8string();
        roundrip_string(&self.bump, s, self.style, self.policy, source)
    }

    fn u16string(&mut self, source: StringSource) {
        let s = self.reader.u16string();
        roundrip_string(&self.bump, s, self.style, self.policy, source)
    }

    fn u8string_array(&mut self, source: StringArraySource) {
        let ss = self.reader.u8string_array();
        roundrip_string_array(&self.bump, ss, self.style, self.policy, source)
    }

    fn u16string_array(&mut self, source: StringArraySource) {
        let ss = self.reader.u16string_array();
        roundrip_string_array(&self.bump, ss, self.style, self.policy, source)
    }

    fn instr_start(&mut self) {}
    fn instr_end(&mut self) {}

    fn has_instr(&self) -> bool {
        self.reader.has_instr()
    }

    fn in_location(&self) -> u32 {
        self.reader.position()
    }
}
