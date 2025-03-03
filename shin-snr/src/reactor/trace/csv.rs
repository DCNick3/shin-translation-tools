use std::io;

use bumpalo::collections::String;
use serde::{ser::SerializeStruct, Serialize, Serializer};
use smallvec::SmallVec;

use crate::reactor::{trace::StringTraceListener, AnyStringSource};

pub struct CsvTraceListener<W: io::Write> {
    writer: csv::Writer<W>,
    record_idx: u32,
}

// NOTE: serialization is kind of hard with bumpalo
// #[derive(Serialize)]
struct Record<'bump> {
    index: u32,
    offset: u32,
    source: AnyStringSource,
    s: &'bump str,
    translated: Option<String<'bump>>,
}

impl<'bump> Serialize for Record<'bump> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use std::io::Write as _;
        let mut buf = SmallVec::<u8, 16>::new();

        let mut s = serializer.serialize_struct("Record", 6)?;
        s.serialize_field("index", &self.index)?;

        write!(buf, "0x{:08x}", self.offset).unwrap();
        debug_assert!(std::str::from_utf8(buf.as_ref()).is_ok());
        let offset = unsafe { std::str::from_utf8_unchecked(buf.as_ref()) };

        s.serialize_field("offset", &offset)?;
        s.serialize_field("source", &self.source.kind())?;
        s.serialize_field("source_subindex", &self.source.subindex())?;
        s.serialize_field("s", &self.s)?;
        s.serialize_field("translated", &self.translated.as_ref().map(|s| s.as_str()))?;
        s.end()
    }
}

impl<W: io::Write> CsvTraceListener<W> {
    pub fn new(writer: csv::Writer<W>) -> Self {
        Self {
            writer,
            record_idx: 0,
        }
    }
}

impl<W: io::Write> StringTraceListener for CsvTraceListener<W> {
    fn on_string(&mut self, instr_offset: u32, source: AnyStringSource, s: &str) {
        self.writer
            .serialize(Record {
                index: self.record_idx,
                offset: instr_offset,
                source,
                s,
                translated: None,
            })
            .unwrap();
        self.record_idx += 1;
    }
}
