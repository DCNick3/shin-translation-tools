use shin_versions::{StringArrayKind, StringKind};

pub mod location_painter;
pub mod offset_validator;
pub mod rewrite;
pub mod string_roundrip_validator;
pub mod trace;

pub trait Reactor {
    /// A single byte
    fn byte(&mut self) -> u8;
    /// A 2-byte short
    fn short(&mut self) -> u16;
    /// A 4-byte uint
    fn uint(&mut self) -> u32;
    /// A 2-byte register
    fn reg(&mut self);
    /// A 4-byte jump offset into the snr file
    fn offset(&mut self);
    fn u8string(&mut self, fixup: bool, source: StringSource);
    fn u16string(&mut self, fixup: bool, source: StringSource);
    fn u8string_array(&mut self, fixup: bool, source: StringArraySource);
    fn u16string_array(&mut self, fixup: bool, source: StringArraySource);

    fn instr_start(&mut self);
    fn instr_end(&mut self);

    fn has_instr(&self) -> bool;

    fn in_location(&self) -> u32;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StringSource {
    Saveinfo,
    Select,
    // TODO: this should really go into a separate enum
    SelectChoice(u32),
    Msgset(u32),
    Dbgout,
    Logset,
    Voiceplay,
    // Game-specific string sources
    // DC4
    Chatset,
    // Alias Carnival
    Named,
    Stageinfo,
}

impl StringSource {
    pub fn kind(&self) -> StringKind {
        match *self {
            StringSource::Saveinfo => StringKind::Saveinfo,
            StringSource::Select => StringKind::SelectTitle,
            StringSource::SelectChoice(_) => unreachable!(),
            StringSource::Msgset(_) => StringKind::Msgset,
            StringSource::Dbgout => StringKind::Dbgout,
            StringSource::Logset => StringKind::Logset,
            StringSource::Voiceplay => StringKind::Voiceplay,
            StringSource::Chatset => StringKind::Chatset,
            StringSource::Named => StringKind::Named,
            StringSource::Stageinfo => StringKind::Stageinfo,
        }
    }

    pub fn subindex(&self) -> u32 {
        match *self {
            StringSource::Saveinfo => 0,
            StringSource::Select => 0,
            StringSource::SelectChoice(i) => i,
            StringSource::Msgset(i) => i,
            StringSource::Dbgout => 0,
            StringSource::Logset => 0,
            StringSource::Voiceplay => 0,
            StringSource::Chatset => 0,
            StringSource::Named => 0,
            StringSource::Stageinfo => 0,
        }
    }
}

#[derive(Debug)]
pub enum StringArraySource {
    Select,
}

impl StringArraySource {
    pub fn kind(&self) -> StringArrayKind {
        match self {
            StringArraySource::Select => StringArrayKind::SelectChoices,
        }
    }
}
