use shin_versions::{StringArrayKind, StringKind};

pub mod offset_validator;
pub mod rewrite;
pub mod trace;

pub trait Reactor {
    fn byte(&mut self) -> u8;
    fn short(&mut self) -> u16;
    fn reg(&mut self);
    fn offset(&mut self);
    fn u8string(&mut self, fixup: bool, source: StringSource);
    fn u16string(&mut self, fixup: bool, source: StringSource);
    fn u8string_array(&mut self, fixup: bool, source: StringArraySource);
    fn u16string_array(&mut self, fixup: bool, source: StringArraySource);
    fn msgid(&mut self) -> u32;

    fn instr_start(&mut self);
    fn instr_end(&mut self);

    fn has_instr(&self) -> bool;

    fn debug_loc(&self) -> String;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StringSource {
    Saveinfo,
    Select,
    // TODO: this should really go into a separate file
    SelectChoice(u32),
    Msgset(u32),
    Dbgout,
    Logset,
    Voiceplay,
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
