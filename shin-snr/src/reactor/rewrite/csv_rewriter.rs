use std::io;

use bumpalo::Bump;
use serde::{de, Deserialize};
use shin_versions::{AnyStringKind, MessageCommandStyle};

use crate::{
    layout::message_parser::lint::diagnostics::LineReport,
    reactor::{rewrite::StringRewriter, AnyStringSource, StringArraySource, StringSource},
};

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

#[derive(Default, Debug, Clone, Copy)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum StringReplacementMode {
    /// Replace strings with the value from `translated` column, leaving those that don't have a translation as-is. The `s` column will be ignored.
    #[default]
    TranslatedOnly,
    /// Replace strings with the value from `translated` column, falling back to `s` column if `translated` is not present. This is the old default behavior.   
    TranslatedOrOriginal,
}

#[derive(Clone)]
struct Entry {
    offset: u32,
    source: AnyStringSource,
    s: String,
    translated: Option<String>,
}

impl Entry {
    pub fn get_effective_string(&self, mode: StringReplacementMode) -> Option<&str> {
        match (&self.translated, mode) {
            (Some(translated), _) => Some(translated),
            (None, StringReplacementMode::TranslatedOnly) => None,
            (None, StringReplacementMode::TranslatedOrOriginal) => Some(&self.s),
        }
    }
}

fn read_csv<R: io::Read>(reader: csv::Reader<R>) -> Vec<Option<Entry>> {
    let mut result = vec![None; 64];

    for (index, entry) in reader
        .into_deserialize()
        .map(|r| r.unwrap())
        .map(|v: RawEntry| (v.index, v.into_entry()))
    {
        if result.len() <= index as usize {
            result.resize_with(result.len() * 2, || None);
        }

        result[index as usize] = Some(entry);
    }

    result
}

pub struct CsvData {
    entries: Vec<Option<Entry>>,
}

impl CsvData {
    pub fn new<R: io::Read>(reader: csv::Reader<R>) -> Self {
        Self {
            entries: read_csv(reader),
        }
    }

    pub fn lint(
        &self,
        mode: StringReplacementMode,
        style: MessageCommandStyle,
    ) -> Result<(), Vec<LineReport>> {
        let mut reports = Vec::new();

        let mut bump = Bump::new();
        for (index, line) in (0..).zip(&self.entries) {
            let Some(line) = line else {
                continue;
            };
            let Some(s) = line.get_effective_string(mode) else {
                continue;
            };

            if let Err(report) = crate::layout::message_parser::lint::lint_string(
                &bump,
                s,
                style,
                line.source,
                index,
            ) {
                reports.push(report);
                if reports.len() >= 64 {
                    // too many errors, stop
                    break;
                }
            }
            bump.reset();
        }

        if reports.is_empty() {
            Ok(())
        } else {
            Err(reports)
        }
    }
}

pub struct CsvRewriter {
    entries: Vec<Option<Entry>>,
    mode: StringReplacementMode,
}

impl CsvRewriter {
    pub fn new(data: CsvData, mode: StringReplacementMode) -> Self {
        Self {
            entries: data.entries,
            mode,
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
        // no entry -> no replacement
        let entry = self.entries[instr_index as usize].as_ref()?;
        assert_eq!(entry.offset, instr_offset);
        assert_eq!(entry.source, source);

        entry.get_effective_string(self.mode)
    }
}
