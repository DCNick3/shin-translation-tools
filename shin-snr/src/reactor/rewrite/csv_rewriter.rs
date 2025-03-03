use std::{collections::HashMap, io};

use bumpalo::Bump;
use serde::{de, Deserialize};
use shin_versions::AnyStringKind;

use crate::reactor::{rewrite::StringRewriter, AnyStringSource, StringArraySource, StringSource};

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
    Named,
    Stageinfo,
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
    source: AnyStringKind,
    source_subindex: u32,
    s: String,
    translated: Option<String>,
}

impl RawEntry {
    fn into_entry(self) -> Entry {
        Entry {
            offset: self.offset,
            source: match self.source {
                AnyStringKind::Singular(singular) => AnyStringSource::Singular(
                    StringSource::from_kind(singular, self.source_subindex),
                ),
                AnyStringKind::Array(array) => AnyStringSource::Array(
                    StringArraySource::from_kind(array),
                    self.source_subindex,
                ),
            },
            s: self.s,
            translated: self.translated,
        }
    }
}

struct Entry {
    offset: u32,
    source: AnyStringSource,
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
    fn rewrite_string<'bump>(
        &'bump self,
        _bump: &'bump Bump,
        _decoded: &'bump str,
        instr_index: u32,
        instr_offset: u32,
        source: AnyStringSource,
    ) -> Option<&'bump str> {
        let entry = self.entries.get(&instr_index)?;
        assert_eq!(entry.offset, instr_offset);
        assert_eq!(entry.source, source);

        let rewrite = entry.translated.as_deref().unwrap_or(entry.s.as_str());

        Some(rewrite)
    }
}
