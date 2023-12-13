use crate::{
    ctx::Ctx,
    reactor::{Reactor, StringArraySource, StringSource},
};

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub enum Instruction {
    // Instructions
    // they do not affect the game state and are internal to the VM
    // these do not seem to change between versions
    // NOTE: not all implemented opcodes are implemented here, because I am lazy
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
    rnd,
    push,
    call,
    r#return,

    // Commands
    // they yield to the game loop and are what affects the game state
    // they can be interpreted differently in different contexts (e.g. running the ADV vs building the log)
    // they do tend to change between versions
    // NOTE: I believe currently all Astral air opcodes are listed here
    EXIT,
    SGET,
    SSET,
    WAIT,
    MSGINIT,
    MSGSET,
    MSGWAIT,
    MSGSIGNAL,
    MSGCLOSE,
    MSGFACE,
    LOGSET,
    SELECT,
    WIPE,
    WIPEWAIT,
    BGMPLAY,
    BGMSTOP,
    BGMVOL,
    BGMWAIT,
    BGMSYNC,
    SEPLAY,
    SESTOP,
    SESTOPALL,
    SEVOL,
    SEPAN,
    SEWAIT,
    SEONCE,
    VOICEPLAY,
    VOICEWAIT,
    SAVEINFO,
    AUTOSAVE,
    EVBEGIN,
    EVEND,
    TROPHY,
    LAYERINIT,
    LAYERLOAD,
    LAYERUNLOAD,
    LAYERCTRL,
    LAYERWAIT,
    LAYERSELECT,
    MOVIEWAIT,
    CHARCLEAR,
    CHARLOAD,
    CHARUNLOAD,
    CHARDISP,
    CHARCTRL,
    CHARWAIT,
    CHARMARK,
    CHARSYNC,
    DEBUGOUT,
}

pub fn decode_instr(opcode: u8) -> Option<Instruction> {
    use Instruction::*;
    Some(match opcode {
        0x00 => EXIT,

        // ===
        // Instructions
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
        0x4c => rnd,
        0x4d => push,
        0x4e => pop,
        0x4f => call,
        0x50 => r#return,
        0x51 => todo!(),
        0x52 => todo!(),

        // ===
        // Commands
        0x81 => SGET,
        0x82 => SSET,
        0x83 => WAIT,
        0x84 => return None,
        0x85 => MSGINIT,
        0x86 => return None,
        // NOTE: !!!!
        // Umineko has this opcode as 0x86
        // something was probably shifted, should check it
        0x87 => MSGSET,
        0x88 => MSGWAIT,
        0x89 => MSGSIGNAL,
        0x8a => MSGCLOSE,
        // Missing in umineko!
        0x8b => MSGFACE,
        // ¿missing? in umi?
        0x8c => LOGSET,
        0x8d => SELECT,
        0x8e => WIPE,
        0x8f => WIPEWAIT,

        // NOTE: this block was not checked against umineko
        0x90 => BGMPLAY,
        0x91 => BGMSTOP,
        0x92 => BGMVOL,
        0x93 => BGMWAIT,
        0x94 => BGMSYNC,
        0x95 => SEPLAY,
        0x96 => SESTOP,
        0x97 => SESTOPALL,
        0x98 => SEVOL,
        0x99 => SEPAN,
        0x9a => SEWAIT,
        0x9b => SEONCE,
        0x9c => VOICEPLAY,
        0x9d => VOICEWAIT,

        0xa0 => SAVEINFO,
        0xa1 => AUTOSAVE,
        0xa2 => EVBEGIN,
        0xa3 => EVEND,

        0xb0 => TROPHY,

        0xc0 => LAYERINIT,
        0xc1 => LAYERLOAD,
        0xc2 => LAYERUNLOAD,
        0xc3 => LAYERCTRL,
        0xc4 => LAYERWAIT,
        0xc7 => LAYERSELECT,
        0xc8 => MOVIEWAIT,

        // ==
        // these commands are not present in umineko at all
        0xd0 => CHARCLEAR,
        0xd1 => CHARLOAD,
        0xd2 => CHARUNLOAD,
        0xd3 => CHARDISP,
        0xd4 => CHARCTRL,
        0xd5 => CHARWAIT,
        0xd6 => CHARMARK,
        0xd7 => CHARSYNC,

        0xff => DEBUGOUT,
        _ => return None,
    })
}

pub fn react_instr<R: Reactor>(ctx: &mut Ctx<R>, instr: Instruction) {
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
        Instruction::MSGINIT => {
            ctx.number();
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
        Instruction::MSGWAIT => {
            ctx.number();
        }
        Instruction::MSGSIGNAL => {}
        Instruction::MSGCLOSE => {
            ctx.byte();
        }
        Instruction::MSGFACE => {
            ctx.number();
        }
        Instruction::LOGSET => {
            ctx.string(StringSource::Logset);
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
        Instruction::WIPEWAIT => {}
        Instruction::BGMPLAY => {
            ctx.number();
            ctx.number();
            ctx.number();
            ctx.number();
        }
        Instruction::BGMSTOP => {
            ctx.number();
        }
        Instruction::BGMVOL => {
            ctx.number();
            ctx.number();
        }
        Instruction::BGMWAIT => {
            ctx.number();
        }
        Instruction::BGMSYNC => {
            ctx.number();
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
        Instruction::SESTOPALL => {
            ctx.number();
        }
        Instruction::SEVOL => {
            ctx.number();
            ctx.number();
            ctx.number();
        }
        Instruction::SEPAN => {
            ctx.number();
            ctx.number();
            ctx.number();
        }
        Instruction::SEWAIT => {
            ctx.number();
            ctx.number();
        }
        Instruction::SEONCE => {
            ctx.number();
            ctx.number();
            ctx.number();
            ctx.number();
        }
        Instruction::VOICEPLAY => {
            ctx.string(StringSource::Voiceplay);
            ctx.number();
            ctx.number();
        }
        Instruction::VOICEWAIT => {
            ctx.number();
        }
        Instruction::SAVEINFO => {
            ctx.number();
            ctx.string(StringSource::Saveinfo);
        }
        Instruction::AUTOSAVE => {}
        Instruction::EVBEGIN => {
            ctx.number();
        }
        Instruction::EVEND => {}
        Instruction::TROPHY => {
            ctx.number();
        }
        Instruction::LAYERINIT => ctx.number(),
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
        Instruction::LAYERSELECT => {
            ctx.number();
            ctx.number();
        }
        Instruction::MOVIEWAIT => {
            ctx.number();
            ctx.number();
        }
        Instruction::CHARCLEAR => {}
        Instruction::CHARLOAD => {
            ctx.number();
            ctx.number();
            ctx.number();
        }
        Instruction::CHARUNLOAD => {
            ctx.number();
        }
        Instruction::CHARDISP => {
            ctx.number();
            ctx.number();
        }
        Instruction::CHARCTRL => {
            ctx.number();
            ctx.number();
            ctx.bitmask_number_array();
        }
        Instruction::CHARWAIT => {
            ctx.number();
            let count = ctx.byte();
            for _ in 0..count {
                ctx.number();
            }
        }
        Instruction::CHARMARK => {
            ctx.number();
            ctx.number();
            ctx.number();
        }
        Instruction::CHARSYNC => {}
        Instruction::DEBUGOUT => {
            ctx.string(StringSource::Dbgout);
            let count = ctx.byte();
            for _ in 0..count {
                ctx.number();
            }
        }
        cmd => {
            panic!("Unimplemented instruction {:?} @ {}", cmd, ctx.debug_loc());
        }
    }
}
