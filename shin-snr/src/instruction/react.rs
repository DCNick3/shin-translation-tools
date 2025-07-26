use shin_versions::ShinVersion;

use crate::{
    ctx::Ctx,
    instruction::Instruction,
    reactor::{Reactor, StringArraySource, StringSource},
};

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
                // not all games support this many expressions, but this is not critical
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
            // NOTE: with varint-based numbers, these are padded to 4 bytes to allow for seeking
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
            let offset_count = ctx.gt_gosubt_length();
            for _ in 0..offset_count {
                ctx.offset();
            }
        }
        Instruction::gosubt => {
            ctx.number();
            let offset_count = ctx.gt_gosubt_length();
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
            ShinVersion::HigurashiSui | ShinVersion::AliasCarnival => {
                ctx.number();
            }
            ShinVersion::WhiteEternity
            | ShinVersion::HigurashiHou
            | ShinVersion::DC4
            | ShinVersion::Umineko
            | ShinVersion::Gerokasu2 => {
                ctx.byte();
                ctx.number();
            }
            ShinVersion::Konosuba => {
                ctx.number();
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
            ShinVersion::HigurashiSui | ShinVersion::AliasCarnival => ctx.number(),
            ShinVersion::WhiteEternity
            | ShinVersion::HigurashiHou
            | ShinVersion::DC4
            | ShinVersion::Konosuba
            | ShinVersion::Umineko
            | ShinVersion::Gerokasu2 => {
                ctx.byte();
                ctx.number();
            }
        },
        Instruction::KEYWAIT => match ctx.version() {
            ShinVersion::HigurashiSui | ShinVersion::AliasCarnival => ctx.number(),
            ShinVersion::WhiteEternity
            | ShinVersion::HigurashiHou
            | ShinVersion::DC4
            | ShinVersion::Umineko
            | ShinVersion::Gerokasu2 => {
                unreachable!()
            }
            ShinVersion::Konosuba => {
                todo!()
            }
        },
        Instruction::MSGINIT => match ctx.version() {
            ShinVersion::HigurashiSui | ShinVersion::AliasCarnival | ShinVersion::WhiteEternity => {
                ctx.number();
                ctx.number();
            }
            ShinVersion::HigurashiHou
            | ShinVersion::DC4
            | ShinVersion::Konosuba
            | ShinVersion::Umineko
            | ShinVersion::Gerokasu2 => {
                ctx.number();
            }
        },
        Instruction::MSGSET => {
            let msgid = ctx.uint() & 0xffffff;

            match ctx.version() {
                ShinVersion::HigurashiSui => {
                    // nothing here
                }
                ShinVersion::AliasCarnival => {
                    ctx.number();
                }
                ShinVersion::WhiteEternity => {
                    // NOTE: these numbers are NOT present in umineko
                    ctx.number();
                    ctx.number();
                }
                ShinVersion::HigurashiHou => {
                    ctx.number();
                }
                ShinVersion::DC4 => {
                    ctx.number();
                }
                ShinVersion::Konosuba => {
                    // nothing here
                }
                ShinVersion::Umineko => {
                    // nothing here
                }
                ShinVersion::Gerokasu2 => {
                    ctx.number();
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
            ShinVersion::HigurashiSui | ShinVersion::AliasCarnival => {}
            ShinVersion::WhiteEternity
            | ShinVersion::HigurashiHou
            | ShinVersion::DC4
            | ShinVersion::Konosuba
            | ShinVersion::Umineko
            | ShinVersion::Gerokasu2 => {
                ctx.byte();
            }
        },
        Instruction::MSGCHECK => {
            ctx.uint();
        }
        Instruction::MSGQUAKE => {
            ctx.number();
        }
        Instruction::MSGFACE => match ctx.version() {
            ShinVersion::HigurashiSui => unreachable!(),
            ShinVersion::AliasCarnival => {
                ctx.number();
                ctx.number();
                ctx.number();
                ctx.number();
            }
            ShinVersion::WhiteEternity
            | ShinVersion::HigurashiHou
            | ShinVersion::DC4
            | ShinVersion::Gerokasu2 => {
                ctx.number();
            }
            ShinVersion::Konosuba => {
                todo!()
            }
            ShinVersion::Umineko => unreachable!(),
        },
        Instruction::LOGSET => match ctx.version() {
            ShinVersion::HigurashiSui => ctx.string(StringSource::Logset),
            ShinVersion::AliasCarnival => {
                ctx.number();
                ctx.string(StringSource::Logset)
            }
            ShinVersion::WhiteEternity | ShinVersion::HigurashiHou => {
                ctx.string(StringSource::Logset)
            }
            ShinVersion::DC4 | ShinVersion::Umineko | ShinVersion::Gerokasu2 => {
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
        Instruction::WIPE => match ctx.version() {
            ShinVersion::HigurashiSui => {
                // this is.... weird....
                let b1 = ctx.byte();
                let b2 = ctx.byte();
                if b1 == 0 {
                    if b2 & 0x1 != 0 {
                        ctx.number();
                    }
                    if b2 & 0x2 != 0 {
                        ctx.number();
                    }
                    if b2 & 0x4 != 0 {
                        ctx.number();
                    }
                    if b2 & 0x8 != 0 {
                        ctx.number();
                    }
                } else {
                    if b2 & 0x1 != 0 {
                        ctx.number();
                    }
                }
            }
            ShinVersion::AliasCarnival
            | ShinVersion::HigurashiHou
            | ShinVersion::WhiteEternity
            | ShinVersion::DC4
            | ShinVersion::Konosuba
            | ShinVersion::Umineko
            | ShinVersion::Gerokasu2 => {
                ctx.number();
                ctx.number();
                ctx.number();
                ctx.bitmask_number_array();
            }
        },
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
            ShinVersion::HigurashiSui | ShinVersion::AliasCarnival => {
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
            ShinVersion::HigurashiHou
            | ShinVersion::DC4
            | ShinVersion::Konosuba
            | ShinVersion::Umineko
            | ShinVersion::Gerokasu2 => {
                // 7x number
                ctx.number();
                ctx.number();
                ctx.number();
                ctx.number();
                ctx.number();
                ctx.number();
                ctx.number();
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
            ShinVersion::HigurashiSui | ShinVersion::AliasCarnival => {
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
            ShinVersion::DC4
            | ShinVersion::HigurashiHou
            | ShinVersion::Konosuba
            | ShinVersion::Umineko
            | ShinVersion::Gerokasu2 => {
                ctx.number();
                ctx.number();
                ctx.number();
                ctx.number();
                ctx.number();
            }
        },
        Instruction::UNK9B => {
            // only used by AliasCarnival and HigurashiSui
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
        Instruction::SYSSE => {
            ctx.number();
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
        Instruction::SYSCALL => {
            ctx.number();
            ctx.number();
        }
        Instruction::TROPHY => {
            ctx.number();
        }
        Instruction::UNLOCK => match ctx.version() {
            ShinVersion::HigurashiSui
            | ShinVersion::AliasCarnival
            | ShinVersion::WhiteEternity
            | ShinVersion::Konosuba => {
                unreachable!()
            }
            ShinVersion::DC4 | ShinVersion::Gerokasu2 => {
                ctx.number();
            }
            ShinVersion::HigurashiHou | ShinVersion::Umineko => {
                ctx.byte();
                let count = ctx.byte();
                for _ in 0..count {
                    ctx.number();
                }
            }
        },
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
        Instruction::LAYERWAIT => match ctx.version() {
            ShinVersion::HigurashiSui => {
                ctx.number();
                ctx.number();
            }
            ShinVersion::AliasCarnival
            | ShinVersion::WhiteEternity
            | ShinVersion::HigurashiHou
            | ShinVersion::DC4
            | ShinVersion::Konosuba
            | ShinVersion::Umineko
            | ShinVersion::Gerokasu2 => {
                ctx.number();
                let count = ctx.byte();
                for _ in 0..count {
                    ctx.number();
                }
            }
        },
        Instruction::LAYERSWAP => {
            ctx.number();
            ctx.number();
        }
        Instruction::LAYERBACK => {
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

        // Konosuba
        Instruction::SLEEP => {
            ctx.short();
            ctx.number();
            ctx.number();
            ctx.number();
            ctx.number();
        }
        Instruction::VSET => {
            ctx.number();
            ctx.number();
        }

        // Higurashi Sui
        Instruction::TIPSGET => {
            let len = ctx.byte();
            for _ in 0..len {
                ctx.number();
            }
        }
        Instruction::CHARSEL => {
            // not _too_ sure about these, but on higu sui engine these are all shorts :shrug:
            ctx.short();
            ctx.reg();
            ctx.short();
            ctx.number();
        }
        Instruction::OTSUGET => {
            ctx.number();
        }
        Instruction::CHART => {
            ctx.byte();
            let len = ctx.byte();
            for _ in 0..len {
                ctx.number();
            }
        }
        Instruction::SNRSEL => {
            ctx.number();
        }

        Instruction::KAKERA => {}
        Instruction::KAKERAGET => {
            ctx.number();
            let len = ctx.byte();
            for _ in 0..len {
                ctx.number();
            }
        }
        Instruction::QUIZ => {
            ctx.reg(); // <= !!! notice me pls
            match ctx.version() {
                ShinVersion::HigurashiSui | ShinVersion::HigurashiHou => {
                    ctx.number();
                    ctx.number();
                    ctx.number();
                }
                ShinVersion::Umineko | ShinVersion::Gerokasu2 => {
                    ctx.number();
                }
                ShinVersion::AliasCarnival
                | ShinVersion::WhiteEternity
                | ShinVersion::DC4
                | ShinVersion::Konosuba => {
                    unreachable!()
                }
            }
        }
        Instruction::FAKESELECT => {}

        // Umineko
        Instruction::CHARS => {
            ctx.number();
            ctx.number();
        }
        Instruction::SHOWCHARS => {}
        Instruction::NOTIFYSET => ctx.number(),

        // Higurashi Hou
        Instruction::CHARSELECT => {
            ctx.short();
            ctx.reg();
            ctx.number();
        }
        Instruction::FEELICON => {
            ctx.number();
            ctx.number();
            ctx.number();
            ctx.number();
            ctx.number();
        }

        Instruction::DEBUGOUT => match ctx.version() {
            ShinVersion::HigurashiSui => unreachable!(),
            ShinVersion::AliasCarnival => {
                ctx.string(StringSource::Dbgout);
                ctx.short();
            }
            ShinVersion::WhiteEternity
            | ShinVersion::HigurashiHou
            | ShinVersion::DC4
            | ShinVersion::Konosuba
            | ShinVersion::Umineko
            | ShinVersion::Gerokasu2 => {
                ctx.string(StringSource::Dbgout);
                let count = ctx.byte();
                for _ in 0..count {
                    ctx.number();
                }
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
