mod console;
mod csv;

use bumpalo::{collections::Vec, Bump};
use shin_text::decode_sjis_zstring;
use shin_versions::MessageCommandStyle;

pub use self::{console::ConsoleTraceListener, csv::CsvTraceListener};
use crate::{
    reactor::{Reactor, StringArraySource, StringSource},
    reader::Reader,
};

pub trait StringTraceListener {
    fn on_string(&mut self, instr_offset: u32, source: StringSource, s: &str);
}

pub struct StringTraceReactor<'a, L> {
    reader: Reader<'a>,
    current_instr_offset: u32,
    snr_style: MessageCommandStyle,
    user_style: MessageCommandStyle,
    listener: L,
    bump: Bump,
}

impl<'a, L> StringTraceReactor<'a, L> {
    pub fn new(
        reader: Reader<'a>,
        snr_style: MessageCommandStyle,
        user_style: MessageCommandStyle,
        listener: L,
    ) -> Self {
        Self {
            reader,
            current_instr_offset: 0,
            snr_style,
            user_style,
            listener,
            bump: Bump::new(),
        }
    }
}

fn on_string_impl<'bump, L: StringTraceListener>(
    listener: &mut L,
    bump: &'bump Bump,
    snr_style: MessageCommandStyle,
    user_style: MessageCommandStyle,
    instr_offset: u32,
    source: StringSource,
    snr_string: &'bump str,
) {
    let user_string =
        crate::message_parser::transform(bump, snr_string, snr_style, user_style, source);

    listener.on_string(instr_offset, source, user_string)
}

impl<'a, L: StringTraceListener> Reactor for StringTraceReactor<'a, L> {
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
        let raw = self.reader.u8string();
        let s = decode_sjis_zstring(&self.bump, raw, source.fixup_on_decode()).unwrap();
        on_string_impl(
            &mut self.listener,
            &self.bump,
            self.snr_style,
            self.user_style,
            self.current_instr_offset,
            source,
            s,
        )
    }

    fn u16string(&mut self, source: StringSource) {
        let s = self.reader.u16string();
        let s = decode_sjis_zstring(&self.bump, s, source.fixup_on_decode()).unwrap();
        on_string_impl(
            &mut self.listener,
            &self.bump,
            self.snr_style,
            self.user_style,
            self.current_instr_offset,
            source,
            s,
        )
    }

    fn u8string_array(&mut self, source: StringArraySource) {
        let mut s = self.reader.u8string_array();
        while s.last() == Some(&0) {
            s = &s[..s.len() - 1];
        }
        let mut res = Vec::new_in(&self.bump);
        for s in s.split(|&v| v == 0) {
            let s = decode_sjis_zstring(&self.bump, s, source.fixup_on_decode()).unwrap();
            res.push(s);
        }

        let source_maker = match source {
            StringArraySource::Select => StringSource::SelectChoice,
        };

        for (i, s) in res.into_iter().enumerate() {
            on_string_impl(
                &mut self.listener,
                &self.bump,
                self.snr_style,
                self.user_style,
                self.current_instr_offset,
                source_maker(i as u32),
                s,
            )
        }
    }

    fn u16string_array(&mut self, source: StringArraySource) {
        let mut s = self.reader.u16string_array();
        while s.last() == Some(&0) {
            s = &s[..s.len() - 1];
        }
        let mut res = Vec::new_in(&self.bump);
        for s in s.split(|&v| v == 0) {
            let s = decode_sjis_zstring(&self.bump, s, source.fixup_on_decode()).unwrap();
            res.push(s);
        }

        let source_maker = match source {
            StringArraySource::Select => StringSource::SelectChoice,
        };

        for (i, s) in res.into_iter().enumerate() {
            on_string_impl(
                &mut self.listener,
                &self.bump,
                self.snr_style,
                self.user_style,
                self.current_instr_offset,
                source_maker(i as u32),
                s,
            )
        }
    }

    fn instr_start(&mut self) {
        self.bump.reset();
        self.current_instr_offset = self.reader.position();
    }
    fn instr_end(&mut self) {}

    fn has_instr(&self) -> bool {
        self.reader.has_instr()
    }

    fn in_location(&self) -> u32 {
        self.reader.position()
    }
}
