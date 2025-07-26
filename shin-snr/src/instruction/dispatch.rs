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
        ShinVersion::HigurashiHou => Some(match opcode {
            0x00 => EXIT,        // 0x71000b7ed0
            0x40 => uo,          // 0x71000b7f48
            0x41 => bo,          // 0x71000b81c4
            0x42 => exp,         // 0x71000b83cc
            0x43 => mm,          // 0x71000b894c
            0x44 => gt,          // 0x71000b8a14
            0x45 => st,          // 0x71000b8aec
            0x46 => jc,          // 0x71000b8bc0
            0x47 => j,           // 0x71000b8ce4
            0x48 => gosub,       // 0x71000b8d1c
            0x49 => retsub,      // 0x71000b8d70
            0x4a => jt,          // 0x71000b8d94
            0x4b => gosubt,      // 0x71000b8e1c
            0x4c => rnd,         // 0x71000b8eb8
            0x4d => push,        // 0x71000b8f88
            0x4e => pop,         // 0x71000b8ffc
            0x4f => call,        // 0x71000b90d0
            0x50 => r#return,    // 0x71000b91a0
            0x51 => igt,         // 0x71000b91dc
            0x52 => todo!(),     // 0x71000b9300
            0x53 => todo!(),     // 0x71000b9424
            0x81 => SGET,        // 0x71000b94f4
            0x82 => SSET,        // 0x71000b955c
            0x83 => WAIT,        // 0x71000b95a8
            0x85 => MSGINIT,     // 0x71000b9614
            0x86 => MSGSET,      // 0x71000b9654
            0x87 => MSGWAIT,     // 0x71000b971c
            0x88 => MSGSIGNAL,   // 0x71000b975c
            0x89 => MSGSYNC,     // 0x71000b9794
            0x8a => MSGCLOSE,    // 0x71000b97e0
            0x8b => MSGFACE,     // 0x71000b9844
            0x8c => LOGSET,      // 0x71000b9884
            0x8d => SELECT,      // 0x71000b9904
            0x8e => WIPE,        // 0x71000b9a68
            0x8f => WIPEWAIT,    // 0x71000b9be4
            0x90 => BGMPLAY,     // 0x71000b9c1c
            0x91 => BGMSTOP,     // 0x71000b9c88
            0x92 => BGMVOL,      // 0x71000b9cc8
            0x93 => BGMWAIT,     // 0x71000b9d14
            0x94 => BGMSYNC,     // 0x71000b9d54
            0x95 => SEPLAY,      // 0x71000b9d94
            0x96 => SESTOP,      // 0x71000b9e24
            0x97 => SESTOPALL,   // 0x71000b9e70
            0x98 => SEVOL,       // 0x71000b9eb0
            0x99 => SEPAN,       // 0x71000b9f10
            0x9a => SEWAIT,      // 0x71000b9f70
            0x9b => SEONCE,      // 0x71000b9fbc
            0x9c => VOICEPLAY,   // 0x71000ba034
            0x9d => VOICESTOP,   // 0x71000ba0d4
            0x9e => VOICEWAIT,   // 0x71000ba10c
            0xa0 => SAVEINFO,    // 0x71000ba14c
            0xa1 => AUTOSAVE,    // 0x71000ba1dc
            0xa2 => EVBEGIN,     // 0x71000ba214
            0xa3 => EVEND,       // 0x71000ba254
            0xb0 => TROPHY,      // 0x71000ba28c
            0xc0 => LAYERINIT,   // 0x71000ba2cc
            0xc1 => LAYERLOAD,   // 0x71000ba30c
            0xc2 => LAYERUNLOAD, // 0x71000ba488
            0xc3 => LAYERCTRL,   // 0x71000ba4d4
            0xc4 => LAYERWAIT,   // 0x71000ba644
            0xc5 => LAYERBACK,   // 0x71000ba6e4
            0xc6 => LAYERSWAP,   // 0x71000ba724
            0xc7 => LAYERSELECT, // 0x71000ba770
            0xc8 => MOVIEWAIT,   // 0x71000ba7bc
            0xc9 => FEELICON,    // 0x71000ba808
            0xd0 => TIPSGET,     // 0x71000ba880
            0xd1 => CHARSELECT,  // 0x71000ba918
            0xd2 => OTSUGET,     // 0x71000ba9ac
            0xd3 => CHART,       // 0x71000ba9ec
            0xd4 => SNRSEL,      // 0x71000baaa4
            0xd5 => KAKERA,      // 0x71000baae4
            0xd6 => KAKERAGET,   // 0x71000bab1c
            0xd7 => QUIZ,        // 0x71000babbc
            0xd8 => FAKESELECT,  // 0x71000bac48
            0xd9 => UNLOCK,      // 0x71000bac80
            0xff => DEBUGOUT,    // 0x71000bad38
            _ => return None,
        }),
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
        ShinVersion::Umineko => Some(match opcode {
            0x00 => EXIT,        // 0x71000eb9f0
            0x40 => uo,          // 0x71000eba70
            0x41 => bo,          // 0x71000ebd50
            0x42 => exp,         // 0x71000ec050
            0x43 => mm,          // 0x71000ec680
            0x44 => gt,          // 0x71000ec750
            0x45 => st,          // 0x71000ec830
            0x46 => jc,          // 0x71000ec910
            0x47 => j,           // 0x71000eca90
            0x48 => gosub,       // 0x71000ecad0
            0x49 => retsub,      // 0x71000ecb30
            0x4a => jt,          // 0x71000ecb60
            0x4b => gosubt,      // 0x71000ecbf0
            0x4c => rnd,         // 0x71000ecc90
            0x4d => push,        // 0x71000ecd80
            0x4e => pop,         // 0x71000ece00
            0x4f => call,        // 0x71000ecee0
            0x50 => r#return,    // 0x71000ecfc0
            0x81 => SGET,        // 0x71000ed330
            0x82 => SSET,        // 0x71000ed3a0
            0x83 => WAIT,        // 0x71000ed3f0
            0x85 => MSGINIT,     // 0x71000ed460
            0x86 => MSGSET,      // 0x71000ed4a0
            0x87 => MSGWAIT,     // 0x71000ed560
            0x88 => MSGSIGNAL,   // 0x71000ed5a0
            0x89 => MSGSYNC,     // 0x71000ed5e0
            0x8a => MSGCLOSE,    // 0x71000ed630
            0x8d => SELECT,      // 0x71000ed6a0
            0x8e => WIPE,        // 0x71000ed810
            0x8f => WIPEWAIT,    // 0x71000ed970
            0x90 => BGMPLAY,     // 0x71000ed9b0
            0x91 => BGMSTOP,     // 0x71000eda20
            0x92 => BGMVOL,      // 0x71000eda60
            0x93 => BGMWAIT,     // 0x71000edab0
            0x94 => BGMSYNC,     // 0x71000edaf0
            0x95 => SEPLAY,      // 0x71000edb30
            0x96 => SESTOP,      // 0x71000edbc0
            0x97 => SESTOPALL,   // 0x71000edc10
            0x98 => SEVOL,       // 0x71000edc50
            0x99 => SEPAN,       // 0x71000edcb0
            0x9a => SEWAIT,      // 0x71000edd10
            0x9b => SEONCE,      // 0x71000edd60
            0x9c => VOICEPLAY,   // 0x71000edde0
            0x9d => VOICESTOP,   // 0x71000ede80
            0x9e => VOICEWAIT,   // 0x71000edec0
            0x9f => SYSSE,       // 0x71000edf00
            0xa0 => SAVEINFO,    // 0x71000edf50
            0xa1 => AUTOSAVE,    // 0x71000edfe0
            0xa2 => EVBEGIN,     // 0x71000ee020
            0xa3 => EVEND,       // 0x71000ee060
            0xa4 => RESUMESET,   // 0x71000ee0a0
            0xa5 => RESUME,      // 0x71000ee0e0
            0xa6 => SYSCALL,     // 0x71000ee120
            0xb0 => TROPHY,      // 0x71000ee170
            0xb1 => UNLOCK,      // 0x71000ee1b0
            0xc0 => LAYERINIT,   // 0x71000ee270
            0xc1 => LAYERLOAD,   // 0x71000ee2b0
            0xc2 => LAYERUNLOAD, // 0x71000ee410
            0xc3 => LAYERCTRL,   // 0x71000ee460
            0xc4 => LAYERWAIT,   // 0x71000ee5c0
            0xc5 => LAYERSWAP,   // 0x71000ee670
            0xc6 => LAYERSELECT, // 0x71000ee6c0
            0xc7 => MOVIEWAIT,   // 0x71000ee710
            0xc9 => TRANSSET,    // 0x71000ee760
            0xca => TRANSWAIT,   // 0x71000ee8c0
            0xcb => PAGEBACK,    // 0x71000ee900
            0xcc => PLANESELECT, // 0x71000ee940
            0xcd => PLANECLEAR,  // 0x71000ee980
            0xce => MASKLOAD,    // 0x71000ee9c0
            0xcf => MASKUNLOAD,  // 0x71000eea20
            0xe0 => CHARS,       // 0x71000eea60
            0xe1 => TIPSGET,     // 0x71000eeab0
            0xe2 => QUIZ,        // 0x71000eeb50
            0xe3 => SHOWCHARS,   // 0x71000eebc0
            0xe4 => NOTIFYSET,   // 0x71000eec00
            0xff => DEBUGOUT,    // 0x71000eec40
            _ => return None,
        }),
        ShinVersion::Gerokasu2 => Some(match opcode {
            0x00 => EXIT,     // 0x71000be194
            0x40 => uo,       // 0x71000be204
            0x41 => bo,       // 0x71000be508
            0x42 => exp,      // 0x71000be840
            0x43 => mm,       // 0x71000beea0
            0x44 => gt,       // 0x71000bef60
            0x45 => st,       // 0x71000bf03c
            0x46 => jc,       // 0x71000bf0ec
            0x47 => j,        // 0x71000bf27c
            0x48 => gosub,    // 0x71000bf2ac
            0x49 => retsub,   // 0x71000bf2f8
            0x4a => jt,       // 0x71000bf31c
            0x4b => gosubt,   // 0x71000bf390
            0x4c => rnd,      // 0x71000bf408
            0x4d => push,     // 0x71000bf4f0
            0x4e => pop,      // 0x71000bf560
            0x4f => call,     // 0x71000bf630
            0x50 => r#return, // 0x71000bf6f8
            0x51 => igt,      // 0x71000bf740
            // 0x52 => ins_0x52,  // 0x71000bf860
            // 0x53 => ins_0x53,  // 0x71000bf978
            0x81 => SGET,      // 0x71000bfa50
            0x82 => SSET,      // 0x71000bfab0
            0x83 => WAIT,      // 0x71000bfafc
            0x85 => MSGINIT,   // 0x71000bfb60
            0x86 => MSGSET,    // 0x71000bfba0
            0x87 => MSGWAIT,   // 0x71000bfc5c
            0x88 => MSGSIGNAL, // 0x71000bfc9c
            0x89 => MSGSYNC,   // 0x71000bfcd0
            0x8a => MSGCLOSE,  // 0x71000bfd1c
            0x8b => MSGFACE,   // 0x71000bfd78
            0x8d => SELECT,    // 0x71000bfdc0
            0x8e => WIPE,      // 0x71000bff18
            0x8f => WIPEWAIT,  // 0x71000c0070
            0x90 => BGMPLAY,   // 0x71000c00a4
            0x91 => BGMSTOP,   // 0x71000c0110
            0x92 => BGMVOL,    // 0x71000c0150
            0x93 => BGMWAIT,   // 0x71000c019c
            0x94 => BGMSYNC,   // 0x71000c01dc
            0x95 => SEPLAY,    // 0x71000c021c
            0x96 => SESTOP,    // 0x71000c02ac
            0x97 => SESTOPALL, // 0x71000c02f8
            0x98 => SEVOL,     // 0x71000c0338
            0x99 => SEPAN,     // 0x71000c0398
            0x9a => SEWAIT,    // 0x71000c03f8
            0x9b => SEONCE,    // 0x71000c0444
            0x9c => VOICEPLAY, // 0x71000c04bc
            0x9d => VOICESTOP, // 0x71000c0554
            0x9e => VOICEWAIT, // 0x71000c0588
            0x9f => SYSSE,     // 0x71000c05c8
            0xa0 => SAVEINFO,  // 0x71000c0614
            0xa1 => AUTOSAVE,  // 0x71000c069c
            0xa2 => EVBEGIN,   // 0x71000c06d0
            0xa3 => EVEND,     // 0x71000c0710
            0xa4 => RESUMESET, // 0x71000c0744
            0xa5 => RESUME,    // 0x71000c0778
            0xa6 => SYSCALL,   // 0x71000c07ac
            0xb0 => TROPHY,    // 0x71000c07f8
            // 0xb1 => UNLOCK, // 0x71000c0840
            0xc0 => LAYERINIT,   // 0x71000c08fc
            0xc1 => LAYERLOAD,   // 0x71000c093c
            0xc2 => LAYERUNLOAD, // 0x71000c0a94
            0xc3 => LAYERCTRL,   // 0x71000c0ae0
            0xc4 => LAYERWAIT,   // 0x71000c0c30
            0xc5 => LAYERSWAP,   // 0x71000c0cc4
            0xc6 => LAYERSELECT, // 0x71000c0d10
            0xc7 => MOVIEWAIT,   // 0x71000c0d5c
            0xc9 => TRANSSET,    // 0x71000c0da8
            0xca => TRANSWAIT,   // 0x71000c0f00
            0xcb => PAGEBACK,    // 0x71000c0f40
            0xcc => PLANESELECT, // 0x71000c0f74
            0xcd => PLANECLEAR,  // 0x71000c0fb4
            0xce => MASKLOAD,    // 0x71000c0fe8
            0xcf => MASKUNLOAD,  // 0x71000c1048
            0xe0 => QUIZ,        // 0x71000c107c
            0xff => DEBUGOUT,    // 0x71000c10e0
            _ => return None,
        }),
    }
}
