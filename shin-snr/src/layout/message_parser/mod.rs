pub mod lint;
pub mod sink;

use std::str::CharIndices;

use bumpalo::{
    Bump,
    collections::{String, Vec},
    vec,
};
use shin_font::FontMetrics;
use shin_text::FixupDetectResult;
use shin_versions::{MessageCommandStyle, SjisMessageFixupPolicy};
use sink::{CountingStringSink, FullStringSink, StringSink, TokenSink};

use crate::{layout::layouter::GameLayoutInfo, reactor::AnyStringSource};

#[derive(Copy, Clone, Debug)]
pub enum ParseIntArgError {
    NoArgument,
    InvalidArgument,
}
#[derive(Copy, Clone, Debug)]
pub struct CommandToken<'b> {
    pub command: char,
    pub argument: Option<&'b str>,
}

impl<'b> CommandToken<'b> {
    pub fn parse_int_arg(self) -> Result<u32, ParseIntArgError> {
        let Some(arg) = self.argument else {
            return Err(ParseIntArgError::NoArgument);
        };
        // NOTE: newer engine versions support HEX arguments here
        // older version is extremely permissive and just does `-0x30` to the s-jis codepoint without any validation
        // we currently only support the lowest common denominator:
        // only decimals and explicit error on anything else (unlike the original code that returns -1)
        let Ok(arg) = arg.parse::<u32>() else {
            return Err(ParseIntArgError::InvalidArgument);
        };
        Ok(arg)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MessageToken<'b> {
    Literal(char),
    Command(CommandToken<'b>),
}

pub struct SpannedMessageToken<'b> {
    pub start: usize,
    pub end: usize,
    pub token: MessageToken<'b>,
}

#[derive(Debug, Clone, Copy)]
pub enum MessageCommand {
    /// @+
    EnableLipsync,
    /// @-
    DisableLipsync,
    /// @/
    VoiceWait,
    /// @<
    RubiBaseStart,
    /// @>
    RubiBaseEnd,
    /// @a
    SetFade,
    /// @b
    RubiContent,
    /// @c
    SetColor,
    /// @e
    NoFinalClickWait,
    /// @k
    ClickWait,
    /// @o
    VoiceVolume,
    /// @r
    Newline,
    /// @s
    TextSpeed,
    /// @t
    StartParallel,
    /// @u
    Unicode,
    /// @v
    Voice,
    /// @w
    Wait,
    /// @x
    VoiceSync,
    /// @y
    Sync,
    /// @z
    FontScale,
    /// @|
    CompleteSection,
    /// @[
    InstantTextStart,
    /// @]
    InstantTextEnd,
    /// @{
    BoldTextStart,
    /// @}
    BoldTextEnd,
}

impl MessageCommand {
    pub fn parse(c: char) -> Option<Self> {
        Some(match c {
            '+' => Self::EnableLipsync,
            '-' => Self::DisableLipsync,
            '/' => Self::VoiceWait,
            '<' => Self::RubiBaseStart,
            '>' => Self::RubiBaseEnd,
            'a' => Self::SetFade,
            'b' => Self::RubiContent,
            'c' => Self::SetColor,
            'e' => Self::NoFinalClickWait,
            'k' => Self::ClickWait,
            'o' => Self::VoiceVolume,
            'r' => Self::Newline,
            's' => Self::TextSpeed,
            't' => Self::StartParallel,
            'u' => Self::Unicode,
            'v' => Self::Voice,
            'w' => Self::Wait,
            'x' => Self::VoiceSync,
            'y' => Self::Sync,
            'z' => Self::FontScale,
            '|' => Self::CompleteSection,
            '[' => Self::InstantTextStart,
            ']' => Self::InstantTextEnd,
            '{' => Self::BoldTextStart,
            '}' => Self::BoldTextEnd,
            _ => return None,
        })
    }

