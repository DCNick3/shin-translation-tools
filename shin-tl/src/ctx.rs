use shin_versions::{LengthSize, NumberSpecStyle, ShinVersion, StringStyle};

use crate::reactor::{Reactor, StringArraySource, StringSource};

pub struct Ctx<'r, R> {
    reactor: &'r mut R,
    version: ShinVersion,
}

impl<'r, R: Reactor> Ctx<'r, R> {
    pub fn new(reactor: &'r mut R, version: ShinVersion) -> Self {
        Self { reactor, version }
    }

    pub fn byte(&mut self) -> u8 {
        self.reactor.byte()
    }

    pub fn short(&mut self) -> u16 {
        self.reactor.short()
    }

    pub fn reg(&mut self) {
        self.reactor.reg()
    }

    pub fn number(&mut self) {
        match self.version.number_spec_style() {
            NumberSpecStyle::Short => {
                self.short();
            }
            NumberSpecStyle::VarInt => {
                todo!()
            }
        }
    }

    pub fn offset(&mut self) {
        self.reactor.offset()
    }

    pub fn string(&mut self, source: StringSource) {
        let StringStyle { length_size, fixup } = self.version.string_style(source.kind());

        match length_size {
            LengthSize::U8Length => self.reactor.u8string(fixup, source),
            LengthSize::U16Length => self.reactor.u16string(fixup, source),
        }
    }

    pub fn string_array(&mut self, source: StringArraySource) {
        let StringStyle { length_size, fixup } = self.version.string_array_style(source.kind());

        match length_size {
            LengthSize::U8Length => self.reactor.u8string_array(fixup, source),
            LengthSize::U16Length => self.reactor.u16string_array(fixup, source),
        }
    }

    pub fn bitmask_number_array(&mut self) {
        let t = self.reactor.byte();
        for _ in 0..t.count_ones() {
            self.number();
        }
    }

    pub fn msgid(&mut self) -> u32 {
        self.reactor.msgid()
    }

    pub fn instr_start(&mut self) {
        self.reactor.instr_start()
    }
    pub fn instr_end(&mut self) {
        self.reactor.instr_end()
    }
    pub fn has_instr(&self) -> bool {
        self.reactor.has_instr()
    }

    pub fn debug_loc(&self) -> String {
        self.reactor.debug_loc()
    }
}
