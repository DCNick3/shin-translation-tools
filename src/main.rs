mod ctx;
mod instruction;
mod reactor;
mod reader;
mod text;

use ctx::{Ctx, Version};
use reactor::Reactor;

use crate::{
    reactor::{CsvTraceListener, StringTraceReactor},
    reader::Reader,
};

fn react<R: Reactor>(ctx: &mut Ctx<R>) {
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
        // TODO: exit condition
    }
}

fn main() {
    let snr_file = std::fs::read("main.snr").unwrap();

    assert_eq!(&snr_file[0..4], b"SNR ");
    let code_offset = u32::from_le_bytes(snr_file[0x20..0x24].try_into().unwrap());

    let writer = csv::Writer::from_path("string.csv").unwrap();

    let reader = Reader::new(&snr_file, code_offset as usize);
    let reactor = StringTraceReactor::new(reader, CsvTraceListener::new(writer));

    let mut ctx = Ctx::new(reactor, Version::AstralAir);
    react(&mut ctx);
}
