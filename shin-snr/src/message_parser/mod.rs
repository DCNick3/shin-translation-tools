pub mod lint;

use std::str::CharIndices;

use bumpalo::{
    collections::{String, Vec},
    vec, Bump,
};
use shin_text::FixupDetectResult;
use shin_versions::{MessageCommandStyle, MessageFixupPolicy};

use crate::reactor::AnyStringSource;

pub struct CommandToken<'b> {
    pub command: char,
    pub argument: Option<&'b str>,
}

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

pub trait StringSink {
    fn push(&mut self, c: char, fixup: bool);
    fn push_str(&mut self, s: &str, fixup: bool);
}

impl<'b> StringSink for String<'b> {
    fn push(&mut self, c: char, _fixup: bool) {
        self.push(c);
    }

    fn push_str(&mut self, s: &str, _fixup: bool) {
        self.push_str(s);
    }
}

pub struct CountingStringSink {
    pub char_count: usize,
    pub utf8_byte_count: usize,
}

impl CountingStringSink {
    pub fn new() -> Self {
        Self {
            char_count: 0,
            utf8_byte_count: 0,
        }
    }
}

impl StringSink for CountingStringSink {
    fn push(&mut self, c: char, _fixup: bool) {
        self.char_count += 1;
        self.utf8_byte_count += c.len_utf8();
    }

    fn push_str(&mut self, s: &str, _fixup: bool) {
        self.char_count += s.chars().count();
        self.utf8_byte_count += s.len();
    }
}

pub struct FullStringSink<'b> {
    pub string: String<'b>,
    pub fixup: Vec<'b, bool>,
}

impl<'b> FullStringSink<'b> {
    pub fn new(bump: &'b Bump, capacity_chars: usize, capacity_bytes: usize) -> Self {
        Self {
            string: String::with_capacity_in(capacity_bytes, bump),
            fixup: Vec::with_capacity_in(capacity_chars, bump),
        }
    }
}

impl StringSink for FullStringSink<'_> {
    fn push(&mut self, c: char, fixup: bool) {
        self.string.push(c);
        self.fixup.push(fixup);
    }

    fn push_str(&mut self, s: &str, fixup: bool) {
        self.string.push_str(s);
        self.fixup
            .extend(std::iter::repeat(fixup).take(s.chars().count()));
    }
}

pub fn serialize<S>(
    style: MessageCommandStyle,
    policy: MessageFixupPolicy,
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
    let policy = MessageFixupPolicy {
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
    policy: MessageFixupPolicy,
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

pub trait TokenSink<'bump> {
    fn push(&mut self, start: usize, end: usize, token: MessageToken<'bump>);
}

impl<'bump> TokenSink<'bump> for Vec<'bump, MessageToken<'bump>> {
    fn push(&mut self, _start: usize, _end: usize, token: MessageToken<'bump>) {
        self.push(token);
    }
}

impl<'bump> TokenSink<'bump> for Vec<'bump, SpannedMessageToken<'bump>> {
    fn push(&mut self, start: usize, end: usize, token: MessageToken<'bump>) {
        self.push(SpannedMessageToken { start, end, token });
    }
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

#[expect(unused)] // not currently used, but it's nice to have it standalone I guess...
pub fn infer_string_fixup_policy<'bump>(
    bump: &'bump Bump,
    decoded: &str,
    style: MessageCommandStyle,
    message_policy: MessageFixupPolicy,
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

/// Combines [`transform`] and [`infer_string_fixup_policy`] into a single pass.
///
/// This is more efficient that doing those separately, since it only parses the message once.
pub fn transform_and_infer_fixup_policy<'bump>(
    bump: &'bump Bump,
    decoded: &'bump str,
    in_style: MessageCommandStyle,
    out_style: MessageCommandStyle,
    message_policy: MessageFixupPolicy,
    detected: FixupDetectResult,
    source: AnyStringSource,
) -> (&'bump str, &'bump [bool]) {
    if source.contains_commands() {
        let mut tokens = Vec::new_in(bump);
        parse(in_style, decoded, &mut tokens);

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

        if in_style == out_style {
            // just a sanity check
            assert_eq!(serialized_string, decoded);
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

pub fn transform<'bump>(
    bump: &'bump Bump,
    decoded: &'bump str,
    in_style: MessageCommandStyle,
    out_style: MessageCommandStyle,
    source: AnyStringSource,
) -> &'bump str {
    if source.contains_commands() {
        let mut tokens = Vec::new_in(bump);
        parse(in_style, decoded, &mut tokens);
        let serialized_string = serialize_string(bump, out_style, &tokens);
        if in_style == out_style {
            // just a sanity check
            assert_eq!(serialized_string, decoded);
        }
        serialized_string
    } else {
        decoded
    }
}
