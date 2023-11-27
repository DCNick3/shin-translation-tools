mod console;
mod csv;

use bumpalo::{
    collections::{String, Vec},
    Bump,
};

pub use self::{console::ConsoleTraceListener, csv::CsvTraceListener};
use crate::{
    reactor::{Reactor, StringArraySource, StringSource},
    reader::Reader,
    text::decode_sjis_zstring,
};

pub trait StringTraceListener {
    fn on_string(&mut self, instr_offset: u32, source: StringSource, s: String);
}

pub struct StringTraceReactor<'a, L> {
    reader: Reader<'a>,
    current_instr_offset: u32,
    listener: L,
    bump: Bump,
}

impl<'a, L> StringTraceReactor<'a, L> {
    pub fn new(reader: Reader<'a>, listener: L) -> Self {
        Self {
            reader,
            current_instr_offset: 0,
            listener,
            bump: Bump::new(),
        }
    }
}

impl<'a, L: StringTraceListener> Reactor for StringTraceReactor<'a, L> {
    fn byte(&mut self) -> u8 {
        self.reader.byte()
    }

    fn short(&mut self) -> u16 {
        self.reader.short()
    }

    fn reg(&mut self) {
        self.reader.reg();
    }

    fn offset(&mut self) {
        self.reader.offset();
    }

    fn u8string(&mut self, fixup: bool, source: StringSource) {
        let s = self.reader.u8string();
        let s = decode_sjis_zstring(&self.bump, s, fixup).unwrap();
        self.listener
            .on_string(self.current_instr_offset, source, s)
    }

    fn u16string(&mut self, fixup: bool, source: StringSource) {
        let s = self.reader.u16string();
        let s = decode_sjis_zstring(&self.bump, s, fixup).unwrap();
        self.listener
            .on_string(self.current_instr_offset, source, s)
    }

    fn u8string_array(&mut self, fixup: bool, source: StringArraySource) {
        let mut s = self.reader.u8string_array();
        while s.last() == Some(&0) {
            s = &s[..s.len() - 1];
        }
        let mut res = Vec::new_in(&self.bump);
        for s in s.split(|&v| v == 0) {
            let s = decode_sjis_zstring(&self.bump, s, fixup).unwrap();
            res.push(s);
        }

        let source_maker = match source {
            StringArraySource::Select => StringSource::SelectChoice,
        };

        for (i, s) in res.into_iter().enumerate() {
            self.listener
                .on_string(self.current_instr_offset, source_maker(i as u32), s)
        }
    }

    fn msgid(&mut self) -> u32 {
        self.reader.msgid()
    }

    fn instr_start(&mut self) {
        self.bump.reset();
        self.current_instr_offset = self.reader.position();
    }
    fn instr_end(&mut self) {}

    fn has_instr(&self) -> bool {
        self.reader.has_instr()
    }

    fn debug_loc(&self) -> std::string::String {
        format!("0x{:08x}", self.reader.position())
    }
}
