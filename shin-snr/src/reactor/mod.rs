use shin_versions::{AnyStringKind, StringArrayKind, StringKind};

use crate::{
    operation::{
        OperationElementRepr,
        arena::OperationArena,
        schema::{Command, EngineSchema, Opcode, OperationSchema},
    },
    reader::Reader,
};

pub mod dump_bin;
pub mod offset_validator;
pub mod rewrite;
pub mod string_roundrip_validator;
pub mod trace;

pub trait Reactor {
    fn react(
        &mut self,
        operation_position: u32,
        raw_opcode: u8,
        opcode: Opcode,
        op_schema: &OperationSchema,
        arena: &OperationArena,
    );
    fn end_of_stream(&mut self) {}
}

pub fn react_with<R: Reactor>(mut reader: Reader, schema: &EngineSchema, reactor: &mut R) {
    let mut arena = OperationArena::new();

    while reader.has_instr() {
        let operation_position = reader.position();
        let raw_opcode = reader.take_u8();

        let Some(opcode) = schema.lookup_opcode(raw_opcode) else {
            panic!(
                "Undefined opcode: {:?} @ 0x{:08x}",
                raw_opcode, operation_position
            );
        };
        let Some(op_schema) = schema.lookup_operation(opcode) else {
            panic!(
                "Opcode with undefined schema: {:?} @ 0x{:08x}",
                opcode, operation_position
            );
        };

        // eprintln!("{:08x} {:?}", operation_position, opcode);

        op_schema.parse(schema.number_style(), &mut reader, &mut arena);

        reactor.react(operation_position, raw_opcode, opcode, &op_schema, &arena);
    }

    reactor.end_of_stream();
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
            AnyStringSource::Singular(s) => s.subindex(),
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
    pub fn for_operation(
        opcode: Opcode,
        op_schema: &OperationSchema,
        arena: &OperationArena,
    ) -> Option<Self> {
        match opcode {
            Opcode::Instruction(_) => None,
            Opcode::Command(cmd) => match cmd {
                Command::SAVEINFO => Some(StringSource::Saveinfo),
                Command::SELECT => Some(StringSource::Select),
                Command::MSGSET => {
                    let OperationElementRepr::U32(first_argument) = arena
                        .iter(op_schema)
                        .next()
                        .expect("Expected MSGSET to have at least one element")
                    else {
                        panic!("Expected the first MSGSET element to be u32");
                    };
                    // the lower 24-bit are the message id
                    // the top 8 bits are some other thing
                    Some(StringSource::Msgset(first_argument & 0xffffff))
                }
                Command::DEBUGOUT => Some(StringSource::Dbgout),
                Command::LOGSET => Some(StringSource::Logset),
                Command::VOICEPLAY => Some(StringSource::Voiceplay),
                Command::CHATSET => Some(StringSource::Chatset),
                Command::NAMED => Some(StringSource::Named),
                Command::STAGEINFO => Some(StringSource::Stageinfo),

                _ => None,
            },
        }
    }

    pub fn from_kind(kind: StringKind, subindex: u32) -> Self {
        match kind {
            StringKind::Saveinfo => StringSource::Saveinfo,
            StringKind::Select => StringSource::Select,
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
            StringSource::Select => StringKind::Select,
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
    pub fn for_operation(
        opcode: Opcode,
        _op_schema: &OperationSchema,
        _arena: &OperationArena,
    ) -> Option<Self> {
        match opcode {
            Opcode::Instruction(_) => None,
            Opcode::Command(cmd) => match cmd {
                Command::SELECT => Some(StringArraySource::Select),
                _ => None,
            },
        }
    }
    pub fn from_kind(kind: StringArrayKind) -> Self {
        match kind {
            StringArrayKind::SelectChoice => StringArraySource::Select,
        }
    }

    pub fn kind(&self) -> StringArrayKind {
        match self {
            StringArraySource::Select => StringArrayKind::SelectChoice,
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
