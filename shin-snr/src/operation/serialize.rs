use shin_versions::{LengthKind, NumberStyle};

use crate::{
    operation::{
        OperationElementRepr,
        arena::{Number, Offset, Register},
        repr::NumberArrayKind,
    },
    writer::Writer,
};

pub struct InstructionSerializeContext<W> {
    number_style: NumberStyle,
    writer: W,
}

impl<W> InstructionSerializeContext<W> {
    pub fn new(number_style: NumberStyle, writer: W) -> Self {
        Self {
            number_style,
            writer,
        }
    }

    pub fn writer_ref(&self) -> &W {
        &self.writer
    }
    pub fn writer_mut(&mut self) -> &mut W {
        &mut self.writer
    }

    pub fn into_parts(self) -> (NumberStyle, W) {
        (self.number_style, self.writer)
    }
}

impl<W: Writer> InstructionSerializeContext<W> {
    pub fn put_u8(&mut self, value: u8) {
        self.writer.put_u8(value);
    }
    pub fn put_u16(&mut self, value: u16) {
        self.writer.put_u16(value);
    }
    pub fn put_u32(&mut self, value: u32) {
        self.writer.put_u32(value);
    }
    pub fn put_operation(&mut self, value: u8) {
        self.writer.put_u8(value);
    }
    pub fn put_condition(&mut self, value: u8) {
        self.writer.put_u8(value);
    }
    pub fn put_register(&mut self, value: Register) {
        self.writer.put_u16(value);
    }
    pub fn put_offset(&mut self, value: Offset) {
        self.writer.put_offset(value);
    }
    pub fn put_number(&mut self, value: Number) {
        self.writer.put_number(self.number_style, value);
    }
    pub fn put_pad_number(&mut self, value: Number) {
        match self.number_style {
            NumberStyle::U16 => self.writer.put_u16(value.try_into().unwrap()),
            NumberStyle::VarInt => {
                // NOTE: this will write the padding bits back as they were read
                self.writer.put_u32(value)
            }
        };
    }
    pub fn put_optional_number(&mut self, value: Option<Number>) {
        // NOTE: what you put here should be consistent with the MSB of the operation
        if let Some(value) = value {
            self.put_number(value)
        }
    }
    pub fn put_string(&mut self, length_kind: LengthKind, string: &[u8]) {
        self.writer.put_string(length_kind, string);
    }
    pub fn put_string_array(&mut self, length_kind: LengthKind, string_array: &[u8]) {
        self.writer.put_string_array(length_kind, string_array);
    }

    pub fn put_length(&mut self, kind: LengthKind, value: usize) {
        self.writer.put_length(kind, value);
    }

    // compounds
    pub fn put_expression(&mut self, operations: &[u8], numbers: &[Number]) {
        let mut numbers = numbers.iter().copied();

        for &code in operations {
            self.writer.put_u8(code);
            match code {
                0x00 => {
                    self.put_number(numbers.next().unwrap());
                }
                0xff => {
                    // this should be the last code written, but we choose not to verify this
                }
                _ => {}
            }
        }
    }

    pub fn put_bitmask_number_array(&mut self, mask: u8, numbers: &[Number]) {
        self.writer.put_u8(mask);
        for &number in numbers {
            self.put_number(number);
        }
    }

    pub fn put_higu_sui_wipe_arg(&mut self, b1: u8, b2: u8, numbers: &[Number]) {
        self.writer.put_u8(b1);
        self.writer.put_u8(b2);

        for &number in numbers {
            self.put_number(number);
        }
    }

    pub fn put_element(&mut self, element: &OperationElementRepr) {
        match *element {
            OperationElementRepr::U8(value) => self.put_u8(value),
            OperationElementRepr::U16(value) => self.put_u16(value),
            OperationElementRepr::U32(value) => self.put_u32(value),
            OperationElementRepr::Operation(value) => self.put_operation(value),
            OperationElementRepr::Condition(value) => self.put_condition(value),
            OperationElementRepr::Expression(operations, numbers) => {
                self.put_expression(operations, numbers)
            }
            OperationElementRepr::Register(value) => self.put_register(value),
            OperationElementRepr::RegisterArray(kind, values) => {
                self.put_length(kind, values.len());
                for &value in values {
                    self.put_register(value);
                }
            }
            OperationElementRepr::Offset(value) => self.put_offset(value),
            OperationElementRepr::OffsetArray(kind, values) => {
                self.put_length(kind, values.len());
                for &value in values {
                    self.put_offset(value);
                }
            }
            OperationElementRepr::Number(value) => {
                self.put_number(value);
            }
            OperationElementRepr::OptionalNumber(value) => {
                self.put_optional_number(value);
            }
            OperationElementRepr::NumberArray(kind, array_kind, values) => {
                self.put_length(kind, values.len());
                for &value in values {
                    match array_kind {
                        NumberArrayKind::Dense => self.put_number(value),
                        NumberArrayKind::Padded => self.put_pad_number(value),
                    }
                }
            }
            OperationElementRepr::BitmaskNumberArray(mask, numbers) => {
                self.put_bitmask_number_array(mask, numbers)
            }
            OperationElementRepr::String(kind, value) => self.put_string(kind, value),
            OperationElementRepr::StringArray(kind, value) => self.put_string_array(kind, value),
            OperationElementRepr::HiguSuiWipeArg(b1, b2, numbers) => {
                self.put_higu_sui_wipe_arg(b1, b2, numbers)
            }
        }
    }
}
