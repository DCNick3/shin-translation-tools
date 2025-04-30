use bumpalo::Bump;

use crate::layout::message_parser::{MessageToken, SpannedMessageToken};

pub trait StringSink {
    fn push(&mut self, c: char, fixup: bool);
    fn push_str(&mut self, s: &str, fixup: bool);
}

impl<'b> StringSink for bumpalo::collections::String<'b> {
    fn push(&mut self, c: char, _fixup: bool) {
        self.push(c);
    }

    fn push_str(&mut self, s: &str, _fixup: bool) {
        self.push_str(s);
    }
}

impl StringSink for String {
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
    pub string: bumpalo::collections::String<'b>,
    pub fixup: bumpalo::collections::Vec<'b, bool>,
}

impl<'b> FullStringSink<'b> {
    pub fn new(bump: &'b Bump, capacity_chars: usize, capacity_bytes: usize) -> Self {
        Self {
            string: bumpalo::collections::String::with_capacity_in(capacity_bytes, bump),
            fixup: bumpalo::collections::Vec::with_capacity_in(capacity_chars, bump),
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

pub trait TokenSink<'bump> {
    fn push(&mut self, start: usize, end: usize, token: MessageToken<'bump>);
}

impl<'bump> TokenSink<'bump> for Vec<MessageToken<'bump>> {
    fn push(&mut self, _start: usize, _end: usize, token: MessageToken<'bump>) {
        self.push(token);
    }
}

impl<'bump> TokenSink<'bump> for bumpalo::collections::Vec<'bump, MessageToken<'bump>> {
    fn push(&mut self, _start: usize, _end: usize, token: MessageToken<'bump>) {
        self.push(token);
    }
}

impl<'bump> TokenSink<'bump> for bumpalo::collections::Vec<'bump, SpannedMessageToken<'bump>> {
    fn push(&mut self, start: usize, end: usize, token: MessageToken<'bump>) {
        self.push(SpannedMessageToken { start, end, token });
    }
}
