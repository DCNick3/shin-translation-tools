mod console;
mod csv;

use bumpalo::Bump;
use shin_text::{decode_sjis_zstring, StringArrayIter};
use shin_versions::MessageCommandStyle;

pub use self::{console::ConsoleTraceListener, csv::CsvTraceListener};
use crate::{
    reactor::{AnyStringSource, Reactor, StringArraySource, StringSource},
    reader::Reader,
};

pub trait StringTraceListener {
    fn on_string(&mut self, instr_offset: u32, source: AnyStringSource, s: &str);
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
    source: AnyStringSource,
    s: &'bump [u8],
) {
    let snr_string = decode_sjis_zstring(bump, s, source.contains_commands()).unwrap();

    let user_string =
        crate::layout::message_parser::transform(bump, snr_string, snr_style, user_style, source);

    listener.on_string(instr_offset, source, user_string)
}

fn on_string_array_impl<'bump, L: StringTraceListener>(
    listener: &mut L,
    bump: &'bump Bump,
    snr_style: MessageCommandStyle,
    user_style: MessageCommandStyle,
    instr_offset: u32,
    source: StringArraySource,
    ss: &[u8],
) {
    for (i, s) in (0..).zip(StringArrayIter::new(ss)) {
        on_string_impl(
            listener,
            bump,
            snr_style,
            user_style,
            instr_offset,
            AnyStringSource::Array(source, i),
            s,
        )
    }
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
        let s = self.reader.u8string();
        on_string_impl(
            &mut self.listener,
            &self.bump,
            self.snr_style,
            self.user_style,
            self.current_instr_offset,
            AnyStringSource::Singular(source),
            s,
        )
    }

    fn u16string(&mut self, source: StringSource) {
        let s = self.reader.u16string();
        on_string_impl(
            &mut self.listener,
            &self.bump,
            self.snr_style,
            self.user_style,
            self.current_instr_offset,
            AnyStringSource::Singular(source),
            s,
        )
    }

    fn u8string_array(&mut self, source: StringArraySource) {
        let ss = self.reader.u8string_array();
        on_string_array_impl(
            &mut self.listener,
            &self.bump,
            self.snr_style,
            self.user_style,
            self.current_instr_offset,
            source,
            ss,
        )
    }

    fn u16string_array(&mut self, source: StringArraySource) {
        let ss = self.reader.u16string_array();
        on_string_array_impl(
            &mut self.listener,
            &self.bump,
            self.snr_style,
            self.user_style,
            self.current_instr_offset,
            source,
            ss,
        )
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
