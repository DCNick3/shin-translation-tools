use std::collections::HashSet;

use crate::{
    reactor::{Reactor, StringArraySource, StringSource},
    reader::Reader,
};

// why do we need it, again?
#[allow(dead_code)]
pub struct OffsetValidatorReactor<'a> {
    reader: Reader<'a>,
    referred_offsets: HashSet<u32>,
    instruction_offsets: HashSet<u32>,
}

impl<'a> OffsetValidatorReactor<'a> {
    pub fn new(reader: Reader<'a>) -> Self {
        Self {
            reader,
            referred_offsets: HashSet::new(),
            instruction_offsets: HashSet::new(),
        }
    }
}

impl<'a> Reactor for OffsetValidatorReactor<'a> {
    fn byte(&mut self) -> u8 {
        self.reader.byte()
    }

    fn short(&mut self) -> u16 {
        self.reader.short()
    }

    fn reg(&mut self) {
        self.reader.reg();
    }

    fn offset(&mut self) {
        let offset = self.reader.offset();
        self.referred_offsets.insert(offset);
    }

    fn u8string(&mut self, _fixup: bool, _source: StringSource) {
        self.reader.u8string();
    }

    fn u16string(&mut self, _fixup: bool, _source: StringSource) {
        self.reader.u16string();
    }

    fn u8string_array(&mut self, _fixup: bool, _source: StringArraySource) {
        self.reader.u8string_array();
    }

    fn u16string_array(&mut self, _fixup: bool, _source: StringArraySource) {
        self.reader.u16string_array();
    }

    fn msgid(&mut self) -> u32 {
        self.reader.msgid()
    }

    fn instr_start(&mut self) {
        self.instruction_offsets.insert(self.reader.position());
    }
    fn instr_end(&mut self) {}

    fn has_instr(&self) -> bool {
        self.reader.has_instr()
    }

    fn in_location(&self) -> u32 {
        self.reader.position()
    }
}

impl<'a> OffsetValidatorReactor<'a> {
    pub fn validate(self) -> Result<(), String> {
        let mut invalid_offsets = Vec::new();
        for offset in self.referred_offsets {
            if !self.instruction_offsets.contains(&offset) {
                invalid_offsets.push(offset);
            }
        }

        if invalid_offsets.is_empty() {
            Ok(())
        } else {
            let invalid_offsets_count = invalid_offsets.len();
            let mut err = format!("{} invalid offsets\n", invalid_offsets_count);
            for offset in invalid_offsets.into_iter().take(32) {
                err.push_str(&format!("- 0x{:08x}\n", offset));
            }
            if invalid_offsets_count > 32 {
                err.push_str("...\n");
            }

            err.remove(err.len() - 1);

            Err(err)
        }
    }
}
