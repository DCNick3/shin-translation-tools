mod ctx;
mod instruction;
pub mod reactor;
pub mod reader;

use shin_versions::ShinVersion;

use self::{ctx::Ctx, reactor::Reactor};

fn react_impl<R: Reactor>(ctx: &mut Ctx<R>) {
    while ctx.has_instr() {
        ctx.instr_start();
        let opcode = ctx.byte();
        let Some(instr) = instruction::decode_instr(opcode) else {
            panic!(
                "Unknown opcode 0x{opcode:02x} ({opcode}) @ {}",
                ctx.debug_loc()
            );
        };
        instruction::react_instr(ctx, instr);
        ctx.instr_end();
    }
}

pub fn react_with<R: Reactor>(reactor: &mut R, version: ShinVersion) {
    let mut ctx = Ctx::new(reactor, version);
    react_impl(&mut ctx);
}
