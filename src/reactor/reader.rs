use crate::{
    reactor::{Reactor, StringArraySource, StringSource},
    reader::Reader,
};

// why do we need it, again?
#[allow(dead_code)]
pub struct ReaderReactor<'a> {
    reader: Reader<'a>,
}

impl<'a> ReaderReactor<'a> {
    pub fn new(reader: Reader<'a>) -> Self {
        Self { reader }
    }
}

impl<'a> Reactor for ReaderReactor<'a> {
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

    fn u8string(&mut self, _fixup: bool, _source: StringSource) {
        self.reader.u8string();
    }

    fn u16string(&mut self, _fixup: bool, _source: StringSource) {
        self.reader.u16string();
    }

    fn u8string_array(&mut self, _fixup: bool, _source: StringArraySource) {
        self.reader.u8string_array();
    }

    fn msgid(&mut self) -> u32 {
        self.reader.msgid()
    }

    fn instr_start(&mut self) {}

    fn has_instr(&self) -> bool {
        self.reader.has_instr()
    }

    fn debug_loc(&self) -> String {
        format!("0x{:08x}", self.reader.position())
    }
}
