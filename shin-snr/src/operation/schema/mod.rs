mod def;
mod opcode;

use std::num::NonZeroU8;

use enum_map::EnumMap;
use shin_versions::{LengthKind, NumberStyle};

pub use self::{
    def::ENGINE_SCHEMAS,
    opcode::{Command, Instruction, Opcode},
};
use crate::{
    operation::{arena::OperationArena, parse::InstructionParseContext},
    reader::Reader,
};

#[derive(Debug, Copy, Clone)]
pub enum OperationElement {
    /// Simple 1-byte integer without additional semantics
    U8,
    /// Simple 2-byte integer without additional semantics
    U16,
    /// Simple 4-byte integer without additional semantics
    U32,

    // special kids used only for single instructions
    Operation,
    Condition,
    Expression,

    /// A register (lvalue). 2 bytes.
    Register,
    RegisterArray(LengthKind),

    Offset,
    OffsetArray(LengthKind),

    /// A number (rvalue). Either 2 bytes (older) or variable-length (newer)
    Number,
    /// A number that may be omitted, depending on the most significant bit in a previously parsed `Operation` element
    OptionalNumber,
    /// An array of numbers prefixed by array length. Length size is determined by LengthKind (either 1 or 2 bytes).
    NumberArray(LengthKind),
    /// A padded array of numbers prefixed by array length. Each number is either 2 or 4 bytes, depending on `NumberStyle`
    PadNumberArray(LengthKind),
    /// A byte mask and then a number for each bit set. Used for a lot of initialization commands
    BitmaskNumberArray,

    /// A zero-terminated string prefixed with length. The size of length depends on version and string source.
    String(LengthKind),
    /// A zero-terminated array of zero-terminated strings prefixed with length. The size of length depends on version and string source.
    StringArray(LengthKind),

    // special kids used only for single commands
    HiguSuiWipeArg,
}

pub struct EngineSchema {
    number_style: NumberStyle,
    opcode_map: EnumMap<u8, Option<Opcode>>,
    instructions: EnumMap<Instruction, Option<OperationSchemaSlice>>,
    commands: EnumMap<Command, Option<OperationSchemaSlice>>,
    elements: Vec<OperationElement>,
}

impl EngineSchema {
    pub fn number_style(&self) -> NumberStyle {
        self.number_style
    }
    pub fn lookup_opcode(&self, opcode: u8) -> Option<Opcode> {
        self.opcode_map[opcode]
    }
    pub fn lookup_operation(&self, operation: Opcode) -> Option<OperationSchema<'_>> {
        match operation {
            Opcode::Instruction(instruction) => self.lookup_instruction(instruction),
            Opcode::Command(command) => self.lookup_command(command),
        }
    }
    pub fn lookup_instruction(&self, instruction: Instruction) -> Option<OperationSchema<'_>> {
        let schema = self.instructions[instruction]?;
        let elements = &self.elements[schema.start.get() as usize - 1..][..schema.count as usize];
        Some(OperationSchema { elements })
    }
    pub fn lookup_command(&self, command: Command) -> Option<OperationSchema<'_>> {
        let schema = self.commands[command]?;
        let elements = &self.elements[schema.start.get() as usize - 1..][..schema.count as usize];
        Some(OperationSchema { elements })
    }

    pub fn opcode_map(&self) -> EnumMap<u8, Option<Opcode>> {
        self.opcode_map
    }
    pub fn instruction_map(&self) -> EnumMap<Instruction, Option<OperationSchema<'_>>> {
        EnumMap::from_fn(|cmd| self.lookup_instruction(cmd))
    }
    pub fn command_map(&self) -> EnumMap<Command, Option<OperationSchema<'_>>> {
        EnumMap::from_fn(|cmd| self.lookup_command(cmd))
    }
}

#[derive(Copy, Clone)]
struct OperationSchemaSlice {
    // using [`NonZeroU8`] to provide a niche
    // the actual value is increased by 1
    start: NonZeroU8,
    count: u8,
}

pub struct OperationSchema<'a> {
    pub elements: &'a [OperationElement],
}

impl<'a> OperationSchema<'a> {
    pub fn parse<'snr>(
        &self,
        number_style: NumberStyle,
        reader: &mut Reader<'snr>,
        arena: &mut OperationArena<'snr>,
    ) {
        arena.clear();
        let mut context = InstructionParseContext::new(number_style, reader, arena);
        for &element in self.elements {
            context.take_element(element)
        }
    }
}
