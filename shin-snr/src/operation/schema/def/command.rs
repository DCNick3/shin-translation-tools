use shin_versions::{LengthKind, ShinVersion};

use crate::operation::schema::{
    OperationElement,
    def::{CommandSchemaCtx, OperationResult},
    opcode::Command,
};

pub(super) fn get_schema_for_command(
    ctx: &mut CommandSchemaCtx,
    command: Command,
) -> OperationResult {
    use Command::*;
    use ShinVersion::*;

    macro_rules! guard {
        ($($pattern:tt)*) => {
            if !matches!(ctx.version(), shin_versions::verpat!($($pattern)*)) {
                return OperationResult::Unreachable;
            }
        };
    }

    match command {
        EXIT => match ctx.version() {
            HigurashiSui | AliasCarnival | WorldRe => {
                ctx.number();
            }
            WhiteEternity | HigurashiHou | HigurashiHouV2 | DC4 | Umineko | Gerokasu2 => {
                ctx.u8();
                ctx.number();
            }
            Konosuba => {
                ctx.number();
            }
        },
        SGET => {
            ctx.reg();
            ctx.number();
        }
        SSET => {
            ctx.number();
            ctx.number();
        }
        WAIT => match ctx.version() {
            HigurashiSui | AliasCarnival | WorldRe => ctx.number(),
            WhiteEternity | HigurashiHou | HigurashiHouV2 | DC4 | Konosuba | Umineko
            | Gerokasu2 => {
                ctx.u8();
                ctx.number();
            }
        },
        KEYWAIT => match ctx.version() {
            HigurashiSui | AliasCarnival | WorldRe => ctx.number(),
            WhiteEternity | HigurashiHou | HigurashiHouV2 | DC4 | Konosuba | Umineko
            | Gerokasu2 => {
                return OperationResult::Unreachable;
            }
        },
        MSGINIT => match ctx.version() {
            HigurashiSui | AliasCarnival | WorldRe | WhiteEternity => {
                ctx.number();
                ctx.number();
            }
            HigurashiHou | DC4 | Konosuba | Umineko | Gerokasu2 => {
                ctx.number();
            }
            HigurashiHouV2 => {
                ctx.number();
                ctx.number();
                ctx.number();
            }
        },
        MSGSET => {
            ctx.u32();

            match ctx.version() {
                HigurashiSui => {
                    // nothing here
                }
                AliasCarnival => {
                    ctx.number();
                }
                WorldRe => {
                    ctx.number();
                    ctx.number();
                    ctx.number();
                }
                WhiteEternity => {
                    // NOTE: these numbers are NOT present in umineko
                    ctx.number();
                    ctx.number();
                }
                HigurashiHou | HigurashiHouV2 => {
                    ctx.number();
                }
                DC4 => {
                    ctx.number();
                }
                Konosuba => {
                    // nothing here
                }
                Umineko => {
                    // nothing here
                }
                Gerokasu2 => {
                    ctx.number();
                }
            }

            ctx.string();
        }
        MSGWAIT => {
            ctx.number();
        }
        MSGSIGNAL => {}
        MSGSYNC => {
            guard!(HigurashiHou | HigurashiHouV2 | DC4 | Umineko | Gerokasu2);
            ctx.number();
            ctx.number();
        }
        MSGCLOSE => match ctx.version() {
            HigurashiSui | AliasCarnival | WorldRe => {}
            WhiteEternity | HigurashiHou | HigurashiHouV2 | DC4 | Konosuba | Umineko
            | Gerokasu2 => {
                ctx.u8();
            }
        },
        MSGCHECK => {
            guard!(@pre-shin);
            ctx.u32();
        }
        MSGQUAKE => {
            guard!(HigurashiSui);
            ctx.number();
        }
        MSGHIDE => {
            // it's game-specific, but is not in the game-specific command range :/
            guard!(HigurashiHouV2);
        }
        MSGFACE => match ctx.version() {
            HigurashiSui | WorldRe | Konosuba | HigurashiHouV2 | Umineko => {
                return OperationResult::Unreachable;
            }
            AliasCarnival => {
                ctx.number();
                ctx.number();
                ctx.number();
                ctx.number();
            }
            WhiteEternity | HigurashiHou | DC4 | Gerokasu2 => {
                ctx.number();
            }
        },
        LOGSET => match ctx.version() {
            HigurashiSui | WhiteEternity | HigurashiHou | HigurashiHouV2 => ctx.string(),
            WorldRe => {
                ctx.number();
                ctx.number();
                ctx.number();
                ctx.string();
            }
            AliasCarnival => {
                ctx.number();
                ctx.string();
            }
            DC4 | Konosuba | Umineko | Gerokasu2 => return OperationResult::Unreachable,
        },
        SELECT => {
            ctx.u16();
            ctx.u16();
            ctx.reg();
            ctx.number();
            ctx.string();
            ctx.string_array();
        }
        WIPE => match ctx.version() {
            HigurashiSui | WorldRe => {
                // this one is.... weird....
                ctx.custom(OperationElement::HiguSuiWipeArg);
            }
            AliasCarnival | HigurashiHou | HigurashiHouV2 | WhiteEternity | DC4 | Konosuba
            | Umineko | Gerokasu2 => {
                ctx.number();
                ctx.number();
                ctx.number();
                ctx.bitmask_number_array();
            }
        },
        WIPEWAIT => {}
        BGMPLAY => {
            ctx.number();
            ctx.number();
            ctx.number();
            ctx.number();
        }
        BGMSTOP => {
            ctx.number();
        }
        BGMVOL => {
            ctx.number();
            ctx.number();
        }
        BGMWAIT => {
            ctx.number();
        }
        BGMSYNC => {
            ctx.number();
        }
        SEPLAY => match ctx.version() {
            HigurashiSui | AliasCarnival | WorldRe => {
                // 5x number
                ctx.number();
                ctx.number();
                ctx.number();
                ctx.number();
                ctx.number();
            }
            WhiteEternity => {
                // 6x number
                ctx.number();
                ctx.number();
                ctx.number();
                ctx.number();
                ctx.number();
                ctx.number();
            }
            HigurashiHou | HigurashiHouV2 | DC4 | Konosuba | Umineko | Gerokasu2 => {
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
        SESTOP => {
            ctx.number();
            ctx.number();
        }
        SESTOPALL => {
            ctx.number();
        }
        SEVOL => {
            ctx.number();
            ctx.number();
            ctx.number();
        }
        SEPAN => {
            guard!(@post-shin);
            ctx.number();
            ctx.number();
            ctx.number();
        }
        SEWAIT => {
            ctx.number();
            ctx.number();
        }
        SEONCE => match ctx.version() {
            HigurashiSui | AliasCarnival | WorldRe => {
                // 3x number
                ctx.number();
                ctx.number();
                ctx.number();
            }
            WhiteEternity => {
                // 4x number
                ctx.number();
                ctx.number();
                ctx.number();
                ctx.number();
            }
            DC4 | HigurashiHou | HigurashiHouV2 | Konosuba | Umineko | Gerokasu2 => {
                // 5x number
                ctx.number();
                ctx.number();
                ctx.number();
                ctx.number();
                ctx.number();
            }
        },
        UNK9B => {
            guard!(@pre-shin);
            ctx.number();
            ctx.number();
        }
        VOICEPLAY => {
            ctx.string();
            ctx.number();
            ctx.number();
        }
        VOICESTOP => {
            guard!(HigurashiHou | HigurashiHouV2 | DC4 | Konosuba | Umineko | Gerokasu2);
        }
        VOICEWAIT => {
            ctx.number();
        }
        SYSSE => {
            guard!(Umineko | Gerokasu2);
            ctx.number();
            ctx.number();
        }

        SAVEINFO => {
            guard!(@not-konosuba);
            ctx.number();
            ctx.string();
        }
        MOVIE => {
            guard!(@pre-shin);
            ctx.number();
        }
        AUTOSAVE => {
            guard!(@not-konosuba);
        }
        EVBEGIN => {
            guard!(@not-konosuba);
            ctx.number();
        }
        EVEND => {
            guard!(@not-konosuba);
        }
        RESUMESET => {
            guard!(@post-plane);
        }
        RESUME => {
            guard!(@post-plane);
        }
        SYSCALL => {
            guard!(Umineko | Gerokasu2);
            ctx.number();
            ctx.number();
        }
        TROPHY => {
            guard!(@not-konosuba);
            ctx.number();
        }
        UNLOCK => match ctx.version() {
            HigurashiSui | AliasCarnival | WorldRe | WhiteEternity | Konosuba => {
                return OperationResult::Unreachable;
            }
            DC4 | Gerokasu2 => {
                ctx.number();
            }
            HigurashiHou | HigurashiHouV2 | Umineko => {
                ctx.u8();
                ctx.number_array(LengthKind::U8Length);
            }
        },
        LAYERCLEAR => {
            guard!(@pre-shin);
        }
        LAYERINIT => {
            guard!(@post-shin);
            ctx.number()
        }
        LAYERLOAD => {
            ctx.number();
            ctx.number();
            ctx.number();
            ctx.bitmask_number_array();
        }
        LAYERUNLOAD => {
            guard!(@post-shin);
            ctx.number();
            ctx.number();
        }
        LAYERCTRL => {
            ctx.number();
            ctx.number();
            ctx.bitmask_number_array();
        }
        LAYERWAIT => match ctx.version() {
            HigurashiSui | WorldRe => {
                ctx.number();
                ctx.number();
            }
            AliasCarnival | WhiteEternity | HigurashiHou | HigurashiHouV2 | DC4 | Konosuba
            | Umineko | Gerokasu2 => {
                ctx.number();
                ctx.number_array(LengthKind::U8Length);
            }
        },
        LAYERSWAP => {
            guard!(HigurashiHou | HigurashiHouV2 | DC4 | Umineko | Gerokasu2);
            ctx.number();
            ctx.number();
        }
        LAYERBACK => {
            guard!(HigurashiHou | HigurashiHouV2 | Konosuba);
            ctx.number();
        }
        LAYERSELECT => {
            guard!(@post-shin);
            ctx.number();
            ctx.number();
        }
        MOVIEWAIT => {
            guard!(@post-shin);
            ctx.number();
            ctx.number();
        }

        TRANSSET => {
            guard!(@post-plane);
            ctx.number();
            ctx.number();
            ctx.number();
            ctx.bitmask_number_array();
        }
        TRANSWAIT => {
            guard!(@post-plane);
            ctx.number();
        }
        PAGEBACK => {
            guard!(@post-plane);
        }
        PLANESELECT => {
            guard!(@post-plane);
            ctx.number();
        }
        PLANECLEAR => {
            guard!(@post-plane);
        }
        MASKLOAD => {
            guard!(@post-plane);
            ctx.number();
            ctx.number();
            ctx.number();
        }
        MASKUNLOAD => {
            guard!(@post-plane);
        }

        // Alias Carnival
        ICOGET => {
            guard!(AliasCarnival);
            ctx.number_array(LengthKind::U8Length);
        }
        STAGEINFO => {
            guard!(AliasCarnival);
            ctx.string();
            ctx.string();
        }
        ICOCHK => {
            guard!(AliasCarnival);
            // not that sure what this short actually represents
            ctx.u16();
        }
        EMOTEWAIT => {
            guard!(AliasCarnival);
            ctx.number();
        }
        NAMED => {
            guard!(AliasCarnival);
            ctx.u8();
            ctx.string();
        }
        BACKINIT => {
            guard!(AliasCarnival);
        }

        // CHAR* commands, present in White Eternity and WorldRe
        CHARCLEAR => {
            guard!(WorldRe | WhiteEternity);
        }
        CHARLOAD => {
            guard!(WhiteEternity);
            ctx.number();
            ctx.number();
            ctx.number();
        }
        CHARUNLOAD => {
            guard!(WhiteEternity);
            ctx.number();
        }
        CHARDISP => {
            guard!(WorldRe | WhiteEternity);
            ctx.number();
            ctx.number();
        }
        CHARCTRL => {
            guard!(WorldRe | WhiteEternity);
            ctx.number();
            ctx.number();
            ctx.bitmask_number_array();
        }
        CHARWAIT => match ctx.version() {
            WorldRe => {
                ctx.number();
                ctx.number();
            }
            WhiteEternity => {
                ctx.number();
                ctx.number_array(LengthKind::U8Length);
            }
            _ => return OperationResult::Unreachable,
        },
        CHARMARK => {
            guard!(WorldRe | WhiteEternity);
            ctx.number();
            ctx.number();
            ctx.number();
        }
        CHARSYNC => {
            guard!(WhiteEternity);
        }
        CHARSET => {
            guard!(WorldRe);
            ctx.number();
            ctx.number();
            ctx.number();
            ctx.bitmask_number_array();
        }
        CHARDEL => {
            guard!(WorldRe);
            ctx.number();
            ctx.number();
        }
        CHARRGB => {
            guard!(WorldRe);
            ctx.number();
            ctx.number();
            ctx.number();
        }
        CHARFUKI => {
            guard!(WorldRe);
            ctx.number();
        }

        // DC4
        CHATSET => {
            guard!(DC4);
            ctx.string()
        }

        // Konosuba
        SLEEP => {
            guard!(Konosuba);
            ctx.u16();
            ctx.number();
            ctx.number();
            ctx.number();
            ctx.number();
        }
        VSET => {
            guard!(Konosuba);
            ctx.number();
            ctx.number();
        }

        // Higurashi Sui
        TIPSGET => {
            guard!(HigurashiSui | WorldRe | HigurashiHou | HigurashiHouV2 | Umineko);
            ctx.number_array(LengthKind::U8Length);
        }
        CHARSEL => {
            guard!(WorldRe | HigurashiSui);
            // not _too_ sure about these, but on higu sui engine these are all shorts :shrug:
            ctx.u16();
            ctx.reg();
            ctx.u16();
            ctx.number();
        }
        OTSUGET => {
            guard!(HigurashiSui | WorldRe | HigurashiHou | HigurashiHouV2);
            ctx.number();
        }
        CHART => {
            guard!(@higurashi);
            ctx.u8();
            ctx.number_array(LengthKind::U8Length);
        }
        SNRSEL => {
            guard!(@higurashi);
            ctx.number();
        }

        KAKERA => {
            guard!(@higurashi);
        }
        KAKERAGET => {
            guard!(@higurashi);
            ctx.number();
            ctx.number_array(LengthKind::U8Length);
        }
        QUIZ => {
            ctx.reg(); // <= !!! notice me pls
            match ctx.version() {
                HigurashiSui | HigurashiHou | HigurashiHouV2 => {
                    ctx.number();
                    ctx.number();
                    ctx.number();
                }
                Umineko | Gerokasu2 => {
                    ctx.number();
                }
                AliasCarnival | WhiteEternity | WorldRe | DC4 | Konosuba => {
                    return OperationResult::Unreachable;
                }
            }
        }
        FAKESELECT => {
            guard!(@higurashi);
        }

        // Umineko
        CHARS => {
            guard!(Umineko);
            ctx.number();
            ctx.number();
        }
        SHOWCHARS => {
            guard!(Umineko);
        }
        NOTIFYSET => {
            guard!(Umineko);
            ctx.number();
        }

        // Higurashi Hou
        CHARSELECT => {
            guard!(HigurashiHou | HigurashiHouV2);
            ctx.u16();
            ctx.reg();
            ctx.number();
        }
        FEELICON => {
            guard!(HigurashiHou | HigurashiHouV2);
            ctx.number();
            ctx.number();
            ctx.number();
            ctx.number();
            ctx.number();
        }

        // Higurashi Hou V2
        KGET => {
            guard!(HigurashiHouV2);
            ctx.reg();
            ctx.number();
        }
        KSET => {
            guard!(HigurashiHouV2);
            ctx.number();
            ctx.number();
        }

        DEBUGOUT => match ctx.version() {
            HigurashiSui | WorldRe => return OperationResult::Unreachable,
            AliasCarnival => {
                ctx.string();
                ctx.u16();
            }
            WhiteEternity | HigurashiHou | HigurashiHouV2 | DC4 | Konosuba | Umineko
            | Gerokasu2 => {
                ctx.string();
                ctx.number_array(LengthKind::U8Length);
            }
        },
        #[allow(unreachable_patterns)]
        _ => return OperationResult::Unimplemented,
    }

    OperationResult::Ok
}