    pub fn into_char(self) -> char {
        match self {
            Self::EnableLipsync => '+',
            Self::DisableLipsync => '-',
            Self::VoiceWait => '/',
            Self::RubiBaseStart => '<',
            Self::RubiBaseEnd => '>',
            Self::SetFade => 'a',
            Self::RubiContent => 'b',
            Self::SetColor => 'c',
            Self::NoFinalClickWait => 'e',
            Self::ClickWait => 'k',
            Self::VoiceVolume => 'o',
            Self::Newline => 'r',
            Self::TextSpeed => 's',
            Self::StartParallel => 't',
            Self::Unicode => 'u',
            Self::Voice => 'v',
            Self::Wait => 'w',
            Self::VoiceSync => 'x',
            Self::Sync => 'y',
            Self::FontScale => 'z',
            Self::CompleteSection => '|',
            Self::InstantTextStart => '[',
            Self::InstantTextEnd => ']',
            Self::BoldTextStart => '{',
            Self::BoldTextEnd => '}',
        }
    }

    pub fn has_arg(self) -> bool {
        match self {
            Self::EnableLipsync => false,
            Self::DisableLipsync => false,
            Self::VoiceWait => false,
            Self::RubiBaseStart => false,
            Self::RubiBaseEnd => false,
            Self::SetFade => true,
            Self::RubiContent => true,
            Self::SetColor => true,
            Self::NoFinalClickWait => false,
            Self::ClickWait => false,
            Self::VoiceVolume => true,
            Self::Newline => false,
            Self::TextSpeed => true,
            Self::StartParallel => false,
            Self::Unicode => true,
            Self::Voice => true,
            Self::Wait => true,
            Self::VoiceSync => false,
            Self::Sync => false,
            Self::FontScale => true,
            Self::CompleteSection => false,
            Self::InstantTextStart => false,
            Self::InstantTextEnd => false,
            Self::BoldTextStart => false,
            Self::BoldTextEnd => false,
        }
    }
}

pub fn serialize<S>(
    style: MessageCommandStyle,
    policy: SjisMessageFixupPolicy,
    is_in_messagebox: bool,
    tokens: &[MessageToken],
    sink: &mut S,
) where
    S: StringSink,
{
    let mut finished_first_line = false;

    for token in tokens {
        match *token {
            MessageToken::Literal(lit) => {
                match style {
                    MessageCommandStyle::Escaped if lit == '@' => {
                        // a literal `@` needs escaping
                        sink.push('@', false);
                        sink.push('@', false);
                    }
                    // the actual game checks `lit < 256` here, but
                    // 1. it operates on Shift-JIS codepoints
                    // 2. it remaps the single-byte half-width katakana to full-width hiragana
                    // so the only codepoints that will pass the `< 256` are the unchanged basic ASCII characters
                    // which, in unicode, corresponds to the range `0x00..=0x7F`
                    MessageCommandStyle::Unescaped if lit.is_ascii() => {
                        // an ascii literal needs escaping with `!`
                        sink.push('!', false);
                        sink.push(lit, false);
                    }
                    _ => {
                        sink.push(
                            lit,
                            if !is_in_messagebox || policy.fixup_character_names {
                                true
                            } else {
                                finished_first_line
                            },
                        );
                    }
                }
            }
            MessageToken::Command(CommandToken { command, argument }) => {
                match style {
                    MessageCommandStyle::Escaped => {
                        sink.push('@', false);
                        sink.push(command, false);
                    }
                    MessageCommandStyle::Unescaped => {
                        sink.push(command, false);
                    }
                }
                if let Some(argument) = argument {
                    sink.push_str(argument, policy.fixup_command_arguments);
                    sink.push('.', false);
                }

                if let Some(MessageCommand::Newline) = MessageCommand::parse(command) {
                    finished_first_line = true;
                }
            }
        }
    }
}

pub fn serialize_string<'bump>(
    bump: &'bump Bump,
    style: MessageCommandStyle,
    tokens: &[MessageToken],
) -> &'bump str {
    // when we don't care about the fixup map, the policy doesn't matter
    // make up some random one
    let policy = SjisMessageFixupPolicy {
        fixup_command_arguments: false,
        fixup_character_names: false,
    };

    let mut counting_sink = CountingStringSink::new();

    serialize(style, policy, false, tokens, &mut counting_sink);

    let mut string_sink = String::with_capacity_in(counting_sink.utf8_byte_count, bump);

    serialize(style, policy, false, tokens, &mut string_sink);

    string_sink.into_bump_str()
}

