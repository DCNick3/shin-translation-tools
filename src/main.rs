trait Reactor {
    fn byte(&mut self) -> u8;
    fn short(&mut self) -> u16;
    fn reg(&mut self);
    fn offset(&mut self);
    fn u8string(&mut self, source: StringSource);
    fn u16string(&mut self, _source: StringSource);
    fn u8string_array(&mut self, source: StringArraySource);
    fn msgid(&mut self) -> u32;

    fn debug_loc(&self) -> String;
}

struct ReaderReactor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reactor for ReaderReactor<'a> {
    fn byte(&mut self) -> u8 {
        let res = self.data[self.pos];
        self.pos += 1;
        res
    }

    fn short(&mut self) -> u16 {
        let res = u16::from_le_bytes([self.data[self.pos], self.data[self.pos + 1]]);
        self.pos += 2;
        res
    }

    fn reg(&mut self) {
        self.short();
    }

    fn offset(&mut self) {
        self.pos += 4;
    }

    fn u8string(&mut self, _source: StringSource) {
        let len = self.byte();
        self.pos += len as usize;
    }

    fn u16string(&mut self, _source: StringSource) {
        let len = self.short();
        self.pos += len as usize;
    }

    fn u8string_array(&mut self, _source: StringArraySource) {
        let len = self.byte();
        self.pos += len as usize;
    }
    fn msgid(&mut self) -> u32 {
        let msgid = u32::from_le_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            0,
        ]);
        self.pos += 3;
        msgid
    }

    fn debug_loc(&self) -> String {
        format!("0x{:08x}", self.pos)
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
enum Version {
    AstralAir,
}

enum NumberImpl {
    Short,
    VarInt,
}

impl Version {
    pub fn number_impl(&self) -> NumberImpl {
        match self {
            Version::AstralAir => NumberImpl::Short,
        }
    }
}

enum StringSource {
    Saveinfo,
    Select,
    Msgset(u32),
    Dbgout,
}

enum StringArraySource {
    Select,
}

struct Ctx<R> {
    reactor: R,
    version: Version,
}

impl<R: Reactor> Ctx<R> {
    fn new(reactor: R, version: Version) -> Self {
        Self { reactor, version }
    }

    fn byte(&mut self) -> u8 {
        self.reactor.byte()
    }

    fn short(&mut self) -> u16 {
        self.reactor.short()
    }

    fn reg(&mut self) {
        self.reactor.reg()
    }

    fn number(&mut self) {
        match self.version.number_impl() {
            NumberImpl::Short => {
                self.short();
            }
            NumberImpl::VarInt => {
                todo!()
            }
        }
    }

    fn offset(&mut self) {
        self.reactor.offset()
    }

    fn string(&mut self, source: StringSource) {
        // TODO: switch on version for string size
        match source {
            StringSource::Saveinfo | StringSource::Select | StringSource::Dbgout => {
                self.reactor.u8string(source)
            }
            StringSource::Msgset(_) => self.reactor.u16string(source),
        }
    }

    fn string_array(&mut self, source: StringArraySource) {
        // TODO: switch on string source/version combinations
        self.reactor.u8string_array(source)
    }

    fn bitmask_number_array(&mut self) {
        let t = self.reactor.byte();
        for _ in 0..t.count_ones() {
            self.number();
        }
    }

    fn msgid(&mut self) -> u32 {
        self.reactor.msgid()
    }

    fn debug_loc(&self) -> String {
        self.reactor.debug_loc()
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
enum Instruction {
    uo,
    bo,
    exp,
    /// Move many
    mm,
    /// Get table (table is array of numbers)
    gt,
    /// Set table (table is array of registers)
    st,
    jc,
    j,
    gosub,
    retsub,
    jt,
    pop,
    gosubt,
    push,
    call,
    r#return,
    EXIT,
    SGET,
    SSET,
    WAIT,
    MSGSET,
    MSGCLOSE,
    SELECT,
    WIPE,
    SEPLAY,
    SESTOP,
    SAVEINFO,
    AUTOSAVE,
    LAYERLOAD,
    LAYERUNLOAD,
    LAYERCTRL,
    LAYERWAIT,
    UNK_D1,
    UNK_D4,
    DBGOUT,
}

fn decode_instr(opcode: u8) -> Option<Instruction> {
    use Instruction::*;
    Some(match opcode {
        0x00 => EXIT,
        0x40 => uo,
        0x41 => bo,
        0x42 => exp,
        0x43 => mm,
        0x44 => gt,
        0x45 => st,
        0x46 => jc,
        0x47 => j,
        0x48 => gosub,
        0x49 => retsub,
        0x4a => jt,
        0x4b => gosubt,
        0x4e => pop,
        0x4d => push,
        0x4f => call,
        0x50 => r#return,
        0x81 => SGET,
        0x82 => SSET,
        0x83 => WAIT,
        // NOTE: !!!!
        // Umineko has this opcode as 0x86
        // something was probably shifted, should check it
        0x87 => MSGSET,
        0x8a => MSGCLOSE,
        0x8d => SELECT,
        0x8e => WIPE,
        0x95 => SEPLAY,
        0x96 => SESTOP,
        0xa0 => SAVEINFO,
        0xa1 => AUTOSAVE,
        0xc1 => LAYERLOAD,
        0xc2 => LAYERUNLOAD,
        0xc3 => LAYERCTRL,
        0xc4 => LAYERWAIT,
        // ==
        // these commands are not present in umineko at all
        0xd1 => UNK_D1,
        0xd4 => UNK_D4,
        // ==
        0xff => DBGOUT,
        _ => return None,
    })
}

fn react_instr<R: Reactor>(ctx: &mut Ctx<R>, instr: Instruction) {
    match instr {
        Instruction::uo => {
            let t = ctx.byte();
            if t & 0x80 != 0 {
                ctx.reg();
                ctx.number();
            } else {
                ctx.reg();
            }
        }
        Instruction::bo => {
            let t = ctx.byte();
            if t & 0x80 != 0 {
                ctx.reg();
                ctx.number();
                ctx.number();
            } else {
                ctx.reg();
                ctx.number();
            }
        }
        Instruction::exp => {
            ctx.reg();
            loop {
                let t = ctx.byte();
                if t == 0x00 {
                    ctx.number();
                }
                if t < 0x20 {
                    continue;
                } else {
                    assert_eq!(t, 0xff, "Unexpected expression byte 0x{:02x}", t);
                    break;
                }
            }
        }
        Instruction::mm => {
            ctx.number();
            let count = ctx.short();
            for _ in 0..count {
                ctx.reg();
            }
        }
        Instruction::gt => {
            ctx.reg();
            ctx.number();
            let number_count = ctx.short();
            // NOTE: in umineko, these are padded to 4 bytes to allow for seeking
            for _ in 0..number_count {
                ctx.number();
            }
        }
        Instruction::st => {
            ctx.number();
            ctx.number();
            let number_count = ctx.short();
            for _ in 0..number_count {
                ctx.reg();
            }
        }
        Instruction::jc => {
            ctx.byte();
            ctx.number();
            ctx.number();
            ctx.offset();
        }
        Instruction::j => {
            ctx.offset();
        }
        Instruction::gosub => {
            ctx.offset();
        }
        Instruction::retsub => {}
        Instruction::jt => {
            ctx.number();
            let offset_count = ctx.short();
            for _ in 0..offset_count {
                ctx.offset();
            }
        }
        Instruction::gosubt => {
            ctx.number();
            let offset_count = ctx.short();
            for _ in 0..offset_count {
                ctx.offset();
            }
        }
        Instruction::push => {
            let push_count = ctx.byte();
            for _ in 0..push_count {
                ctx.number();
            }
        }
        Instruction::pop => {
            let pop_count = ctx.byte();
            for _ in 0..pop_count {
                ctx.reg();
            }
        }
        Instruction::call => {
            ctx.offset();
            let arg_count = ctx.byte();
            for _ in 0..arg_count {
                ctx.number();
            }
        }
        Instruction::r#return => {}
        Instruction::EXIT => {
            ctx.byte();
            ctx.number();
        }
        Instruction::SGET => {
            ctx.reg();
            ctx.number();
        }
        Instruction::SSET => {
            ctx.number();
            ctx.number();
        }
        Instruction::WAIT => {
            ctx.byte();
            ctx.number();
        }
        Instruction::MSGSET => {
            let msgid = ctx.msgid();
            ctx.byte();
            // NOTE: these numbers are NOT present in umineko
            ctx.number();
            ctx.number();
            ctx.string(StringSource::Msgset(msgid));
        }
        Instruction::MSGCLOSE => {
            ctx.byte();
        }
        Instruction::SELECT => {
            ctx.short();
            ctx.short();
            ctx.reg();
            ctx.number();
            ctx.string(StringSource::Select);
            ctx.string_array(StringArraySource::Select);
        }
        Instruction::WIPE => {
            ctx.number();
            ctx.number();
            ctx.number();
            ctx.bitmask_number_array();
        }
        Instruction::SEPLAY => {
            ctx.number();
            ctx.number();
            ctx.number();
            ctx.number();
            ctx.number();
            ctx.number();
        }
        Instruction::SESTOP => {
            ctx.number();
            ctx.number();
        }
        Instruction::SAVEINFO => {
            ctx.number();
            ctx.string(StringSource::Saveinfo);
        }
        Instruction::AUTOSAVE => {}
        Instruction::LAYERLOAD => {
            ctx.number();
            ctx.number();
            ctx.number();
            ctx.bitmask_number_array();
        }
        Instruction::LAYERUNLOAD => {
            ctx.number();
            ctx.number();
        }
        Instruction::LAYERCTRL => {
            ctx.number();
            ctx.number();
            ctx.bitmask_number_array();
        }
        Instruction::LAYERWAIT => {
            ctx.number();
            let count = ctx.byte();
            for _ in 0..count {
                ctx.number();
            }
        }
        Instruction::UNK_D1 => {
            ctx.number();
            ctx.number();
            ctx.number();
        }
        Instruction::UNK_D4 => {
            ctx.number();
            ctx.number();
            ctx.bitmask_number_array();
        }
        Instruction::DBGOUT => {
            ctx.string(StringSource::Dbgout);
            let count = ctx.byte();
            for _ in 0..count {
                ctx.number();
            }
        }
    }
}

fn react<R: Reactor>(ctx: &mut Ctx<R>) {
    loop {
        let opcode = ctx.byte();
        let Some(instr) = decode_instr(opcode) else {
            panic!(
                "Unknown opcode 0x{opcode:02x} ({opcode}) @ {}",
                ctx.debug_loc()
            );
        };
        react_instr(ctx, instr);
        println!("{:?}", instr);
        // TODO: exit condition
    }
}

fn main() {
    let snr_file = std::fs::read("main.snr").unwrap();

    assert_eq!(&snr_file[0..4], b"SNR ");
    let code_offset = u32::from_le_bytes([
        snr_file[0x20],
        snr_file[0x21],
        snr_file[0x22],
        snr_file[0x23],
    ]);

    let reader = ReaderReactor {
        data: &snr_file,
        pos: code_offset as usize,
    };

    let mut ctx = Ctx::new(reader, Version::AstralAir);
    react(&mut ctx);
}
