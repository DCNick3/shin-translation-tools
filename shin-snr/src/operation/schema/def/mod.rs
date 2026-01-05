use std::{num::NonZeroU8, sync::LazyLock};

use drop_bomb::DebugDropBomb;
use enum_map::EnumMap;
use shin_versions::{LengthKind, ShinVersion};

use crate::operation::schema::{OperationElement, OperationSchemaSlice, opcode::Command};

pub mod command;
mod instruction;
pub mod opcode;

pub enum OperationResult {
    Ok,
    Unreachable,
    Unimplemented,
}

struct SchemaBuilder<E> {
    elements: Vec<E>,
}

impl<E> SchemaBuilder<E> {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    pub fn start(&mut self) -> OperationSchemaBuilder<'_, E> {
        OperationSchemaBuilder {
            start: u8::try_from(self.elements.len()).unwrap(),
            parent: self,
            bomb: DebugDropBomb::new(
                "`CommandSchemaBuilder::finish` or `CommandSchemaBuilder::abandon` has to be called",
            ),
        }
    }

    pub fn finish(self) -> Vec<E> {
        self.elements
    }
}

struct OperationSchemaBuilder<'a, E> {
    parent: &'a mut SchemaBuilder<E>,
    start: u8,
    bomb: DebugDropBomb,
}

impl<E> OperationSchemaBuilder<'_, E> {
    pub fn add_element(&mut self, element: E) {
        self.parent.elements.push(element);
    }

    pub fn finish(self) -> OperationSchemaSlice {
        let Self {
            parent,
            start,
            mut bomb,
        } = self;
        bomb.defuse();

        let count = u8::try_from(parent.elements.len()).unwrap() - start;

        let start = NonZeroU8::new(start.checked_add(1).unwrap()).unwrap();

        OperationSchemaSlice { start, count }
    }

    pub fn abandon(self) {
        let Self {
            parent,
            start,
            mut bomb,
        } = self;
        bomb.defuse();

        parent.elements.truncate(start as usize);
    }
}

struct InstructionSchemaCtx<'a> {
    version: ShinVersion,
    builder: OperationSchemaBuilder<'a, OperationElement>,
}

impl<'a> InstructionSchemaCtx<'a> {
    pub fn version(&self) -> ShinVersion {
        self.version
    }
    pub fn operation(&mut self) {
        self.builder.add_element(OperationElement::Operation);
    }
    pub fn condition(&mut self) {
        self.builder.add_element(OperationElement::Condition);
    }
    pub fn expression(&mut self) {
        self.builder.add_element(OperationElement::Expression);
    }
    pub fn reg(&mut self) {
        self.builder.add_element(OperationElement::Register);
    }
    pub fn reg_array(&mut self, length: LengthKind) {
        self.builder
            .add_element(OperationElement::RegisterArray(length));
    }
    pub fn offset(&mut self) {
        self.builder.add_element(OperationElement::Offset);
    }
    pub fn offset_array(&mut self, length: LengthKind) {
        self.builder
            .add_element(OperationElement::OffsetArray(length));
    }
    pub fn number(&mut self) {
        self.builder.add_element(OperationElement::Number);
    }
    pub fn optional_number(&mut self) {
        self.builder.add_element(OperationElement::OptionalNumber);
    }
    pub fn number_array(&mut self, length: LengthKind) {
        self.builder
            .add_element(OperationElement::NumberArray(length));
    }
    pub fn pad_number_array(&mut self, length: LengthKind) {
        self.builder
            .add_element(OperationElement::PadNumberArray(length));
    }
}

struct CommandSchemaCtx<'a> {
    version: ShinVersion,
    command: Command,
    builder: OperationSchemaBuilder<'a, OperationElement>,
}

impl<'a> CommandSchemaCtx<'a> {
    /// Get engine version (to handle encodings that are different between versions)
    pub fn version(&self) -> ShinVersion {
        self.version
    }

    /// Simple 1-byte integer without additional semantics
    pub fn u8(&mut self) {
        self.builder.add_element(OperationElement::U8);
    }

    /// Simple 2-byte integer without additional semantics
    pub fn u16(&mut self) {
        self.builder.add_element(OperationElement::U16);
    }

    /// Simple 4-byte integer without additional semantics
    pub fn u32(&mut self) {
        self.builder.add_element(OperationElement::U32);
    }

    /// A register (lvalue). 2 bytes.
    pub fn reg(&mut self) {
        self.builder.add_element(OperationElement::Register);
    }

    /// A number (rvalue). Either 2 bytes (older) or variable-length (newer)
    pub fn number(&mut self) {
        self.builder.add_element(OperationElement::Number);
    }

    /// An array of numbers prefixed by array length. Length size is determined by LengthKind (either 1 or 2 bytes).
    pub fn number_array(&mut self, length: LengthKind) {
        self.builder
            .add_element(OperationElement::NumberArray(length));
    }

    /// A byte mask and then a number for each bit set. Used for a lot of initialization commands
    pub fn bitmask_number_array(&mut self) {
        self.builder
            .add_element(OperationElement::BitmaskNumberArray)
    }

    /// A zero-terminated string prefixed with length. The size of length depends on version and string source.
    pub fn string(&mut self) {
        let size = self
            .version
            .string_style(self.command.try_into().unwrap())
            .size_kind;
        self.builder.add_element(OperationElement::String(size))
    }

