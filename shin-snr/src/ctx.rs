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

    pub fn version(&self) -> ShinVersion {
        self.version
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
                let t = self.byte();

                // t=TXXXXXXX
                // T=0 => XXXXXXX is a 7-bit signed constant
                // T=1 => futher processing needed
                if t & 0x80 != 0 {
                    // t=1PPPKKKK
                    let p = (t & 0x70) >> 4;
                    // P=0 => 12-bit signed constant (KKKK denotes the upper 4 bits, lsb is read from the next byte)
                    // P=1 => 20-bit signed constant (KKKK denotes the upper 4 bits, 2 lower bytes are read from the stream)
                    // P=2 => 28-bit signed constante (KKKK denotes the upper 4 bits, 3 lower bytes are read from the stream)
                    // P=3 => 4-bit regular register, KKKK is the index
                    // P=4 => 12-bit regular register, KKKK denotes the upper 4 bits, lsb is read from the next byte
                    // P=5 => 4-bit argument register, KKKK is the index
                    // P=6 => constant 0x80000000 (NOTE: this is not implemented in `shin-core`, as it isn't used in umineko)
                    match p {
                        0 => {
                            self.byte();
                        }
                        1 => {
                            self.byte();
                            self.byte();
                        }
                        2 => {
                            self.byte();
                            self.byte();
                            self.byte();
                        }
                        3 => {}
                        4 => {
                            self.byte();
                        }
                        5 => {}
                        6 => {}
                        _ => {
                            panic!("Unknown NumberSpec type: t=0x{:02x}, P={}", t, p)
                        }
                    }
                } else {
                    // signed 7-bit integer, nothing more to read
                }
            }
        }
    }

    pub fn padnumber(&mut self) {
        match self.version.number_spec_style() {
            NumberSpecStyle::Short => {
                self.short();
            }
            NumberSpecStyle::VarInt => {
                // varints are padded to 4 bytes when used in tables
                self.byte();
                self.byte();
                self.byte();
                self.byte();
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

    pub fn in_location(&self) -> u32 {
        self.reactor.in_location()
    }
}
