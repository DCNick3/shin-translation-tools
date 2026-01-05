use std::collections::HashSet;

use crate::{
    operation::{
        OperationElementRepr,
        arena::OperationArena,
        schema::{Opcode, OperationSchema},
    },
    reactor::Reactor,
};

// why do we need it, again?
pub struct OffsetValidatorReactor {
    referred_offsets: HashSet<u32>,
    operation_offsets: HashSet<u32>,
}

impl OffsetValidatorReactor {
    pub fn new() -> Self {
        Self {
            referred_offsets: HashSet::new(),
            operation_offsets: HashSet::new(),
        }
    }
}

impl Reactor for OffsetValidatorReactor {
    fn react(
        &mut self,
        operation_position: u32,
        _raw_opcode: u8,
        _opcode: Opcode,
        op_schema: &OperationSchema,
        arena: &OperationArena,
    ) {
        self.operation_offsets.insert(operation_position);

        for element in arena.iter(op_schema) {
            match element {
                OperationElementRepr::Offset(offset) => {
                    self.referred_offsets.insert(offset);
                }
                OperationElementRepr::OffsetArray(_, offsets) => {
                    self.referred_offsets.extend(offsets);
                }
                _ => {}
            }
        }
    }
}

impl OffsetValidatorReactor {
    pub fn validate(self) -> Result<(), String> {
        let mut invalid_offsets = Vec::new();
        for offset in self.referred_offsets {
            if !self.operation_offsets.contains(&offset) {
                invalid_offsets.push(offset);
            }
        }

        if invalid_offsets.is_empty() {
            Ok(())
        } else {
            let invalid_offsets_count = invalid_offsets.len();
            let mut err = format!("{} invalid offsets\n", invalid_offsets_count);
            for offset in invalid_offsets.into_iter().take(32) {
                err.push_str(&format!("- 0x{:08x}\n", offset));
            }
            if invalid_offsets_count > 32 {
                err.push_str("...\n");
            }

            err.remove(err.len() - 1);

            Err(err)
        }
    }
}