pub fn serialize_full<'bump>(
    bump: &'bump Bump,
    style: MessageCommandStyle,
    policy: SjisMessageFixupPolicy,
    is_in_messagebox: bool,
    tokens: &[MessageToken],
) -> (&'bump str, &'bump [bool]) {
    let mut counting_sink = CountingStringSink::new();

    serialize(style, policy, is_in_messagebox, tokens, &mut counting_sink);

    // bump is very bad with interleaving allocations from multiple collections
    // so make sure to reserve the correct amount of space beforehand
    let mut full_sink = FullStringSink::new(
        bump,
        counting_sink.char_count,
        counting_sink.utf8_byte_count,
    );

    serialize(style, policy, is_in_messagebox, tokens, &mut full_sink);

    (
        full_sink.string.into_bump_str(),
        full_sink.fixup.into_bump_slice(),
    )
}

pub fn parse<'bump, S: TokenSink<'bump>>(
    style: MessageCommandStyle,
    message: &'bump str,
    sink: &mut S,
) {
    struct Parse<'b> {
        iter: CharIndices<'b>,
        can_have_dot: bool,
    }

    impl<'b> Parse<'b> {
        fn read_argument(&mut self) -> Option<&'b str> {
            if self.can_have_dot {
                let mut iter = self.iter.clone();
                let start = iter.offset();
                while let Some((ofs, c)) = iter.next() {
                    if c == '.' {
                        let arg = &self.iter.as_str()[..ofs - start];
                        self.iter = iter;
                        return Some(arg);
                    }
                }
                self.can_have_dot = false;
                None
            } else {
                None
            }
        }

        fn next(&mut self) -> Option<(usize, char)> {
            self.iter.next()
        }

        fn position(&self) -> usize {
            self.iter.offset()
        }
    }

    let mut parse = Parse {
        iter: message.char_indices(),
        can_have_dot: true,
    };

    while let Some((start, c)) = parse.next() {
        match style {
            MessageCommandStyle::Escaped => {
                if c == '@' {
                    let Some((_, c)) = parse.next() else {
                        todo!("handle unmatched @");
                    };

                    if c == '@' {
                        sink.push(start, parse.position(), MessageToken::Literal('@'));
                    } else {
                        let has_argument = MessageCommand::parse(c).is_some_and(|c| c.has_arg());
                        let argument = has_argument.then(|| parse.read_argument()).flatten();
                        sink.push(
                            start,
                            parse.position(),
                            MessageToken::Command(CommandToken {
                                command: c,
                                argument,
                            }),
                        );
                    }
                } else {
                    sink.push(start, parse.position(), MessageToken::Literal(c));
                }
            }
            MessageCommandStyle::Unescaped => {
                if (c as u16) < 128 {
                    if c == '!' {
                        // the game doesn't check end-of-line here too,
                        // so this is almost 100% invalid string that we will never encounter in the wild
                        // and it's fine to just unwrap
                        // maybe it's actually not ideal since our end users can supply such strings...
                        let (_, c) = parse.next().unwrap();
                        sink.push(start, parse.position(), MessageToken::Literal(c));
                    } else {
                        // NOTE: we handle invalid commands the same way the engine would: by ignoring them
                        // the only difference is that we can recognize more commands than the engine for older version of the engine
                        // (they didn't have the `/`, `t` and `u`)
                        let has_argument = MessageCommand::parse(c).is_some_and(|c| c.has_arg());
                        let argument = has_argument.then(|| parse.read_argument()).flatten();
                        sink.push(
                            start,
                            parse.position(),
                            MessageToken::Command(CommandToken {
                                command: c,
                                argument,
                            }),
                        );
                    }
                } else {
                    sink.push(start, parse.position(), MessageToken::Literal(c));
                }
            }
        }
    }
}

