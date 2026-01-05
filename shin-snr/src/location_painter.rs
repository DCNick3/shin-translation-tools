//! A debugging aid that will determine element type for each byte.

use crate::{
    operation::{
        arena::OperationArena,
        parse::InstructionParseContext,
        schema::{EngineSchema, OperationElement},
    },
    reader::Reader,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LocationColor {
    #[default]
    Unmarked,
    Opcode,
    U8,
    U16,
    U32,
    Operation,
    Condition,
    Expression,
    Register,
    RegisterArray,
    Offset,
    OffsetArray,
    Number,
    OptionalNumber,
    NumberArray,
    PadNumberArray,
    BitmaskNumberArray,
    String,
    StringArray,
    HiguSuiWipeArg,
}

pub fn paint_locations(mut reader: Reader, schema: &EngineSchema) -> Vec<LocationColor> {
    let mut colors = vec![LocationColor::Unmarked; reader.size()];

    let mut arena = OperationArena::new();

    while reader.has_instr() {
        let operation_position = reader.position();

        colors[operation_position as usize] = LocationColor::Opcode;
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

        arena.clear();
        let mut context =
            InstructionParseContext::new(schema.number_style(), &mut reader, &mut arena);
        for &element in op_schema.elements {
            let start_position = context.reader_ref().position();
            context.take_element(element);
            let end_position = context.reader_ref().position();

            let color = match element {
                OperationElement::U8 => LocationColor::U8,
                OperationElement::U16 => LocationColor::U16,
                OperationElement::U32 => LocationColor::U32,
                OperationElement::Operation => LocationColor::Operation,
                OperationElement::Condition => LocationColor::Condition,
                OperationElement::Expression => LocationColor::Expression,
                OperationElement::Register => LocationColor::Register,
                OperationElement::RegisterArray(_) => LocationColor::RegisterArray,
                OperationElement::Offset => LocationColor::Offset,
                OperationElement::OffsetArray(_) => LocationColor::OffsetArray,
                OperationElement::Number => LocationColor::Number,
                OperationElement::OptionalNumber => LocationColor::OptionalNumber,
                OperationElement::NumberArray(_) => LocationColor::NumberArray,
                OperationElement::PadNumberArray(_) => LocationColor::PadNumberArray,
                OperationElement::BitmaskNumberArray => LocationColor::BitmaskNumberArray,
                OperationElement::String(_) => LocationColor::String,
                OperationElement::StringArray(_) => LocationColor::StringArray,
                OperationElement::HiguSuiWipeArg => LocationColor::HiguSuiWipeArg,
            };

            colors[start_position as usize..end_position as usize].fill(color);
        }
    }

    colors
}
