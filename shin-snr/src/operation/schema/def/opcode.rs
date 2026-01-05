use shin_versions::ShinVersion;

use crate::operation::schema::opcode::{Command, Instruction, Opcode};

pub const fn get_opcode_name(version: ShinVersion, opcode: u8) -> Option<Opcode> {
    use Command::*;
    use Instruction::*;
    // TODO: those can probably be smartly merged (need to gather some data first though)
    match version {
        ShinVersion::HigurashiSui => Some(if opcode < 0x80 {
            // ===
            // Instructions
            Opcode::Instruction(match opcode {
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
                _ => return None,
            })
        } else {
            // ===
            // Commands
            Opcode::Command(match opcode {
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

                0xa0 => SAVEINFO, // 0x81047ca5
                0xa1 => MOVIE,    // 0x81047d2f
                0xa2 => EVBEGIN,  // 0x81047d93
                0xa3 => EVEND,    // 0x81047df9

                0xa5 => AUTOSAVE,  // 0x81047e2b
                0xa6 => VOICEPLAY, // 0x81047e5d
                0xa7 => VOICEWAIT, // 0x81047f35

                0xaa => TIPSGET, // 0x81047f9b

                0xac => CHARSEL,    // 0x81048037
                0xad => OTSUGET,    // 0x810480d1
                0xae => CHART,      // 0x81048137
                0xaf => SNRSEL,     // 0x810481e1
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
        }),
        ShinVersion::AliasCarnival => Some(if opcode < 0x80 {
            // ===
            // Instructions
            Opcode::Instruction(match opcode {
                0x40 => uo,       // 0x810431a1
                0x41 => bo,       // 0x810432af
                0x42 => exp,      // 0x81043461
                0x43 => mm,       // 0x81043877
                0x44 => gt,       // 0x81043925
                0x45 => st,       // 0x81043a31
                0x46 => jc,       // 0x81043b3b
                0x47 => j,        // 0x81043c65
                0x48 => gosub,    // 0x81043c81
                0x49 => retsub,   // 0x81043cb3
                0x4a => jt,       // 0x81043ccb
                0x4b => gosubt,   // 0x81043d4f
                0x4c => rnd,      // 0x81043de7
                0x4d => push,     // 0x81043ed3
                0x4e => pop,      // 0x81043f5b
                0x4f => call,     // 0x81043fd7
                0x50 => r#return, // 0x8104409b
                _ => return None,
            })
        } else {
            // ===
            // Commands
            Opcode::Command(match opcode {
                0x80 => EXIT,      // 0x810440c7
                0x81 => SGET,      // 0x81044145
                0x82 => SSET,      // 0x810441d9
                0x83 => WAIT,      // 0x81044295
                0x84 => KEYWAIT,   // 0x81044313
                0x85 => MSGINIT,   // 0x81044391
                0x86 => MSGFACE,   // 0x8104444d
                0x87 => MSGSET,    // 0x81044599
                0x88 => MSGWAIT,   // 0x8104466d
                0x89 => MSGSIGNAL, // 0x810446eb
                0x8a => MSGCLOSE,  // 0x8104471b
                0x8b => MSGCHECK,  // 0x8104474b
                0x8c => LOGSET,    // 0x8104479d
                0x8d => SELECT,    // 0x81044841
                0x8e => WIPE,      // 0x8104496b
                0x8f => WIPEWAIT,  // 0x81044acd
                0x90 => BGMPLAY,   // 0x81044afd
                0x91 => BGMSTOP,   // 0x81044c63
                0x92 => BGMVOL,    // 0x81044ce1
                0x93 => BGMWAIT,   // 0x81044db7
                0x94 => BGMSYNC,   // 0x81044e35
                0x95 => SEPLAY,    // 0x81044eb3
                0x96 => SESTOP,    // 0x81045061
                0x97 => SESTOPALL, // 0x8104511d
                0x98 => SEVOL,     // 0x8104519b
                0x99 => SEWAIT,    // 0x810452b9
                0x9a => SEONCE,    // 0x81045375
                0x9b => UNK9B,     // 0x81045493

                0xa0 => SAVEINFO, // 0x8104554f
                0xa1 => MOVIE,    // 0x810455f3
                0xa2 => EVBEGIN,  // 0x81045671
                0xa3 => EVEND,    // 0x810456f1

                0xa5 => AUTOSAVE,  // 0x81045723
                0xa6 => VOICEPLAY, // 0x81045755
                0xa7 => VOICEWAIT, // 0x8104585f

                0xb0 => TROPHY,    // 0x810458df
                0xb1 => ICOGET,    // 0x8104595f
                0xb2 => STAGEINFO, // 0x81045a17
                0xb3 => ICOCHK,    // 0x81045a93

                0xc0 => LAYERCLEAR, // 0x81045ae7
                0xc1 => LAYERLOAD,  // 0x81045b19
                0xc2 => LAYERCTRL,  // 0x81045c7d
                0xc3 => LAYERWAIT,  // 0x81045da7
                0xc4 => EMOTEWAIT,  // 0x81045ea5
                0xc5 => NAMED,      // 0x81045f25
                0xc6 => BACKINIT,   // 0x81045f91

                0xff => DEBUGOUT, // 0x81045fc3
                _ => return None,
            })
        }),
        ShinVersion::WhiteEternity => Some(if opcode != 0 && opcode < 0x80 {
            // ===
            // Instructions
            Opcode::Instruction(match opcode {
                0x40 => uo,       // 0x81053e37
                0x41 => bo,       // 0x81054159
                0x42 => exp,      // 0x810543c1
                0x43 => mm,       // 0x810549b5
                0x44 => gt,       // 0x81054a5f
                0x45 => st,       // 0x81054b57
                0x46 => jc,       // 0x81054c49
                0x47 => j,        // 0x81054d83
                0x48 => gosub,    // 0x81054db3
                0x49 => retsub,   // 0x81054dfb
                0x4a => jt,       // 0x81054e15
                0x4b => gosubt,   // 0x81054e93
                0x4c => rnd,      // 0x81054f35
                0x4d => push,     // 0x81055027
                0x4e => pop,      // 0x810550af
                0x4f => call,     // 0x81055127
                0x50 => r#return, // 0x81055205
                // TODO: those two are unverified
                0x51 => fmt,  // 0x81055237
                0x52 => fnmt, // 0x81055367
                _ => return None,
            })
        } else {
            // ===
            // Commands
            Opcode::Command(match opcode {
                0x00 => EXIT, // 0x81053d93

                0x81 => SGET, // 0x81055463
                0x82 => SSET, // 0x810554fb
                0x83 => WAIT, // 0x810555c1

                0x85 => MSGINIT, // 0x81055655

                0x87 => MSGSET,    // 0x8105571b
                0x88 => MSGWAIT,   // 0x81055833
                0x89 => MSGSIGNAL, // 0x810558b3
                0x8a => MSGCLOSE,  // 0x810558e7
                0x8b => MSGFACE,   // 0x81055941
                0x8c => LOGSET,    // 0x810559c1
                0x8d => SELECT,    // 0x81055a1b
                0x8e => WIPE,      // 0x81055b4d
                0x8f => WIPEWAIT,  // 0x81055ce5
                0x90 => BGMPLAY,   // 0x81055d19
                0x91 => BGMSTOP,   // 0x81055e67
                0x92 => BGMVOL,    // 0x81055ee7
                0x93 => BGMWAIT,   // 0x81055fad
                0x94 => BGMSYNC,   // 0x8105602d
                0x95 => SEPLAY,    // 0x810560ad
                0x96 => SESTOP,    // 0x81056283
                0x97 => SESTOPALL, // 0x81056349
                0x98 => SEVOL,     // 0x810563c9
                0x99 => SEPAN,     // 0x810564d3
                0x9a => SEWAIT,    // 0x810565dd
                0x9b => SEONCE,    // 0x810566a3
                0x9c => VOICEPLAY, // 0x810567f1
                0x9d => VOICEWAIT, // 0x810568d9

                0xa0 => SAVEINFO, // 0x81056959
                0xa1 => AUTOSAVE, // 0x810569fb
                0xa2 => EVBEGIN,  // 0x81056a2f
                0xa3 => EVEND,    // 0x81056ab1

                0xb0 => TROPHY, // 0x81056ae7

                0xc0 => LAYERINIT,   // 0x81056b69
                0xc1 => LAYERLOAD,   // 0x81056beb
                0xc2 => LAYERUNLOAD, // 0x81056d85
                0xc3 => LAYERCTRL,   // 0x81056e4d
                0xc4 => LAYERWAIT,   // 0x81056fa5

                0xc7 => LAYERSELECT, // 0x810570af
                0xc8 => MOVIEWAIT,   // 0x81057177

                0xd0 => CHARCLEAR,  // 0x8105723f
                0xd1 => CHARLOAD,   // 0x81057275
                0xd2 => CHARUNLOAD, // 0x81057381
                0xd3 => CHARDISP,   // 0x81057403
                0xd4 => CHARCTRL,   // 0x810574cb
                0xd5 => CHARWAIT,   // 0x81057623
                0xd6 => CHARMARK,   // 0x8105772d
                0xd7 => CHARSYNC,   // 0x81057839

                0xff => DEBUGOUT, // 0x8105786f
                _ => return None,
            })
        }),
        ShinVersion::HigurashiHou => Some(if opcode != 0 && opcode < 0x80 {
            // ===
            // Instructions
            Opcode::Instruction(match opcode {
                0x40 => uo,       // 0x71000b7f48
                0x41 => bo,       // 0x71000b81c4
                0x42 => exp,      // 0x71000b83cc
                0x43 => mm,       // 0x71000b894c
                0x44 => gt,       // 0x71000b8a14
                0x45 => st,       // 0x71000b8aec
                0x46 => jc,       // 0x71000b8bc0
                0x47 => j,        // 0x71000b8ce4
                0x48 => gosub,    // 0x71000b8d1c
                0x49 => retsub,   // 0x71000b8d70
                0x4a => jt,       // 0x71000b8d94
                0x4b => gosubt,   // 0x71000b8e1c
                0x4c => rnd,      // 0x71000b8eb8
                0x4d => push,     // 0x71000b8f88
                0x4e => pop,      // 0x71000b8ffc
                0x4f => call,     // 0x71000b90d0
                0x50 => r#return, // 0x71000b91a0
                // TODO: those three are unverified
                0x51 => fmt,      // 0x71000b91dc
                0x52 => fnmt,     // 0x71000b9300
                0x53 => getbupid, // 0x71000b9424
                _ => return None,
            })
        } else {
            // ===
            // Commands
            Opcode::Command(match opcode {
                0x00 => EXIT,        // 0x71000b7ed0
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
            })
        }),
        ShinVersion::HigurashiHouV2 => Some(if opcode != 0 && opcode < 0x80 {
            // ===
            // Instructions
            Opcode::Instruction(match opcode {
                0x40 => uo,       // 0x71000bd308
                0x41 => bo,       // 0x71000bd610
                0x42 => exp,      // 0x71000bd8f8
                0x43 => mm,       // 0x71000bdf48
                0x44 => gt,       // 0x71000be014
                0x45 => st,       // 0x71000be0ec
                0x46 => jc,       // 0x71000be1c0
                0x47 => j,        // 0x71000be314
                0x48 => gosub,    // 0x71000be34c
                0x49 => retsub,   // 0x71000be3a0
                0x4a => jt,       // 0x71000be3c4
                0x4b => gosubt,   // 0x71000be44c
                0x4c => rnd,      // 0x71000be4e8
                0x4d => push,     // 0x71000be5b8
                0x4e => pop,      // 0x71000be628
                0x4f => call,     // 0x71000be6fc
                0x50 => r#return, // 0x71000be7d0
                // TODO: those three are unverified
                0x51 => fmt,      // 0x71000be80c
                0x52 => fnmt,     // 0x71000be92c
                0x53 => getbupid, // 0x71000bea4c
                _ => return None,
            })
        } else {
            // ===
            // Commands
            Opcode::Command(match opcode {
                0x00 => EXIT,        // 0x71000bd290
                0x81 => SGET,        // 0x71000beb2c
                0x82 => SSET,        // 0x71000beb94
                0x83 => WAIT,        // 0x71000bebe0
                0x85 => MSGINIT,     // 0x71000bec4c
                0x86 => MSGSET,      // 0x71000becac
                0x87 => MSGWAIT,     // 0x71000bed74
                0x88 => MSGSIGNAL,   // 0x71000bedb4
                0x89 => MSGSYNC,     // 0x71000bede8
                0x8a => MSGCLOSE,    // 0x71000bee34
                0x8b => MSGHIDE,     // 0x71000bee98
                0x8c => LOGSET,      // 0x71000beecc
                0x8d => SELECT,      // 0x71000bef4c
                0x8e => WIPE,        // 0x71000bf0b4
                0x8f => WIPEWAIT,    // 0x71000bf214
                0x90 => BGMPLAY,     // 0x71000bf248
                0x91 => BGMSTOP,     // 0x71000bf2b4
                0x92 => BGMVOL,      // 0x71000bf2f4
                0x93 => BGMWAIT,     // 0x71000bf340
                0x94 => BGMSYNC,     // 0x71000bf380
                0x95 => SEPLAY,      // 0x71000bf3c0
                0x96 => SESTOP,      // 0x71000bf450
                0x97 => SESTOPALL,   // 0x71000bf49c
                0x98 => SEVOL,       // 0x71000bf4dc
                0x99 => SEPAN,       // 0x71000bf53c
                0x9a => SEWAIT,      // 0x71000bf59c
                0x9b => SEONCE,      // 0x71000bf5e8
                0x9c => VOICEPLAY,   // 0x71000bf660
                0x9d => VOICESTOP,   // 0x71000bf700
                0x9e => VOICEWAIT,   // 0x71000bf734
                0xa0 => SAVEINFO,    // 0x71000bf774
                0xa1 => AUTOSAVE,    // 0x71000bf804
                0xa2 => EVBEGIN,     // 0x71000bf838
                0xa3 => EVEND,       // 0x71000bf878
                0xb0 => TROPHY,      // 0x71000bf8ac
                0xc0 => LAYERINIT,   // 0x71000bf8ec
                0xc1 => LAYERLOAD,   // 0x71000bf92c
                0xc2 => LAYERUNLOAD, // 0x71000bfa8c
                0xc3 => LAYERCTRL,   // 0x71000bfad8
                0xc4 => LAYERWAIT,   // 0x71000bfc2c
                0xc5 => LAYERBACK,   // 0x71000bfccc
                0xc6 => LAYERSWAP,   // 0x71000bfd0c
                0xc7 => LAYERSELECT, // 0x71000bfd58
                0xc8 => MOVIEWAIT,   // 0x71000bfda4
                0xc9 => FEELICON,    // 0x71000bfdf0
                0xd0 => TIPSGET,     // 0x71000bfe68
                0xd1 => CHARSELECT,  // 0x71000bff00
                0xd2 => OTSUGET,     // 0x71000bff94
                0xd3 => CHART,       // 0x71000bffd4
                0xd4 => SNRSEL,      // 0x71000c008c
                0xd5 => KAKERA,      // 0x71000c00cc
                0xd6 => KAKERAGET,   // 0x71000c0100
                0xd7 => QUIZ,        // 0x71000c01a0
                0xd8 => FAKESELECT,  // 0x71000c022c
                0xd9 => UNLOCK,      // 0x71000c0260
                0xda => KGET,        // 0x71000c0318
                0xdb => KSET,        // 0x71000c0380
                0xff => DEBUGOUT,    // 0x71000c03cc
                _ => return None,
            })
        }),
        ShinVersion::DC4 => Some(if opcode != 0 && opcode < 0x80 {
            // ===
            // Instructions
            Opcode::Instruction(match opcode {
                0x40 => uo,       // 0x71000baa50
                0x41 => bo,       // 0x71000bad20
                0x42 => exp,      // 0x71000bb010
                0x43 => mm,       // 0x71000bb640
                0x44 => gt,       // 0x71000bb710
                0x45 => st,       // 0x71000bb7f0
                0x46 => jc,       // 0x71000bb8d0
                0x47 => j,        // 0x71000bba00
                0x48 => gosub,    // 0x71000bba40
                0x49 => retsub,   // 0x71000bbaa0
                0x4a => jt,       // 0x71000bbad0
                0x4b => gosubt,   // 0x71000bbb60
                0x4c => rnd,      // 0x71000bbc00
                0x4d => push,     // 0x71000bbcf0
                0x4e => pop,      // 0x71000bbd70
                0x4f => call,     // 0x71000bbe50
                0x50 => r#return, // 0x71000bbf30
                0x51 => fmt,      // 0x71000bbf70
                // TODO: not verified
                0x52 => fnmt,     // 0x71000bc090
                0x53 => getbupid, // 0x71000bc1b0
                _ => return None,
            })
        } else {
            // ===
            // Commands
            Opcode::Command(match opcode {
                0x00 => EXIT, // 0x71000ba9d0

                0x81 => SGET, // 0x71000bc2a0
                0x82 => SSET, // 0x71000bc310
                0x83 => WAIT, // 0x71000bc360

                0x85 => MSGINIT,   // 0x71000bc3d0
                0x86 => MSGSET,    // 0x71000bc410
                0x87 => MSGWAIT,   // 0x71000bc4e0
                0x88 => MSGSIGNAL, // 0x71000bc520
                0x89 => MSGSYNC,   // 0x71000bc560
                0x8a => MSGCLOSE,  // 0x71000bc5b0
                0x8b => MSGFACE,   // 0x71000bc620

                0x8d => SELECT,    // 0x71000bc660
                0x8e => WIPE,      // 0x71000bc7d0
                0x8f => WIPEWAIT,  // 0x71000bc930
                0x90 => BGMPLAY,   // 0x71000bc970
                0x91 => BGMSTOP,   // 0x71000bc9e0
                0x92 => BGMVOL,    // 0x71000bca20
                0x93 => BGMWAIT,   // 0x71000bca70
                0x94 => BGMSYNC,   // 0x71000bcab0
                0x95 => SEPLAY,    // 0x71000bcaf0
                0x96 => SESTOP,    // 0x71000bcb80
                0x97 => SESTOPALL, // 0x71000bcbd0
                0x98 => SEVOL,     // 0x71000bcc10
                0x99 => SEPAN,     // 0x71000bcc70
                0x9a => SEWAIT,    // 0x71000bccd0
                0x9b => SEONCE,    // 0x71000bcd20
                0x9c => VOICEPLAY, // 0x71000bcda0
                0x9d => VOICESTOP, // 0x71000bce40
                0x9e => VOICEWAIT, // 0x71000bce80

                0xa0 => SAVEINFO,  // 0x71000bcec0
                0xa1 => AUTOSAVE,  // 0x71000bcf50
                0xa2 => EVBEGIN,   // 0x71000bcf90
                0xa3 => EVEND,     // 0x71000bcfd0
                0xa4 => RESUMESET, // 0x71000bd010
                0xa5 => RESUME,    // 0x71000bd050

                0xb0 => TROPHY, // 0x71000bd090
                0xb1 => UNLOCK, // 0x71000bd0d0

                0xc0 => LAYERINIT,   // 0x71000bd190
                0xc1 => LAYERLOAD,   // 0x71000bd1d0
                0xc2 => LAYERUNLOAD, // 0x71000bd330
                0xc3 => LAYERCTRL,   // 0x71000bd380
                0xc4 => LAYERWAIT,   // 0x71000bd4e0
                0xc5 => LAYERSWAP,   // 0x71000bd590
                0xc6 => LAYERSELECT, // 0x71000bd5e0
                0xc7 => MOVIEWAIT,   // 0x71000bd630

                0xc9 => TRANSSET,    // 0x71000bd680
                0xca => TRANSWAIT,   // 0x71000bd7e0
                0xcb => PAGEBACK,    // 0x71000bd820
                0xcc => PLANESELECT, // 0x71000bd860
                0xcd => PLANECLEAR,  // 0x71000bd8a0
                0xce => MASKLOAD,    // 0x71000bd8e0
                0xcf => MASKUNLOAD,  // 0x71000bd940
                0xd0 => CHATSET,     // 0x71000bd980

                0xff => DEBUGOUT, // 0x71000bda00
                _ => return None,
            })
        }),
        ShinVersion::Konosuba => Some(if opcode != 0 && opcode < 0x80 {
            // ===
            // Instructions
            Opcode::Instruction(match opcode {
                0x40 => uo,       // 0x710009cd90
                0x41 => bo,       // 0x710009d060
                0x42 => exp,      // 0x710009d340
                0x43 => mm,       // 0x710009d960
                0x44 => gt,       // 0x710009da10
                0x45 => st,       // 0x710009dae0
                0x46 => jc,       // 0x710009dba0
                0x47 => j,        // 0x710009dd10
                0x48 => gosub,    // 0x710009dd40
                0x49 => retsub,   // 0x710009dd90
                0x4a => jt,       // 0x710009ddc0
                0x4b => gosubt,   // 0x710009de40
                0x4c => rnd,      // 0x710009ded0
                0x4d => push,     // 0x710009dfa0
                0x4e => pop,      // 0x710009e010
                0x4f => call,     // 0x710009e0d0
                0x50 => r#return, // 0x710009e1a0
                // NOTE: those two are unverified
                0x51 => fmt,      // 0x710009e1e0
                0x52 => fnmt,     // 0x710009e300
                0x53 => getbupid, // 0x710009e420
                _ => return None,
            })
        } else {
            // ===
            // Commands
            Opcode::Command(match opcode {
                0x00 => EXIT,        // 0x710009cd40
                0x80 => SGET,        // 0x710009e520
                0x81 => SSET,        // 0x710009e590
                0x82 => WAIT,        // 0x710009e5e0
                0x83 => MSGINIT,     // 0x710009e650
                0x84 => MSGSET,      // 0x710009e690
                0x85 => MSGWAIT,     // 0x710009e740
                0x86 => MSGSIGNAL,   // 0x710009e780
                0x87 => MSGCLOSE,    // 0x710009e7c0
                0x88 => SELECT,      // 0x710009e820
                0x89 => WIPE,        // 0x710009e980
                0x8a => WIPEWAIT,    // 0x710009eae0
                0x90 => BGMPLAY,     // 0x710009eb20
                0x91 => BGMSTOP,     // 0x710009eb90
                0x92 => BGMVOL,      // 0x710009ebd0
                0x93 => BGMWAIT,     // 0x710009ec20
                0x94 => BGMSYNC,     // 0x710009ec60
                0x95 => SEPLAY,      // 0x710009eca0
                0x96 => SESTOP,      // 0x710009ed30
                0x97 => SESTOPALL,   // 0x710009ed80
                0x98 => SEVOL,       // 0x710009edc0
                0x99 => SEPAN,       // 0x710009ee20
                0x9a => SEWAIT,      // 0x710009ee80
                0x9b => SEONCE,      // 0x710009eed0
                0x9c => VOICEPLAY,   // 0x710009ef50
                0x9d => VOICESTOP,   // 0x710009eff0
                0x9e => VOICEWAIT,   // 0x710009f030
                0xc0 => LAYERINIT,   // 0x710009f070
                0xc1 => LAYERLOAD,   // 0x710009f0b0
                0xc2 => LAYERUNLOAD, // 0x710009f210
                0xc3 => LAYERCTRL,   // 0x710009f260
                0xc4 => LAYERWAIT,   // 0x710009f3b0
                0xc5 => LAYERBACK,   // 0x710009f450
                0xc6 => LAYERSELECT, // 0x710009f490
                0xc7 => MOVIEWAIT,   // 0x710009f4e0
                0xe0 => SLEEP,       // 0x710009f530
                0xe1 => VSET,        // 0x710009f5c0
                0xff => DEBUGOUT,    // 0x710009f610
                _ => return None,
            })
        }),
        ShinVersion::Umineko => Some(if opcode != 0 && opcode < 0x80 {
            // ===
            // Instructions
            Opcode::Instruction(match opcode {
                0x40 => uo,       // 0x71000eba70
                0x41 => bo,       // 0x71000ebd50
                0x42 => exp,      // 0x71000ec050
                0x43 => mm,       // 0x71000ec680
                0x44 => gt,       // 0x71000ec750
                0x45 => st,       // 0x71000ec830
                0x46 => jc,       // 0x71000ec910
                0x47 => j,        // 0x71000eca90
                0x48 => gosub,    // 0x71000ecad0
                0x49 => retsub,   // 0x71000ecb30
                0x4a => jt,       // 0x71000ecb60
                0x4b => gosubt,   // 0x71000ecbf0
                0x4c => rnd,      // 0x71000ecc90
                0x4d => push,     // 0x71000ecd80
                0x4e => pop,      // 0x71000ece00
                0x4f => call,     // 0x71000ecee0
                0x50 => r#return, // 0x71000ecfc0
                0x51 => fmt,      // 0x71000ed000
                0x52 => fnmt,     // 0x71000ed120
                0x53 => getbupid, // 0x71000ed240
                _ => return None,
            })
        } else {
            // ===
            // Commands
            Opcode::Command(match opcode {
                0x00 => EXIT,        // 0x71000eb9f0
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
            })
        }),
        ShinVersion::Gerokasu2 => Some(if opcode != 0 && opcode < 0x80 {
            // ===
            // Instructions
            Opcode::Instruction(match opcode {
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
                0x51 => fmt,      // 0x71000bf740
                // TODO: these two are unverified
                0x52 => fnmt,     // 0x71000bf860
                0x53 => getbupid, // 0x71000bf978
                _ => return None,
            })
        } else {
            // ===
            // Commands
            Opcode::Command(match opcode {
                0x00 => EXIT,        // 0x71000be194
                0x81 => SGET,        // 0x71000bfa50
                0x82 => SSET,        // 0x71000bfab0
                0x83 => WAIT,        // 0x71000bfafc
                0x85 => MSGINIT,     // 0x71000bfb60
                0x86 => MSGSET,      // 0x71000bfba0
                0x87 => MSGWAIT,     // 0x71000bfc5c
                0x88 => MSGSIGNAL,   // 0x71000bfc9c
                0x89 => MSGSYNC,     // 0x71000bfcd0
                0x8a => MSGCLOSE,    // 0x71000bfd1c
                0x8b => MSGFACE,     // 0x71000bfd78
                0x8d => SELECT,      // 0x71000bfdc0
                0x8e => WIPE,        // 0x71000bff18
                0x8f => WIPEWAIT,    // 0x71000c0070
                0x90 => BGMPLAY,     // 0x71000c00a4
                0x91 => BGMSTOP,     // 0x71000c0110
                0x92 => BGMVOL,      // 0x71000c0150
                0x93 => BGMWAIT,     // 0x71000c019c
                0x94 => BGMSYNC,     // 0x71000c01dc
                0x95 => SEPLAY,      // 0x71000c021c
                0x96 => SESTOP,      // 0x71000c02ac
                0x97 => SESTOPALL,   // 0x71000c02f8
                0x98 => SEVOL,       // 0x71000c0338
                0x99 => SEPAN,       // 0x71000c0398
                0x9a => SEWAIT,      // 0x71000c03f8
                0x9b => SEONCE,      // 0x71000c0444
                0x9c => VOICEPLAY,   // 0x71000c04bc
                0x9d => VOICESTOP,   // 0x71000c0554
                0x9e => VOICEWAIT,   // 0x71000c0588
                0x9f => SYSSE,       // 0x71000c05c8
                0xa0 => SAVEINFO,    // 0x71000c0614
                0xa1 => AUTOSAVE,    // 0x71000c069c
                0xa2 => EVBEGIN,     // 0x71000c06d0
                0xa3 => EVEND,       // 0x71000c0710
                0xa4 => RESUMESET,   // 0x71000c0744
                0xa5 => RESUME,      // 0x71000c0778
                0xa6 => SYSCALL,     // 0x71000c07ac
                0xb0 => TROPHY,      // 0x71000c07f8
                0xb1 => UNLOCK,      // 0x71000c0840
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
            })
        }),
    }
}
