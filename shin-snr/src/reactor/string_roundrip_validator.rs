use std::iter;

use bumpalo::{Bump, collections::Vec};
use owo_colors::OwoColorize;
use shin_text::{FixupDetectResult, StringArrayIter, detect_fixup, encode_sjis_zstring};
use shin_versions::{MessageCommandStyle, StringPolicy};
use unicode_width::UnicodeWidthChar as _;

use crate::{
    layout::message_parser::MessageReflowMode,
    operation::{
        OperationElementRepr,
        arena::OperationArena,
        schema::{Opcode, OperationSchema},
    },
    reactor::{AnyStringSource, Reactor, StringArraySource, StringSource},
    text::{decode_zstring, encode_utf8_zstring},
};

pub struct StringRoundtripValidatorReactor {
    snr_style: MessageCommandStyle,
    user_style: MessageCommandStyle,
    policy: StringPolicy,
    bump: Bump,
}

impl StringRoundtripValidatorReactor {
    pub fn new(
        snr_style: MessageCommandStyle,
        user_style: MessageCommandStyle,
        policy: StringPolicy,
    ) -> Self {
        Self {
            snr_style,
            user_style,
            policy,
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

impl StringRoundtripValidatorReactor {
    fn roundtrip_string(&self, source: AnyStringSource, string: &[u8]) {
        let decoded = decode_zstring(
            &self.bump,
            self.policy.encoding(),
            string,
            source.contains_commands(),
        )
        .unwrap();

        if decoded.contains(|v| shin_text::UNFIXED_UP_CHARACTERS.contains(&v)) {
            panic!(
                "decoded string contains unfixed-up characters: {:?}",
                decoded
            );
        }

        // need to do two transforms to simulate what running a full roundtrip would do (from in_style into out_style and back)
        // first transform into what the user would see
        let user_transformed = crate::layout::message_parser::transform_reflow(
            &self.bump,
            decoded,
            self.snr_style,
            MessageReflowMode::NoReflow,
            self.user_style,
            source,
        );

        match self.policy {
            StringPolicy::ShiftJis(policy) => {
                let mut fixup_map = Vec::with_capacity_in(string.len(), &self.bump);
                detect_fixup(string, &mut fixup_map).unwrap();

                // and then transform back into what the game would see
                let (game_transformed, fixup_policy) =
                    crate::layout::message_parser::transform_reflow_and_infer_fixup_policy(
                        &self.bump,
                        user_transformed,
                        self.user_style,
                        MessageReflowMode::NoReflow,
                        self.snr_style,
                        policy,
                        FixupDetectResult::merge_all(&fixup_map),
                        source,
                    );
                validate_fixup_policy(game_transformed, fixup_map.as_slice(), fixup_policy);
                assert_eq!(decoded, game_transformed);

                let reencoded =
                    encode_sjis_zstring(&self.bump, game_transformed, fixup_policy).unwrap();

                if string != reencoded {
                    format_mismatch(string, reencoded, decoded);
                }
            }
            StringPolicy::Utf8 => {
                let game_transformed = crate::layout::message_parser::transform_reflow(
                    &self.bump,
                    user_transformed,
                    self.user_style,
                    MessageReflowMode::NoReflow,
                    self.snr_style,
                    source,
                );

                assert_eq!(decoded, game_transformed);

                let reencoded = encode_utf8_zstring(&self.bump, game_transformed);

                assert_eq!(string, reencoded);
            }
        }
    }
}

impl<'a> Reactor for StringRoundtripValidatorReactor {
    fn react(
        &mut self,
        _operation_position: u32,
        _raw_opcode: u8,
        opcode: Opcode,
        op_schema: &OperationSchema,
        arena: &OperationArena,
    ) {
        for element in arena.iter(&op_schema) {
            match element {
                OperationElementRepr::String(_, string) => {
                    let Some(source) = StringSource::for_operation(opcode, op_schema, arena) else {
                        panic!("Could not determine StringSource for opcode {:?}", opcode)
                    };

                    self.roundtrip_string(AnyStringSource::Singular(source), string);
                }
                OperationElementRepr::StringArray(_, string_array) => {
                    let Some(source) = StringArraySource::for_operation(opcode, op_schema, arena)
                    else {
                        panic!(
                            "Could not determine StringArraySource for opcode {:?}",
                            opcode
                        )
                    };

                    for (i, string) in (0..).zip(StringArrayIter::new(string_array)) {
                        self.roundtrip_string(AnyStringSource::Array(source, i), string);
                    }
                }
                _ => {
                    // ignore
                }
            }
        }
    }
}