    /// A zero-terminated array of zero-terminated strings prefixed with length. The size of length depends on version and string source.
    pub fn string_array(&mut self) {
        let size = self
            .version
            .string_array_style(self.command.try_into().unwrap())
            .size_kind;
        self.builder
            .add_element(OperationElement::StringArray(size))
    }

    pub fn custom(&mut self, element: OperationElement) {
        self.builder.add_element(element)
    }
}

fn get_schema_for_version(version: ShinVersion) -> super::EngineSchema {
    let mut builder = SchemaBuilder::new();

    let number_style = version.number_style();

    let opcode_map = EnumMap::from_fn(|opcode| opcode::get_opcode_name(version, opcode));

    let instructions = EnumMap::from_fn(|command| {
        let mut ctx = InstructionSchemaCtx {
            version,
            builder: builder.start(),
        };
        match instruction::get_schema_for_instruction(&mut ctx, command) {
            OperationResult::Ok => Some(ctx.builder.finish()),
            OperationResult::Unreachable | OperationResult::Unimplemented => {
                ctx.builder.abandon();
                None
            }
        }
    });

    let commands = EnumMap::from_fn(|command| {
        let mut ctx = CommandSchemaCtx {
            version,
            command,
            builder: builder.start(),
        };
        match command::get_schema_for_command(&mut ctx, command) {
            OperationResult::Ok => Some(ctx.builder.finish()),
            OperationResult::Unreachable | OperationResult::Unimplemented => {
                ctx.builder.abandon();
                None
            }
        }
    });

    let elements = builder.finish();

    super::EngineSchema {
        number_style,
        opcode_map,
        instructions,
        commands,
        elements,
    }
}

pub static ENGINE_SCHEMAS: LazyLock<EnumMap<ShinVersion, super::EngineSchema>> =
    LazyLock::new(|| EnumMap::from_fn(get_schema_for_version));

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, ops::Deref};

    use enum_map::{Enum, EnumMap};
    use shin_versions::ShinVersion;

    use crate::operation::schema::{
        EngineSchema, OperationElement, OperationSchema,
        opcode::{Command, Instruction, Opcode},
    };

    trait Aspect {
        type Key: Debug + Copy + Enum;

        fn lower_opcode(opcode: Opcode) -> Option<Self::Key>;
        fn get_map(schema: &EngineSchema) -> EnumMap<Self::Key, Option<OperationSchema<'_>>>;
    }

    struct InstructionAspect;
    struct CommandAspect;

    impl Aspect for InstructionAspect {
        type Key = Instruction;
        fn lower_opcode(opcode: Opcode) -> Option<Self::Key> {
            match opcode {
                Opcode::Instruction(instruction) => Some(instruction),
                Opcode::Command(_) => None,
            }
        }

        fn get_map(schema: &EngineSchema) -> EnumMap<Self::Key, Option<OperationSchema<'_>>> {
            schema.instruction_map()
        }
    }
    impl Aspect for CommandAspect {
        type Key = Command;

        fn lower_opcode(opcode: Opcode) -> Option<Self::Key> {
            match opcode {
                Opcode::Instruction(_) => None,
                Opcode::Command(command) => Some(command),
            }
        }

        fn get_map(schema: &EngineSchema) -> EnumMap<Self::Key, Option<OperationSchema<'_>>> {
            schema.command_map()
        }
    }

    fn analyze_reachables<A>(
        engine_schemas: &EnumMap<ShinVersion, EngineSchema>,
        diags: &mut Vec<String>,
    ) where
        A: Aspect,
    {
        let mut opcode_reachables = EnumMap::<_, EnumMap<_, _>>::default();
        let mut operation_reachables = EnumMap::<_, EnumMap<_, _>>::default();

        for (version, schema) in engine_schemas {
            for (_, opcode) in schema.opcode_map() {
                let Some(operation) = opcode.and_then(A::lower_opcode) else {
                    continue;
                };

                opcode_reachables[operation][version] = true;
            }

            for (operation, elements) in A::get_map(schema) {
                if !elements.is_some() {
                    continue;
                }

                operation_reachables[operation][version] = true;
            }
        }

        for ((command, opcodes), (_, commands)) in
            opcode_reachables.iter().zip(&operation_reachables)
        {
            if opcodes != commands {
                let guard = opcodes
                    .iter()
                    .filter(|&(_, &v)| v)
                    .map(|(k, _)| format!("{k:?}"))
                    .collect::<Vec<_>>()
                    .join(" | ");

                diags.push(format!("{command:?} should have `guard!({guard})`"));
            }
        }
    }

    #[test]
    fn schema_consistency() {
        let engine_schemas = super::ENGINE_SCHEMAS.deref();

        let engine_schema = &engine_schemas[ShinVersion::Gerokasu2];

        eprintln!(
            "Note: engine_schemas is 0x{:x} bytes long and engine_schema is 0x{:x} bytes long",
            size_of_val(engine_schemas),
            size_of_val(engine_schema),
        );

        let mut diags = Vec::new();

        analyze_reachables::<InstructionAspect>(engine_schemas, &mut diags);
        analyze_reachables::<CommandAspect>(engine_schemas, &mut diags);

        if !diags.is_empty() {
            panic!(
                "Found following issues with the schema:\n{}",
                diags.join("\n")
            )
        } else {
            eprintln!("No consistency issues found with the schema!")
        }
    }
}
