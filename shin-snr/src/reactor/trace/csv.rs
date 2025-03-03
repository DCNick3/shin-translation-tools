use std::io;

use bumpalo::collections::String;
use serde::{ser::SerializeStruct, Serialize, Serializer};
use smallvec::SmallVec;

use crate::reactor::{trace::StringTraceListener, StringSource};

pub struct CsvTraceListener<W: io::Write> {
    writer: csv::Writer<W>,
    record_idx: u32,
}

// NOTE: serialization is kind of hard with bumpalo
// #[derive(Serialize)]
struct Record<'bump> {
    index: u32,
    offset: u32,
    source: StringSource,
    s: &'bump str,
    translated: Option<String<'bump>>,
}

impl Serialize for StringSource {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            StringSource::Saveinfo => serializer.serialize_str("saveinfo"),
            StringSource::Select => serializer.serialize_str("select"),
            StringSource::SelectChoice(_) => serializer.serialize_str("select_choice"),
            StringSource::Msgset(_) => serializer.serialize_str("msgset"),
            StringSource::Dbgout => serializer.serialize_str("dbgout"),
            StringSource::Logset => serializer.serialize_str("logset"),
            StringSource::Voiceplay => serializer.serialize_str("voiceplay"),
            StringSource::Chatset => serializer.serialize_str("chatset"),
            StringSource::Named => serializer.serialize_str("named"),
            StringSource::Stageinfo => serializer.serialize_str("stageinfo"),
        }
    }
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
        s.serialize_field("source", &self.source)?;
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
    fn on_string(&mut self, instr_offset: u32, source: StringSource, s: &str) {
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
