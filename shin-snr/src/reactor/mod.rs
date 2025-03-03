use shin_versions::{AnyStringKind, StringArrayKind, StringKind};

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
    fn u8string(&mut self, source: StringSource);
    fn u16string(&mut self, source: StringSource);
    fn u8string_array(&mut self, source: StringArraySource);
    fn u16string_array(&mut self, source: StringArraySource);

    fn instr_start(&mut self);
    fn instr_end(&mut self);

    fn has_instr(&self) -> bool;

    fn in_location(&self) -> u32;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AnyStringSource {
    Singular(StringSource),
    Array(StringArraySource, u32),
}

impl AnyStringSource {
    pub fn kind(&self) -> AnyStringKind {
        match self {
            AnyStringSource::Singular(s) => AnyStringKind::Singular(s.kind()),
            AnyStringSource::Array(s, _) => AnyStringKind::Array(s.kind()),
        }
    }

    pub fn subindex(&self) -> u32 {
        match self {
            AnyStringSource::Singular(s) => s.subindex2(),
            &AnyStringSource::Array(_, i) => i,
        }
    }

    pub fn is_for_messagebox(&self) -> bool {
        match self {
            AnyStringSource::Singular(s) => s.is_for_messagebox(),
            AnyStringSource::Array(s, _) => s.is_for_messagebox(),
        }
    }

    pub fn contains_commands(&self) -> bool {
        match self {
            AnyStringSource::Singular(s) => s.contains_commands(),
            AnyStringSource::Array(s, _) => s.contains_commands(),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StringSource {
    Saveinfo,
    Select,
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
    pub fn from_kind(kind: StringKind, subindex: u32) -> Self {
        match kind {
            StringKind::Saveinfo => StringSource::Saveinfo,
            StringKind::SelectTitle => StringSource::Select,
            StringKind::Msgset => StringSource::Msgset(subindex),
            StringKind::Dbgout => StringSource::Dbgout,
            StringKind::Logset => StringSource::Logset,
            StringKind::Voiceplay => StringSource::Voiceplay,
            StringKind::Chatset => StringSource::Chatset,
            StringKind::Named => StringSource::Named,
            StringKind::Stageinfo => StringSource::Stageinfo,
        }
    }

    pub fn kind(&self) -> StringKind {
        match *self {
            StringSource::Saveinfo => StringKind::Saveinfo,
            StringSource::Select => StringKind::SelectTitle,
            StringSource::Msgset(_) => StringKind::Msgset,
            StringSource::Dbgout => StringKind::Dbgout,
            StringSource::Logset => StringKind::Logset,
            StringSource::Voiceplay => StringKind::Voiceplay,
            StringSource::Chatset => StringKind::Chatset,
            StringSource::Named => StringKind::Named,
            StringSource::Stageinfo => StringKind::Stageinfo,
        }
    }

    pub fn subindex2(&self) -> u32 {
        match *self {
            StringSource::Saveinfo => 0,
            StringSource::Select => 0,
            StringSource::Msgset(i) => i,
            StringSource::Dbgout => 0,
            StringSource::Logset => 0,
            StringSource::Voiceplay => 0,
            StringSource::Chatset => 0,
            StringSource::Named => 0,
            StringSource::Stageinfo => 0,
        }
    }

    pub fn is_for_messagebox(&self) -> bool {
        match self {
            StringSource::Msgset(_) | StringSource::Logset => true,
            _ => false,
        }
    }

    pub fn contains_commands(&self) -> bool {
        match self {
            StringSource::Msgset(_) | StringSource::Logset => true,
            _ => false,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StringArraySource {
    Select,
}

impl StringArraySource {
    pub fn from_kind(kind: StringArrayKind) -> Self {
        match kind {
            StringArrayKind::SelectChoices => StringArraySource::Select,
        }
    }

    pub fn kind(&self) -> StringArrayKind {
        match self {
            StringArraySource::Select => StringArrayKind::SelectChoices,
        }
    }

    pub fn is_for_messagebox(&self) -> bool {
        match self {
            StringArraySource::Select => false,
        }
    }

    pub fn contains_commands(&self) -> bool {
        match self {
            StringArraySource::Select => true,
        }
    }
}
