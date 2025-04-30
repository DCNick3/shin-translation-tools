use shin_text::StringArrayIter;

use crate::{
    reactor::{AnyStringSource, Reactor, StringArraySource, StringSource},
    reader::Reader,
};

pub struct DumpBinReactor<'a, W> {
    reader: Reader<'a>,
    writer: W,
}

impl<'a, W> DumpBinReactor<'a, W> {
    pub fn new(reader: Reader<'a>, writer: W) -> Self {
        Self { reader, writer }
    }
}

fn on_string_impl<W>(writer: &mut W, source: AnyStringSource, s: &[u8])
where
    W: std::io::Write,
{
    if source.is_for_messagebox() {
        writer.write_all(s).expect("Failed to write bin dump");
    }
}

fn on_string_array_impl<W>(writer: &mut W, source: StringArraySource, ss: &[u8])
where
    W: std::io::Write,
{
    for (i, s) in (0..).zip(StringArrayIter::new(ss)) {
        on_string_impl(writer, AnyStringSource::Array(source, i), s)
    }
}

impl<'a, W> Reactor for DumpBinReactor<'a, W>
where
    W: std::io::Write,
{
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
        on_string_impl(
            &mut self.writer,
            AnyStringSource::Singular(source),
            self.reader.u8string(),
        );
    }

    fn u16string(&mut self, source: StringSource) {
        on_string_impl(
            &mut self.writer,
            AnyStringSource::Singular(source),
            self.reader.u16string(),
        );
    }

    fn u8string_array(&mut self, source: StringArraySource) {
        on_string_array_impl(&mut self.writer, source, self.reader.u8string_array());
    }

    fn u16string_array(&mut self, source: StringArraySource) {
        on_string_array_impl(&mut self.writer, source, self.reader.u16string_array());
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
