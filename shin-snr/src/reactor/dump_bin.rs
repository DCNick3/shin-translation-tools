use shin_text::StringArrayIter;

use crate::{
    operation::{
        OperationElementRepr,
        arena::OperationArena,
        schema::{Opcode, OperationSchema},
    },
    reactor::{Reactor, StringArraySource, StringSource},
};

pub struct DumpBinReactor<W> {
    writer: W,
}

impl<W> DumpBinReactor<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }
}

impl<W> Reactor for DumpBinReactor<W>
where
    W: std::io::Write,
{
    fn react(
        &mut self,
        _operation_position: u32,
        _raw_opcode: u8,
        opcode: Opcode,
        op_schema: &OperationSchema,
        arena: &OperationArena,
    ) {
        for element in arena.iter(&op_schema) {
            match element {
                OperationElementRepr::String(_, string) => {
                    let Some(source) = StringSource::for_operation(opcode, op_schema, arena) else {
                        panic!("Could not determine StringSource for opcode {:?}", opcode)
                    };

                    if source.is_for_messagebox() {
                        self.writer
                            .write_all(string)
                            .expect("Failed to write bin dump");
                    }
                }
                OperationElementRepr::StringArray(_, string_array) => {
                    let Some(source) = StringArraySource::for_operation(opcode, op_schema, arena)
                    else {
                        panic!(
                            "Could not determine StringArraySource for opcode {:?}",
                            opcode
                        )
                    };

                    for (_, string) in (0..).zip(StringArrayIter::new(string_array)) {
                        if source.is_for_messagebox() {
                            self.writer
                                .write_all(string)
                                .expect("Failed to write bin dump");
                        }
                    }
                }
                _ => {
                    // ignore
                }
            }
        }
    }
}
