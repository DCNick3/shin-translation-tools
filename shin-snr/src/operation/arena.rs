use crate::operation::{
    repr::{NumberArrayKind, OperationElementRepr},
    schema::{OperationElement, OperationSchema},
};

pub type Register = u16;
pub type Number = u32;
pub type Offset = u32;

pub struct OperationArenaPosition {
    u8s: u32,
    u16s: u32,
    u32s: u32,

    registers: u32,
    offsets: u32,
    numbers: u32,

    strings: u32,
    string_arrays: u32,

    lengths: u32,
}

macro_rules! take {
    ($this:expr, $arena:expr, $tys:ident) => {{
        let res = $arena.$tys[$this.$tys as usize];
        $this.$tys += 1;
        res
    }};

    ($this:expr, $arena:expr, @multiple $tys:ident) => {{
        let count = $this.take_length($arena);
        let res = &$arena.$tys[$this.$tys as usize..][..count as usize];
        $this.$tys += count as u32;
        res
    }};
}

impl OperationArenaPosition {
    pub fn new() -> Self {
        Self {
            u8s: 0,
            u16s: 0,
            u32s: 0,
            registers: 0,
            offsets: 0,
            numbers: 0,
            strings: 0,
            string_arrays: 0,
            lengths: 0,
        }
    }

    pub fn take_element<'a, 'snr>(
        &mut self,
        arena: &'a OperationArena<'snr>,
        element: OperationElement,
    ) -> OperationElementRepr<'a, 'snr> {
        use OperationElementRepr::*;

        match element {
            OperationElement::U8 => U8(self.take_u8(arena)),
            OperationElement::U16 => U16(self.take_u16(arena)),
            OperationElement::U32 => U32(self.take_u32(arena)),
            OperationElement::Operation => Operation(self.take_u8(arena)),
            OperationElement::Condition => Condition(self.take_u8(arena)),
            OperationElement::Expression => {
                let (expressions, numbers) = self.take_expression(arena);
                Expression(expressions, numbers)
            }
            OperationElement::Register => Register(self.take_register(arena)),
            OperationElement::RegisterArray(kind) => {
                RegisterArray(kind, self.take_registers(arena))
            }
            OperationElement::Offset => Offset(self.take_offset(arena)),
            OperationElement::OffsetArray(kind) => OffsetArray(kind, self.take_offsets(arena)),
            OperationElement::Number => Number(self.take_number(arena)),
            OperationElement::OptionalNumber => OptionalNumber(self.take_optional_number(arena)),
            OperationElement::NumberArray(kind) => {
                NumberArray(kind, NumberArrayKind::Dense, self.take_numbers(arena))
            }
            OperationElement::PadNumberArray(kind) => {
                NumberArray(kind, NumberArrayKind::Padded, self.take_numbers(arena))
            }
            OperationElement::BitmaskNumberArray => {
                let mask = self.take_u8(arena);
                let numbers = self.take_numbers(arena);
                BitmaskNumberArray(mask, numbers)
            }
            OperationElement::String(kind) => String(kind, self.take_string(arena)),
            OperationElement::StringArray(kind) => StringArray(kind, self.take_string_array(arena)),
            OperationElement::HiguSuiWipeArg => {
                let b1 = self.take_u8(arena);
                let b2 = self.take_u8(arena);
                let numbers = self.take_numbers(arena);
                HiguSuiWipeArg(b1, b2, numbers)
            }
        }
    }

