use shin_versions::ShinVersion;

use crate::{
    ctx::Ctx,
    reactor::{Reactor, StringArraySource, StringSource},
};

// we break the rust naming rules in order to
// 1. match game's COMMAND names
// 2. distinguish instructions (lowercase) from commands (UPPERCASE)
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Debug, Copy, Clone)]
pub enum Instruction {
    // Instructions
    // they do not affect the game state and are internal to the VM
    // these do not seem to change between versions
    // NOTE: not all implemented opcodes are implemented here, because I am lazy
    /// Unary operation
    uo,
    /// Binary operation
    bo,
    /// Expression (encoded in RPN)
    exp,
    /// Move many
    mm,
    /// Get table (table is array of numbers)
    gt,
    /// Set table (table is array of registers)
    st,
    /// Conditional jump
    jc,
    /// Unconditional jump
    j,
    /// Call subroutine
    gosub,
    /// Return from subroutine
    retsub,
    /// Jump table
    jt,
    pop,
    /// Call subroutine table
    gosubt,
    rnd,
    push,
    /// Call function
    call,
    /// Return from function
    r#return,
    /// Inverted get table (index of matching entry in table)
    igt,

    /// Get the character ID corresponding to the passed Bustup info
    getbupid,

    // Commands
    // they yield to the game loop and are what affects the game state
    // they can be interpreted differently in different contexts (e.g. running the ADV vs building the log)
    // they do tend to change between versions
    // NOTE: I believe currently all Astral air opcodes are listed here
    EXIT,
    SGET,
    SSET,
    WAIT,
    // only on old
    KEYWAIT,
    MSGINIT,
    MSGSET,
    MSGWAIT,
    MSGSIGNAL,
    MSGSYNC,
    MSGCLOSE,
    MSGFACE,
    MSGCHECK,
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
    // Unknown instuction with no semantics (like, really, the handlers are empty)
    // but it has some arguments. Probably a leftover from even older version...
    UNK9B,
    VOICEPLAY,
    VOICESTOP,
    VOICEWAIT,

    SAVEINFO,
    // only on old
    MOVIE,
    AUTOSAVE,
    EVBEGIN,
    EVEND,
    RESUMESET,
    RESUME,

    TROPHY,
    UNLOCK,

    // only on old
    LAYERCLEAR,
    LAYERINIT,
    LAYERLOAD,
    LAYERUNLOAD,
    LAYERCTRL,
    LAYERWAIT,
    LAYERSWAP,
    LAYERSELECT,
    MOVIEWAIT,
    // "new" plane-related commands
    TRANSSET,
    TRANSWAIT,
    PAGEBACK,
    PLANESELECT,
    PLANECLEAR,
    MASKLOAD,
    MASKUNLOAD,

    // Game-specific commands
    // Alias Carnival
    ICOGET,
    STAGEINFO,
    ICOCHK,
    EMOTEWAIT,
    NAMED,
    BACKINIT,

    // White Eternity
    CHARCLEAR,
    CHARLOAD,
    CHARUNLOAD,
    CHARDISP,
    CHARCTRL,
    CHARWAIT,
    CHARMARK,
    CHARSYNC,

    // DC4
    CHATSET,

    // this is the last thing in the opcode space
    DEBUGOUT,
}

