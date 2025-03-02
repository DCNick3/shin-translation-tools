use std::str::Chars;

use bumpalo::{
    collections::{String, Vec},
    vec, Bump,
};
use shin_text::FixupDetectResult;
use shin_versions::{MessageCommandStyle, MessageFixupPolicy};

use crate::reactor::StringSource;

pub struct CommandToken<'b> {
    pub command: char,
    pub argument: Option<&'b str>,
}

pub enum MessageToken<'b> {
    Literal(char),
    EscapedLiteral(char),
    Command(CommandToken<'b>),
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

pub trait Sink {
    fn push(&mut self, c: char, fixup: bool);
    fn push_str(&mut self, s: &str, fixup: bool);
}

impl<'b> Sink for String<'b> {
    fn push(&mut self, c: char, _fixup: bool) {
        self.push(c);
    }

    fn push_str(&mut self, s: &str, _fixup: bool) {
        self.push_str(s);
    }
}

pub struct CountingSink {
    pub char_count: usize,
    pub utf8_byte_count: usize,
}

impl CountingSink {
    pub fn new() -> Self {
        Self {
            char_count: 0,
            utf8_byte_count: 0,
        }
    }
}

impl Sink for CountingSink {
    fn push(&mut self, c: char, _fixup: bool) {
        self.char_count += 1;
        self.utf8_byte_count += c.len_utf8();
    }

    fn push_str(&mut self, s: &str, _fixup: bool) {
        self.char_count += s.chars().count();
        self.utf8_byte_count += s.len();
    }
}

pub struct FullSink<'b> {
    pub string: String<'b>,
    pub fixup: Vec<'b, bool>,
}

impl<'b> FullSink<'b> {
    pub fn new(bump: &'b Bump, capacity_chars: usize, capacity_bytes: usize) -> Self {
        Self {
            string: String::with_capacity_in(capacity_bytes, bump),
            fixup: Vec::with_capacity_in(capacity_chars, bump),
        }
    }
}

impl Sink for FullSink<'_> {
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
    tokens: &[MessageToken],
    sink: &mut S,
) where
    S: Sink,
{
    let mut finished_first_line = false;

    for token in tokens {
        match *token {
            MessageToken::Literal(lit) => {
                sink.push(
                    lit,
                    if policy.fixup_character_names {
                        true
                    } else {
                        finished_first_line
                    },
                );
            }
            MessageToken::EscapedLiteral(lit) => match style {
                MessageCommandStyle::Escaped => {
                    sink.push('@', false);
                    sink.push(lit, false);
                }
                MessageCommandStyle::Unescaped => {
                    sink.push('!', false);
                    sink.push(lit, false);
                }
            },
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

pub fn serialize_full<'b>(
    bump: &'b Bump,
    style: MessageCommandStyle,
    policy: MessageFixupPolicy,
    tokens: &[MessageToken],
) -> (&'b str, &'b [bool]) {
    let mut counting_sink = CountingSink::new();

    serialize(style, policy, tokens, &mut counting_sink);

    let mut full_sink = FullSink::new(
        bump,
        counting_sink.char_count,
        counting_sink.utf8_byte_count,
    );

    serialize(style, policy, tokens, &mut full_sink);

    (
        full_sink.string.into_bump_str(),
        full_sink.fixup.into_bump_slice(),
    )
}

pub fn parse<'b>(
    bump: &'b Bump,
    style: MessageCommandStyle,
    message: &'b str,
) -> Vec<'b, MessageToken<'b>> {
    let mut iter = message.chars();

    fn read_argument<'b>(iter: &mut Chars<'b>) -> Option<&'b str> {
        let s = iter.as_str();
        match s.find('.') {
            None => None,
            Some(pos) => {
                let arg = &s[..pos];
                let tail = &s[pos + 1..];
                *iter = tail.chars();
                Some(arg)
            }
        }
    }

    let mut results = Vec::with_capacity_in(message.len(), bump);

    while let Some(c) = iter.next() {
        match style {
            MessageCommandStyle::Escaped => {
                if c == '@' {
                    let Some(c) = iter.next() else {
                        todo!("handle unmatched @");
                    };
                    let has_argument = MessageCommand::parse(c).is_some_and(|c| c.has_arg());
                    let argument = has_argument.then(|| read_argument(&mut iter)).flatten();
                    results.push(MessageToken::Command(CommandToken {
                        command: c,
                        argument,
                    }));
                } else {
                    results.push(MessageToken::Literal(c));
                }
            }
            MessageCommandStyle::Unescaped => {
                if (c as u16) < 128 {
                    if c == '!' {
                        // the game doesn't check end-of-line here too,
                        // so this is almost 100% invalid string that we will never encounter in the wild
                        let c = iter.next().unwrap();
                        results.push(MessageToken::EscapedLiteral(c));
                    } else {
                        // NOTE: we handle invalid commands the same way the engine would: by ignoring them
                        // the only difference is that we can recognize more commands than the engine for older version of the engine
                        // (they didn't have the `/`, `t` and `u`)
                        let has_argument = MessageCommand::parse(c).is_some_and(|c| c.has_arg());
                        let argument = has_argument.then(|| read_argument(&mut iter)).flatten();
                        results.push(MessageToken::Command(CommandToken {
                            command: c,
                            argument,
                        }));
                    }
                } else {
                    results.push(MessageToken::Literal(c));
                }
            }
        }
    }

    results
}

pub fn infer_string_fixup_policy<'b>(
    bump: &'b Bump,
    decoded: &str,
    style: MessageCommandStyle,
    message_policy: MessageFixupPolicy,
    detected: FixupDetectResult,
    source: StringSource,
) -> &'b [bool] {
    // some messages are just not fixed up, even though it makes sense to do so
    // in higurashi sui this happens with messages from the debug menu
    if detected == FixupDetectResult::UnfixedUp {
        return vec![in bump; false; decoded.chars().count()].into_bump_slice();
    }

    match source {
        StringSource::Msgset(_) | StringSource::Logset => {
            let tokens = parse(bump, style, decoded);
            let (serialized_string, fixup_policy) =
                serialize_full(bump, style, message_policy, &tokens);
            assert_eq!(serialized_string, decoded);
            fixup_policy
        }
        _ => vec![in bump; false; decoded.chars().count()].into_bump_slice(),
    }
}