pub fn infer_string_fixup_policy<'bump>(
    bump: &'bump Bump,
    decoded: &str,
    style: MessageCommandStyle,
    message_policy: SjisMessageFixupPolicy,
    detected: FixupDetectResult,
    source: AnyStringSource,
) -> &'bump [bool] {
    // some messages are just not fixed up, even though it makes sense to do so
    // in higurashi sui this happens with messages from the debug menu
    if detected == FixupDetectResult::UnfixedUp {
        return vec![in bump; false; decoded.chars().count()].into_bump_slice();
    }

    if source.contains_commands() {
        let mut tokens = Vec::new_in(bump);
        parse(style, decoded, &mut tokens);
        let (serialized_string, fixup_policy) = serialize_full(
            bump,
            style,
            message_policy,
            source.is_for_messagebox(),
            &tokens,
        );
        assert_eq!(serialized_string, decoded);
        fixup_policy
    } else {
        vec![in bump; false; decoded.chars().count()].into_bump_slice()
    }
}

#[derive(Default, Clone, Copy)]
pub enum MessageReflowMode<'a> {
    /// Do not attempt to reflow text to introduce additional line breaks.
    #[default]
    NoReflow,
    /// Use a greedy algorithm to insert hardbreaks (@r) to correctly word-wrap western text.
    ///
    /// This uses UAX#14 to identify possible line break points.
    Greedy {
        metrics: &'a FontMetrics,
        layout: GameLayoutInfo,
    },
}

/// Combines [`transform_reflow`] and [`infer_string_fixup_policy`] into a single pass.
///
/// This is more efficient that doing those separately, since it only parses the message once.
pub fn transform_reflow_and_infer_fixup_policy<'bump>(
    bump: &'bump Bump,
    decoded: &'bump str,
    in_style: MessageCommandStyle,
    reflow: MessageReflowMode,
    out_style: MessageCommandStyle,
    has_useless_escapes: bool,
    message_policy: SjisMessageFixupPolicy,
    detected: FixupDetectResult,
    source: AnyStringSource,
) -> (&'bump str, &'bump [bool]) {
    if source.contains_commands() {
        let mut tokens = Vec::new_in(bump);
        parse(in_style, decoded, &mut tokens);

        if source.is_for_messagebox() {
            if let MessageReflowMode::Greedy { metrics, layout } = reflow {
                let mut tokens_out = Vec::new_in(bump);
                super::reflow::reflow_message(bump, metrics, layout, &tokens, &mut tokens_out);
                tokens = tokens_out;
            }
        }

        let (serialized_string, fixup_policy) = if detected == FixupDetectResult::UnfixedUp {
            // if the original string ignored fixups, hijack the policy to do the same
            let serialized = serialize_string(bump, out_style, &tokens);

            (
                serialized,
                vec![in bump; false; serialized.chars().count()].into_bump_slice(),
            )
        } else {
            serialize_full(
                bump,
                out_style,
                message_policy,
                source.is_for_messagebox(),
                &tokens,
            )
        };

        if in_style == out_style
            && let MessageReflowMode::NoReflow = reflow
        {
            if !has_useless_escapes {
                // just a sanity check
                assert_eq!(serialized_string, decoded);
            }
        }

        (serialized_string, fixup_policy)
    } else {
        // do not touch strings that don't have commands
        (
            decoded,
            vec![in bump; false; decoded.chars().count()].into_bump_slice(),
        )
    }
}

pub fn transform_reflow<'bump>(
    bump: &'bump Bump,
    decoded: &'bump str,
    in_style: MessageCommandStyle,
    reflow: MessageReflowMode,
    out_style: MessageCommandStyle,
    has_useless_escapes: bool,
    source: AnyStringSource,
) -> &'bump str {
    if source.contains_commands() {
        let mut tokens = Vec::new_in(bump);
        parse(in_style, decoded, &mut tokens);

        if source.is_for_messagebox() {
            if let MessageReflowMode::Greedy { metrics, layout } = reflow {
                let mut tokens_out = Vec::new_in(bump);
                super::reflow::reflow_message(bump, metrics, layout, &tokens, &mut tokens_out);
                tokens = tokens_out;
            }
        }

        let serialized_string = serialize_string(bump, out_style, &tokens);

        if in_style == out_style {
            // WorldRe has a string with needless escapes, triggering this assert
            // This is the problematic string: `r　両目を見開き、こんな感じの顔をした→!(!？Д!？!)`
            if !has_useless_escapes {
                // just a sanity check
                assert_eq!(serialized_string, decoded);
            }
            return decoded;
        }
        serialized_string
    } else {
        decoded
    }
}
