mod console;
mod csv;

use bumpalo::Bump;
use shin_text::StringArrayIter;
use shin_versions::{MessageCommandStyle, StringEncoding};

pub use self::{console::ConsoleTraceListener, csv::CsvTraceListener};
use crate::{
    layout::message_parser::MessageReflowMode,
    operation::{
        OperationElementRepr,
        arena::OperationArena,
        schema::{Opcode, OperationSchema},
    },
    reactor::{AnyStringSource, Reactor, StringArraySource, StringSource},
    text::decode_zstring,
};

pub trait StringTraceListener {
    fn on_string(&mut self, instr_offset: u32, source: AnyStringSource, s: &str);
}

pub struct StringTraceReactor<L> {
    string_encoding: StringEncoding,
    snr_style: MessageCommandStyle,
    user_style: MessageCommandStyle,
    has_useless_escapes: bool,
    listener: L,
    bump: Bump,
}

impl<L: StringTraceListener> StringTraceReactor<L> {
    pub fn new(
        string_encoding: StringEncoding,
        snr_style: MessageCommandStyle,
        user_style: MessageCommandStyle,
        has_useless_escapes: bool,
        listener: L,
    ) -> Self {
        Self {
            string_encoding,
            snr_style,
            user_style,
            has_useless_escapes,
            listener,
            bump: Bump::new(),
        }
    }

    fn on_string_impl(&mut self, operation_position: u32, source: AnyStringSource, s: &[u8]) {
        let snr_string = decode_zstring(
            &self.bump,
            self.string_encoding,
            s,
            source.contains_commands(),
        )
        .unwrap();

        let user_string = crate::layout::message_parser::transform_reflow(
            &self.bump,
            snr_string,
            self.snr_style,
            MessageReflowMode::NoReflow,
            self.user_style,
            self.has_useless_escapes,
            source,
        );

        self.listener
            .on_string(operation_position, source, user_string)
    }
}

impl<L: StringTraceListener> Reactor for StringTraceReactor<L> {
    fn react(
        &mut self,
        operation_position: u32,
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

                    self.on_string_impl(
                        operation_position,
                        AnyStringSource::Singular(source),
                        string,
                    );
                }
                OperationElementRepr::StringArray(_, string_array) => {
                    let Some(source) = StringArraySource::for_operation(opcode, op_schema, arena)
                    else {
                        panic!(
                            "Could not determine StringArraySource for opcode {:?}",
                            opcode
                        )
                    };

                    for (i, string) in (0..).zip(StringArrayIter::new(string_array)) {
                        self.on_string_impl(
                            operation_position,
                            AnyStringSource::Array(source, i),
                            string,
                        );
                    }
                }
                _ => {
                    // ignore
                }
            }
        }

        self.bump.reset();
    }
}
