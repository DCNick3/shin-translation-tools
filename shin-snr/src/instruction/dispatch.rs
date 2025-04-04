use shin_versions::ShinVersion;

use crate::instruction::Instruction;

pub fn decode_instr(version: ShinVersion, opcode: u8) -> Option<Instruction> {
    use crate::instruction::Instruction::*;
    // TODO: those can probably be smartly merged (need to gather some data first though)
    match version {
        ShinVersion::HigurashiSui => {
            Some(match opcode {
                // ===
                // Instructions
                0x40 => uo,     // 0x810461e3
                0x41 => bo,     // 0x810462a3
                0x42 => exp,    // 0x810463d3
                0x43 => mm,     // 0x81046771
                0x44 => gt,     // 0x810467e1
                0x45 => st,     // 0x8104689b
                0x46 => jc,     // 0x81046957
                0x47 => j,      // 0x81046a4d
                0x48 => gosub,  // 0x81046a69
                0x49 => retsub, // 0x81046a9b
                0x4a => jt,     // 0x81046ab3
                0x4b => gosubt, // 0x81046b1d
                0x4c => rnd,    // 0x81046b9b
                0x4d => push,   // 0x81046c39
                0x4e => pop,    // 0x81046ca7

                // ===
                // Commands
                0x80 => EXIT,      // 0x81046d09
                0x81 => SGET,      // 0x81046d6d
                0x82 => SSET,      // 0x81046de7
                0x83 => WAIT,      // 0x81046e71
                0x84 => KEYWAIT,   // 0x81046ed5
                0x85 => MSGINIT,   // 0x81046f39
                0x86 => MSGSET,    // 0x81046fc3
                0x87 => MSGWAIT,   // 0x8104704d
                0x88 => MSGSIGNAL, // 0x810470b1
                0x89 => MSGCLOSE,  // 0x810470e1
                0x8a => MSGCHECK,  // 0x81047111
                0x8b => MSGQUAKE,  // 0x81047163
                0x8c => LOGSET,    // 0x810471c7
                0x8d => SELECT,    // 0x81047223
                0x8e => WIPE,      // 0x81047349
                0x8f => WIPEWAIT,  // 0x810474d9

                0x90 => BGMPLAY,   // 0x81047509
                0x91 => BGMSTOP,   // 0x81047607
                0x92 => BGMVOL,    // 0x8104766b
                0x93 => BGMWAIT,   // 0x8104770f
                0x94 => BGMSYNC,   // 0x81047773
                0x95 => SEPLAY,    // 0x810477d7
                0x96 => SESTOP,    // 0x81047903
                0x97 => SESTOPALL, // 0x8104798d
                0x98 => SEVOL,     // 0x810479f1
                0x99 => SEWAIT,    // 0x81047ac1
                0x9a => SEONCE,    // 0x81047b4b
                0x9b => UNK9B,     // 0x81047c1b

                0xa0 => SAVEINFO,  // 0x81047ca5
                0xa1 => MOVIE,     // 0x81047d2f
                0xa2 => EVBEGIN,   // 0x81047d93
                0xa3 => EVEND,     // 0x81047df9
                0xa5 => AUTOSAVE,  // 0x81047e2b
                0xa6 => VOICEPLAY, // 0x81047e5d
                0xa7 => VOICEWAIT, // 0x81047f35
                0xaa => TIPSGET,   // 0x81047f9b
                0xac => CHARSEL,   // 0x81048037
                0xad => OTSUGET,   // 0x810480d1
                0xae => CHART,     // 0x81048137
                0xaf => SNRSEL,    // 0x810481e1

                0xb0 => KAKERA,     // 0x81048247
                0xb1 => KAKERAGET,  // 0x81048279
                0xb2 => QUIZ,       // 0x81048337
                0xb3 => FAKESELECT, // 0x81048405
                0xb4 => TROPHY,     // 0x81048437

                0xc0 => LAYERCLEAR, // 0x8104849d
                0xc1 => LAYERLOAD,  // 0x810484cf
                0xc2 => LAYERCTRL,  // 0x810485cb
                0xc3 => LAYERWAIT,  // 0x810486a7
                _ => return None,
            })
        }
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
        ShinVersion::Konosuba => Some(match opcode {
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
            // those exist, but let's hope they are not used
            0x51 => todo!(),
            0x52 => todo!(),
            0x53 => getbupid,

            // ===
            // Commands
            0x80 => SGET,
            0x81 => SSET,
            0x82 => WAIT,
            0x83 => MSGINIT,
            0x84 => MSGSET,
            0x85 => MSGWAIT,
            0x86 => MSGSIGNAL,
            0x87 => MSGCLOSE,
            0x88 => SELECT,
            0x89 => WIPE,
            0x8a => WIPEWAIT,

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

            0xc0 => LAYERINIT,
            0xc1 => LAYERLOAD,
            0xc2 => LAYERUNLOAD,
            0xc3 => LAYERCTRL,
            0xc4 => LAYERWAIT,
            0xc5 => LAYERBACK,
            0xc6 => LAYERSELECT,
            0xc7 => MOVIEWAIT,

            0xe0 => SLEEP,
            0xe1 => VSET,

            _ => return None,
        }),
    }
}
