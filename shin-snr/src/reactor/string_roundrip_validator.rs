use std::iter;

use bumpalo::{collections::Vec, Bump};
use owo_colors::OwoColorize;
use shin_text::{
    decode_sjis_zstring, detect_fixup, encode_sjis_zstring, FixupDetectResult, StringArrayIter,
};
use shin_versions::{MessageCommandStyle, MessageFixupPolicy};
use unicode_width::UnicodeWidthChar as _;

use crate::{
    layout::message_parser::MessageReflowMode,
    reactor::{AnyStringSource, Reactor, StringArraySource, StringSource},
    reader::Reader,
};

pub struct StringRoundtripValidatorReactor<'a> {
    snr_style: MessageCommandStyle,
    user_style: MessageCommandStyle,
    policy: MessageFixupPolicy,
    reader: Reader<'a>,
    bump: Bump,
}

impl<'a> StringRoundtripValidatorReactor<'a> {
    pub fn new(
        snr_style: MessageCommandStyle,
        user_style: MessageCommandStyle,
        policy: MessageFixupPolicy,
        reader: Reader<'a>,
    ) -> Self {
        Self {
            snr_style,
            user_style,
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

    if policy.len() != decoded.chars().count() {
        panic!("policy length mismatch");
    }
}

fn roundrip_string(
    bump: &Bump,
    s: &[u8],
    snr_style: MessageCommandStyle,
    user_style: MessageCommandStyle,
    policy: MessageFixupPolicy,
    source: AnyStringSource,
) {
    let decoded = decode_sjis_zstring(bump, s, source.contains_commands()).unwrap();

    if decoded.contains(|v| shin_text::UNFIXED_UP_CHARACTERS.contains(&v)) {
        panic!(
            "decoded string contains unfixed-up characters: {:?}",
            decoded
        );
    }

    let mut fixup_map = Vec::with_capacity_in(s.len(), bump);
    detect_fixup(s, &mut fixup_map).unwrap();

    // need to do two transforms to simulate what running a full roundtrip would do (from in_style into out_style and back)
    // first transform into what the user would see
    let user_transformed =
        crate::layout::message_parser::transform(bump, decoded, snr_style, user_style, source);

    // and then transform back into what the game would see
    let (game_transformed, fixup_policy) =
        crate::layout::message_parser::transform_reflow_and_infer_fixup_policy(
            bump,
            user_transformed,
            user_style,
            MessageReflowMode::NoReflow,
            snr_style,
            policy,
            FixupDetectResult::merge_all(&fixup_map),
            source,
        );
    validate_fixup_policy(game_transformed, fixup_map.as_slice(), fixup_policy);
    assert_eq!(decoded, game_transformed);

    let reencoded = encode_sjis_zstring(bump, game_transformed, fixup_policy).unwrap();

    if s != reencoded {
        format_mismatch(s, reencoded, decoded);
    }
}

fn roundrip_string_array(
    bump: &Bump,
    ss: &[u8],
    snr_style: MessageCommandStyle,
    user_style: MessageCommandStyle,
    policy: MessageFixupPolicy,
    source: StringArraySource,
) {
    for (i, s) in (0..).zip(StringArrayIter::new(ss)) {
        roundrip_string(
            bump,
            s,
            snr_style,
            user_style,
            policy,
            AnyStringSource::Array(source, i),
        );
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
        roundrip_string(
            &self.bump,
            s,
            self.snr_style,
            self.user_style,
            self.policy,
            AnyStringSource::Singular(source),
        )
    }

    fn u16string(&mut self, source: StringSource) {
        let s = self.reader.u16string();
        roundrip_string(
            &self.bump,
            s,
            self.snr_style,
            self.user_style,
            self.policy,
            AnyStringSource::Singular(source),
        )
    }

    fn u8string_array(&mut self, source: StringArraySource) {
        let ss = self.reader.u8string_array();
        roundrip_string_array(
            &self.bump,
            ss,
            self.snr_style,
            self.user_style,
            self.policy,
            source,
        )
    }

    fn u16string_array(&mut self, source: StringArraySource) {
        let ss = self.reader.u16string_array();
        roundrip_string_array(
            &self.bump,
            ss,
            self.snr_style,
            self.user_style,
            self.policy,
            source,
        )
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
