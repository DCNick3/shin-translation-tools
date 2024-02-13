use std::{collections::HashMap, io};

use bumpalo::Bump;
use serde::{de, Deserialize};

use crate::reactor::{rewrite::StringRewriter, StringSource};

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum RawStringSource {
    Saveinfo,
    Select,
    SelectChoice,
    Msgset,
    Dbgout,
    Logset,
    Voiceplay,
    Chatset,
}

fn deser_hex<'de, D: serde::Deserializer<'de>>(deser: D) -> Result<u32, D::Error> {
    struct HexVisitor;
    impl<'de> de::Visitor<'de> for HexVisitor {
        type Value = u32;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a hex-encoded number")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            u32::from_str_radix(v.strip_prefix("0x").unwrap_or(v), 16).map_err(de::Error::custom)
        }
    }
    deser.deserialize_str(HexVisitor)
}

#[derive(Deserialize)]
struct RawEntry {
    index: u32,
    #[serde(deserialize_with = "deser_hex")]
    offset: u32,
    source: RawStringSource,
    source_subindex: u32,
    s: String,
    translated: Option<String>,
}

impl RawEntry {
    fn into_entry(self) -> Entry {
        Entry {
            offset: self.offset,
            source: match self.source {
                RawStringSource::Saveinfo => StringSource::Saveinfo,
                RawStringSource::Select => StringSource::Select,
                RawStringSource::SelectChoice => StringSource::SelectChoice(self.source_subindex),
                RawStringSource::Msgset => StringSource::Msgset(self.source_subindex),
                RawStringSource::Dbgout => StringSource::Dbgout,
                RawStringSource::Logset => StringSource::Logset,
                RawStringSource::Voiceplay => StringSource::Voiceplay,
                RawStringSource::Chatset => StringSource::Chatset,
            },
            s: self.s,
            translated: self.translated,
        }
    }
}

struct Entry {
    offset: u32,
    source: StringSource,
    s: String,
    translated: Option<String>,
}

fn read_csv<R: io::Read>(reader: csv::Reader<R>) -> HashMap<u32, Entry> {
    reader
        .into_deserialize()
        .map(|r| r.unwrap())
        .map(|v: RawEntry| (v.index, v.into_entry()))
        .collect()
}

pub struct CsvRewriter {
    entries: HashMap<u32, Entry>,
}

impl CsvRewriter {
    pub fn new<R: io::Read>(reader: csv::Reader<R>) -> Self {
        Self {
            entries: read_csv(reader),
        }
    }
}

impl StringRewriter for CsvRewriter {
    fn rewrite_string<'a>(
        &'a self,
        _bump: &'a Bump,
        instr_index: u32,
        instr_offset: u32,
        source: StringSource,
    ) -> Option<&'a str> {
        let entry = self.entries.get(&instr_index)?;
        assert_eq!(entry.offset, instr_offset);
        assert_eq!(entry.source, source);

        let rewrite = entry.translated.as_deref().unwrap_or(entry.s.as_str());

        Some(rewrite)
    }
}