    // primitives
    fn take_u8(&mut self, arena: &OperationArena) -> u8 {
        take!(self, arena, u8s)
    }
    fn take_u16(&mut self, arena: &OperationArena) -> u16 {
        take!(self, arena, u16s)
    }
    fn take_u32(&mut self, arena: &OperationArena) -> u32 {
        take!(self, arena, u32s)
    }
    fn take_register(&mut self, arena: &OperationArena) -> Register {
        take!(self, arena, registers)
    }
    fn take_registers<'a>(&mut self, arena: &'a OperationArena) -> &'a [Register] {
        take!(self, arena, @multiple registers)
    }
    fn take_offset(&mut self, arena: &OperationArena) -> Offset {
        take!(self, arena, offsets)
    }
    fn take_offsets<'a>(&mut self, arena: &'a OperationArena) -> &'a [Offset] {
        take!(self, arena, @multiple offsets)
    }
    fn take_number(&mut self, arena: &OperationArena) -> Number {
        take!(self, arena, numbers)
    }
    fn take_numbers<'a>(&mut self, arena: &'a OperationArena) -> &'a [Number] {
        take!(self, arena, @multiple numbers)
    }
    fn take_string<'snr>(&mut self, arena: &OperationArena<'snr>) -> &'snr [u8] {
        take!(self, arena, strings)
    }
    fn take_string_array<'snr>(&mut self, arena: &OperationArena<'snr>) -> &'snr [u8] {
        take!(self, arena, string_arrays)
    }
    fn take_length(&mut self, arena: &OperationArena) -> u16 {
        take!(self, arena, lengths)
    }

    // compounds
    fn take_expression<'a>(&mut self, arena: &'a OperationArena) -> (&'a [u8], &'a [Number]) {
        let expressions = take!(self, arena, @multiple u8s);
        let numbers = self.take_numbers(arena);

        (expressions, numbers)
    }

    fn take_optional_number(&mut self, arena: &OperationArena) -> Option<Number> {
        let number = self.take_number(arena);
        if number == u32::MAX {
            None
        } else {
            Some(number)
        }
    }
}

pub struct OperationElementReprIter<'a, 'snr> {
    elements: std::slice::Iter<'a, OperationElement>,
    arena: &'a OperationArena<'snr>,
    position: OperationArenaPosition,
}

impl<'a, 'snr> OperationElementReprIter<'a, 'snr> {
    pub fn new(elements: &'a [OperationElement], arena: &'a OperationArena<'snr>) -> Self {
        Self {
            elements: elements.iter(),
            arena,
            position: OperationArenaPosition::new(),
        }
    }
}

impl<'a, 'snr> Iterator for OperationElementReprIter<'a, 'snr> {
    type Item = OperationElementRepr<'a, 'snr>;

    fn next(&mut self) -> Option<Self::Item> {
        let &element = self.elements.next()?;
        Some(self.position.take_element(self.arena, element))
    }
}

/// Stores data for all elements of an instruction in a columnar format
///
/// Using this columnar format allows us to quickly iterate over elements of certain types,
/// as well as omit certain elements from parsing, if deemed unnecessary.
#[derive(Debug)]
pub struct OperationArena<'snr> {
    pub u8s: Vec<u8>,
    pub u16s: Vec<u16>,
    pub u32s: Vec<u32>,

    pub registers: Vec<Register>,
    pub offsets: Vec<Offset>,
    pub numbers: Vec<Number>,

    pub strings: Vec<&'snr [u8]>,
    pub string_arrays: Vec<&'snr [u8]>,

    pub lengths: Vec<u16>,
}

impl<'snr> OperationArena<'snr> {
    pub fn new() -> Self {
        Self {
            // capacities chosen by parsing the umineko script and observing the capacities after parsing the whole file
            u8s: Vec::with_capacity(16),
            u16s: Vec::with_capacity(2),
            u32s: Vec::with_capacity(2),
            registers: Vec::with_capacity(8),
            offsets: Vec::with_capacity(64),
            numbers: Vec::with_capacity(16),
            strings: Vec::with_capacity(2),
            string_arrays: Vec::with_capacity(1),
            lengths: Vec::with_capacity(2),
        }
    }

    pub fn clear(&mut self) {
        self.u8s.clear();
        self.u16s.clear();
        self.u32s.clear();
        self.registers.clear();
        self.offsets.clear();
        self.numbers.clear();
        self.strings.clear();
        self.string_arrays.clear();
        self.lengths.clear();
    }

    pub fn iter<'a>(&'a self, schema: &OperationSchema<'a>) -> OperationElementReprIter<'a, 'snr> {
        OperationElementReprIter::new(schema.elements, self)
    }
}
