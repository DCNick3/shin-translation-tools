use std::collections::BTreeSet;

use crate::{
    reactor::{Reactor, StringArraySource, StringSource},
    reader::Reader,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LocationColor {
    #[default]
    Unmarked,
    Byte,
    Short,
    Uint,
    Reg,
    Offset,
    U8String,
    U16String,
    U8StringArray,
    U16StringArray,
}

pub struct LocationPainterReactor<'a> {
    reader: Reader<'a>,
    colors: Vec<LocationColor>,
}

pub struct OffsetLocatorResults {
    pub offset_positions: BTreeSet<u32>,
    pub instruction_offsets: BTreeSet<u32>,
}

impl<'a> LocationPainterReactor<'a> {
    pub fn new(reader: Reader<'a>) -> Self {
        let size = reader.size();
        Self {
            reader,
            colors: vec![LocationColor::Unmarked; size],
        }
    }

    pub fn finish(self) -> Vec<LocationColor> {
        self.colors
    }

    fn mark<R>(&mut self, color: LocationColor, inner: impl FnOnce(&mut Reader<'a>) -> R) -> R {
        let start_position = self.reader.position();
        let result = inner(&mut self.reader);
        let end_position = self.reader.position();
        assert!(end_position >= start_position);
        for i in start_position..end_position {
            self.colors[i as usize] = color;
        }

        result
    }
}

impl<'a> Reactor for LocationPainterReactor<'a> {
    fn byte(&mut self) -> u8 {
        self.mark(LocationColor::Byte, |reader| reader.byte())
    }

    fn short(&mut self) -> u16 {
        self.mark(LocationColor::Short, |reader| reader.short())
    }

    fn uint(&mut self) -> u32 {
        self.mark(LocationColor::Uint, |reader| reader.uint())
    }

    fn reg(&mut self) {
        self.mark(LocationColor::Reg, |reader| reader.reg());
    }

    fn offset(&mut self) {
        self.mark(LocationColor::Offset, |reader| reader.offset());
    }

    fn u8string(&mut self, _fixup: bool, _source: StringSource) {
        self.mark(LocationColor::U8String, |reader| reader.u8string());
    }

    fn u16string(&mut self, _fixup: bool, _source: StringSource) {
        self.mark(LocationColor::U16String, |reader| reader.u16string());
    }

    fn u8string_array(&mut self, _fixup: bool, _source: StringArraySource) {
        self.mark(LocationColor::U8StringArray, |reader| {
            reader.u8string_array()
        });
    }

    fn u16string_array(&mut self, _fixup: bool, _source: StringArraySource) {
        self.mark(LocationColor::U16StringArray, |reader| {
            reader.u16string_array()
        });
    }

    fn instr_start(&mut self) {}
    fn instr_end(&mut self) {}

    fn has_instr(&self) -> bool {
        self.reader.has_instr()
    }

    fn in_location(&self) -> u32 {
        self.reader.position()
    }
}