pub fn decode_instr(version: ShinVersion, opcode: u8) -> Option<Instruction> {
    use Instruction::*;
    // TODO: those can probably be smartly merged (need to gather some data first though)
    match version {
        ShinVersion::AliasCarnival => {
            // TODO: can opcode tables be part of shin-version?
            Some(match opcode {
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
                // no 0x51 and 0x52

                // ===
                // Commands
                0x80 => EXIT,
                0x81 => SGET,
                0x82 => SSET,
                0x83 => WAIT,
                0x84 => KEYWAIT,
                0x85 => MSGINIT,
                0x86 => MSGFACE,
                0x87 => MSGSET,
                0x88 => MSGWAIT,
                0x89 => MSGSIGNAL,
                0x8a => MSGCLOSE,
                // WTF is MSGCHECK
                0x8b => MSGCHECK,
                0x8c => LOGSET,
                0x8d => SELECT,
                0x8e => WIPE,
                0x8f => WIPEWAIT,

                0x90 => BGMPLAY,
                0x91 => BGMSTOP,
                0x92 => BGMVOL,
                0x93 => BGMWAIT,
                0x94 => BGMSYNC,
                0x95 => SEPLAY,
                0x96 => SESTOP,
                0x97 => SESTOPALL,
                0x98 => SEVOL,
                // no SEPAN!
                // 0x99 => SEPAN,
                0x99 => SEWAIT,
                0x9a => SEONCE,
                // ADV's handler for this command is empty...
                // VOICEPLAY and VOICEWAIT are in the next block
                0x9b => UNK9B,

                0xa0 => SAVEINFO,
                // 0xa1 => AUTOSAVE,
                0xa1 => MOVIE,
                0xa2 => EVBEGIN,
                0xa3 => EVEND,
                // no 0xa4
                0xa5 => AUTOSAVE,
                0xa6 => VOICEPLAY,
                0xa7 => VOICEWAIT,

                0xb0 => TROPHY,
                0xb1 => ICOGET,
                0xb2 => STAGEINFO,
                0xb3 => ICOCHK,

                // layer WHAT?
                0xc0 => LAYERCLEAR,
                0xc1 => LAYERLOAD,
                // no LAYERUNLOAD
                0xc2 => LAYERCTRL,
                0xc3 => LAYERWAIT,
                0xc4 => EMOTEWAIT,
                0xc5 => NAMED,
                0xc6 => BACKINIT,

                0xff => DEBUGOUT,
                _ => return None,
            })
        }
        ShinVersion::WhiteEternity => {
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
                // TODO: why is there a hole here?
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
        ShinVersion::DC4 => Some(match opcode {
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
            0x4c => rnd,
            0x4d => push,
            0x4e => pop,
            0x4f => call,
            0x50 => r#return,
            0x51 => igt,
            // also present in umineko, but, thankfully, not used
            0x53 => getbupid,
            //
            0x81 => SGET,
            0x82 => SSET,
            0x83 => WAIT,
            // 0x84 unused
            0x85 => MSGINIT,
            0x86 => MSGSET,
            0x87 => MSGWAIT,
            0x88 => MSGSIGNAL,
            0x89 => MSGSYNC,
            0x8a => MSGCLOSE,
            0x8b => MSGFACE,
            // 0x8c unused
            0x8d => SELECT,
            0x8e => WIPE,
            0x8f => WIPEWAIT,
            //
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
            0x9d => VOICESTOP,
            0x9e => VOICEWAIT,
            //
            0xa0 => SAVEINFO,
            0xa1 => AUTOSAVE,
            0xa2 => EVBEGIN,
            0xa3 => EVEND,
            0xa4 => RESUMESET,
            0xa5 => RESUME,
            //
            0xb0 => TROPHY,
            0xb1 => UNLOCK,
            //
            0xc0 => LAYERINIT,
            0xc1 => LAYERLOAD,
            0xc2 => LAYERUNLOAD,
            0xc3 => LAYERCTRL,
            0xc4 => LAYERWAIT,
            0xc5 => LAYERSWAP,
            0xc6 => LAYERSELECT,
            0xc7 => MOVIEWAIT,
            // 0xc8 unused
            0xc9 => TRANSSET,
            0xca => TRANSWAIT,
            0xcb => PAGEBACK,
            0xcc => PLANESELECT,
            0xcd => PLANECLEAR,
            0xce => MASKLOAD,
            0xcf => MASKUNLOAD,
            0xd0 => CHATSET,
            0xff => DEBUGOUT,
            _ => return None,
        }),
        ShinVersion::Konosuba => {
            todo!()
        }
    }
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
                    assert_eq!(t, 0xff, "Unexpected expression byte 0x{t:02x}");
                    break;
                }
            }
        }
        Instruction::mm => {
            ctx.number();
            let count = ctx.mm_gt_st_length();
            for _ in 0..count {
                ctx.reg();
            }
        }
        Instruction::gt => {
            ctx.reg();
            ctx.number();
            let number_count = ctx.mm_gt_st_length();
            // NOTE: in umineko, these are padded to 4 bytes to allow for seeking
            // with NumberSpecStyle::Short, this is not necessary
            for _ in 0..number_count {
                ctx.padnumber();
            }
        }
        Instruction::st => {
            ctx.number();
            ctx.number();
            let number_count = ctx.mm_gt_st_length();
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
        Instruction::rnd => {
            ctx.reg();
            ctx.number();
            ctx.number();
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
        Instruction::igt => {
            ctx.reg();
            ctx.number();
            ctx.number();
            let number_count = ctx.short();
            for _ in 0..number_count {
                ctx.padnumber();
            }
        }
        Instruction::getbupid => {
            ctx.reg();
            ctx.number();
            ctx.number();
        }

        Instruction::EXIT => match ctx.version() {
            ShinVersion::AliasCarnival => {
                ctx.number();
            }
            ShinVersion::WhiteEternity | ShinVersion::DC4 => {
                ctx.byte();
                ctx.number();
            }
            ShinVersion::Konosuba => {
                todo!()
            }
        },
        Instruction::SGET => {
            ctx.reg();
            ctx.number();
        }
        Instruction::SSET => {
            ctx.number();
            ctx.number();
        }
        Instruction::WAIT => match ctx.version() {
            ShinVersion::AliasCarnival => ctx.number(),
            ShinVersion::WhiteEternity | ShinVersion::DC4 => {
                ctx.byte();
                ctx.number();
            }
            ShinVersion::Konosuba => {
                todo!()
            }
        },
        Instruction::KEYWAIT => match ctx.version() {
            ShinVersion::AliasCarnival => ctx.number(),
            ShinVersion::WhiteEternity | ShinVersion::DC4 => unreachable!(),
            ShinVersion::Konosuba => {
                todo!()
            }
        },
        Instruction::MSGINIT => match ctx.version() {
            ShinVersion::AliasCarnival | ShinVersion::WhiteEternity => {
                ctx.number();
                ctx.number();
            }
            ShinVersion::DC4 => {
                ctx.number();
            }
            ShinVersion::Konosuba => {
                todo!()
            }
        },
        Instruction::MSGSET => {
            let msgid = ctx.uint() & 0xffffff;

            match ctx.version() {
                ShinVersion::AliasCarnival => {
                    ctx.number();
                }
                ShinVersion::WhiteEternity => {
                    // NOTE: these numbers are NOT present in umineko
                    ctx.number();
                    ctx.number();
                }
                ShinVersion::DC4 => {
                    ctx.number();
                }
                ShinVersion::Konosuba => {
                    todo!()
                }
            }

            ctx.string(StringSource::Msgset(msgid));
        }
        Instruction::MSGWAIT => {
            ctx.number();
        }
        Instruction::MSGSIGNAL => {}
        Instruction::MSGSYNC => {
            ctx.number();
            ctx.number();
        }
        Instruction::MSGCLOSE => match ctx.version() {
            ShinVersion::AliasCarnival => {}
            ShinVersion::WhiteEternity | ShinVersion::DC4 => {
                ctx.byte();
            }
            ShinVersion::Konosuba => {
                todo!()
            }
        },
        Instruction::MSGCHECK => {
            ctx.uint();
        }
        Instruction::MSGFACE => match ctx.version() {
            ShinVersion::AliasCarnival => {
                ctx.number();
                ctx.number();
                ctx.number();
                ctx.number();
            }
            ShinVersion::WhiteEternity | ShinVersion::DC4 => ctx.number(),
            ShinVersion::Konosuba => {
                todo!()
            }
        },
        Instruction::LOGSET => match ctx.version() {
            ShinVersion::AliasCarnival => {
                ctx.number();
                ctx.string(StringSource::Logset)
            }
            ShinVersion::WhiteEternity => ctx.string(StringSource::Logset),
            ShinVersion::DC4 => {
                unreachable!()
            }
            ShinVersion::Konosuba => {
                todo!()
            }
        },
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
        Instruction::SEPLAY => match ctx.version() {
            ShinVersion::AliasCarnival => {
                // 5x number
                ctx.number();
                ctx.number();
                ctx.number();
                ctx.number();
                ctx.number();
            }
            ShinVersion::WhiteEternity => {
                // 6x number
                ctx.number();
                ctx.number();
                ctx.number();
                ctx.number();
                ctx.number();
                ctx.number();
            }
            ShinVersion::DC4 => {
                // 7x number
                ctx.number();
                ctx.number();
                ctx.number();
                ctx.number();
                ctx.number();
                ctx.number();
                ctx.number();
            }
            ShinVersion::Konosuba => {
                todo!()
            }
        },
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
        Instruction::SEONCE => match ctx.version() {
            ShinVersion::AliasCarnival => {
                ctx.number();
                ctx.number();
                ctx.number();
            }
            ShinVersion::WhiteEternity => {
                ctx.number();
                ctx.number();
                ctx.number();
                ctx.number();
            }
            ShinVersion::DC4 => {
                ctx.number();
                ctx.number();
                ctx.number();
                ctx.number();
                ctx.number();
            }
            ShinVersion::Konosuba => {
                todo!()
            }
        },
        Instruction::UNK9B => {
            ctx.number();
            ctx.number();
        }
        Instruction::VOICEPLAY => {
            ctx.string(StringSource::Voiceplay);
            ctx.number();
            ctx.number();
        }
        Instruction::VOICESTOP => {}
        Instruction::VOICEWAIT => {
            ctx.number();
        }
        Instruction::SAVEINFO => {
            ctx.number();
            ctx.string(StringSource::Saveinfo);
        }
        Instruction::MOVIE => {
            ctx.number();
        }
        Instruction::AUTOSAVE => {}
        Instruction::EVBEGIN => {
            ctx.number();
        }
        Instruction::EVEND => {}
        Instruction::RESUMESET => {}
        Instruction::RESUME => {}
        Instruction::TROPHY => {
            ctx.number();
        }
        Instruction::UNLOCK => {
            ctx.number();
        }
        Instruction::LAYERCLEAR => {}
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
        Instruction::LAYERSWAP => {
            ctx.number();
            ctx.number();
        }
        Instruction::LAYERSELECT => {
            ctx.number();
            ctx.number();
        }
        Instruction::MOVIEWAIT => {
            ctx.number();
            ctx.number();
        }

        Instruction::TRANSSET => {
            ctx.number();
            ctx.number();
            ctx.number();
            ctx.bitmask_number_array();
        }
        Instruction::TRANSWAIT => {
            ctx.number();
        }
        Instruction::PAGEBACK => {}
        Instruction::PLANESELECT => {
            ctx.number();
        }
        Instruction::PLANECLEAR => {}
        Instruction::MASKLOAD => {
            ctx.number();
            ctx.number();
            ctx.number();
        }
        Instruction::MASKUNLOAD => {}

        // Alias Carnival
        Instruction::ICOGET => {
            let len = ctx.byte();
            for _ in 0..len {
                ctx.number();
            }
        }
        Instruction::STAGEINFO => {
            ctx.string(StringSource::Stageinfo);
            ctx.string(StringSource::Stageinfo);
        }
        Instruction::ICOCHK => {
            // not that sure what this short actually represents
            ctx.short();
        }
        // Instruction::EMOTEWAIT => todo!(),
        Instruction::NAMED => {
            ctx.byte();
            ctx.string(StringSource::Named);
        }
        Instruction::BACKINIT => {}

        // White Eternity
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

        // DC4
        Instruction::CHATSET => ctx.string(StringSource::Chatset),

        Instruction::DEBUGOUT => match ctx.version() {
            ShinVersion::AliasCarnival => {
                ctx.string(StringSource::Dbgout);
                ctx.short();
            }
            ShinVersion::WhiteEternity | ShinVersion::DC4 => {
                ctx.string(StringSource::Dbgout);
                let count = ctx.byte();
                for _ in 0..count {
                    ctx.number();
                }
            }
            ShinVersion::Konosuba => {
                todo!()
            }
        },
        #[allow(unreachable_patterns)]
        cmd => {
            panic!(
                "Unimplemented instruction {:?} @ 0x{:08x}",
                cmd,
                ctx.in_location()
            );
        }
    }
}
