use shin_versions::LengthKind;

use crate::operation::schema::{
    def::{InstructionSchemaCtx, OperationResult},
    opcode::Instruction,
};

pub(super) fn get_schema_for_instruction(
    ctx: &mut InstructionSchemaCtx,
    instruction: Instruction,
) -> OperationResult {
    use shin_versions::ShinVersion::*;
    macro_rules! guard {
        ($($pattern:tt)*) => {
            if !matches!(ctx.version(), shin_versions::verpat!($($pattern)*)) {
                return OperationResult::Unreachable;
            }
        };
    }

    match instruction {
        Instruction::uo => {
            ctx.operation();
            ctx.reg();
            ctx.optional_number();
        }
        Instruction::bo => {
            ctx.operation();
            ctx.reg();
            ctx.optional_number();
            ctx.number();
        }
        Instruction::exp => {
            ctx.reg();
            ctx.expression();
        }
        Instruction::mm => {
            ctx.number();
            ctx.reg_array(ctx.version().mm_gt_st_length());
        }
        Instruction::gt => {
            ctx.reg();
            ctx.number();
            ctx.pad_number_array(ctx.version().mm_gt_st_length());
        }
        Instruction::st => {
            ctx.number();
            ctx.number();
            ctx.reg_array(ctx.version().mm_gt_st_length());
        }
        Instruction::jc => {
            ctx.condition();
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
            ctx.offset_array(ctx.version().gt_gosubt_length());
        }
        Instruction::gosubt => {
            ctx.number();
            ctx.offset_array(ctx.version().gt_gosubt_length());
        }
        Instruction::rnd => {
            ctx.reg();
            ctx.number();
            ctx.number();
        }
        Instruction::push => {
            ctx.number_array(LengthKind::U8Length);
        }
        Instruction::pop => {
            ctx.reg_array(LengthKind::U8Length);
        }
        Instruction::call => {
            guard!(
                AliasCarnival
                    | WhiteEternity
                    | HigurashiHou
                    | HigurashiHouV2
                    | DC4
                    | Konosuba
                    | Umineko
                    | Gerokasu2
            );
            ctx.offset();
            ctx.number_array(LengthKind::U8Length);
        }
        Instruction::r#return => {
            guard!(
                AliasCarnival
                    | WhiteEternity
                    | HigurashiHou
                    | HigurashiHouV2
                    | DC4
                    | Konosuba
                    | Umineko
                    | Gerokasu2
            );
        }
        Instruction::fmt | Instruction::fnmt => {
            guard!(@post-shin);
            ctx.reg(); // destination
            ctx.number(); // needle
            ctx.number(); // starting index
            ctx.pad_number_array(LengthKind::U16Length);
        }
        Instruction::getbupid => {
            guard!(@switch);
            ctx.reg();
            ctx.number();
            ctx.number();
        }
    }

    OperationResult::Ok
}
