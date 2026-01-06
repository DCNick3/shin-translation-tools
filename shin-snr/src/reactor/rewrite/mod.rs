mod csv_rewriter;
mod noop_rewriter;
mod x_rewriter;

use std::collections::HashMap;

use bumpalo::Bump;
use shin_text::{FixupDetectResult, StringArrayIter, encode_sjis_zstring};
use shin_versions::{MessageCommandStyle, NumberStyle, StringPolicy};

pub use self::{
    csv_rewriter::{CsvData, CsvRewriter, StringReplacementMode},
    noop_rewriter::NoopRewriter,
    x_rewriter::XRewriter,
};
use crate::{
    layout::message_parser::MessageReflowMode,
    operation::{
        OperationElementRepr,
        arena::OperationArena,
        schema::{Opcode, OperationSchema},
        serialize::InstructionSerializeContext,
    },
    reactor::{AnyStringSource, Reactor, StringArraySource, StringSource},
    text::{decode_zstring, encode_utf8_zstring},
    writer::{CountingWriter, RealWriter, Writer},
};

#[derive(Default)]
pub struct OffsetMapBuilder {
    orig_to_idx: HashMap<u32, u32>,
    idx_to_out: HashMap<u32, u32>,
}

impl OffsetMapBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn build(self) -> OffsetMap {
        let mut map = OffsetMap::new();
        for (orig, idx) in self.orig_to_idx {
            map.map.insert(orig, self.idx_to_out[&idx]);
        }
        map
    }
}

#[derive(Default)]
pub struct OffsetMap {
    map: HashMap<u32, u32>,
}

impl OffsetMap {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get(&self, in_offset: u32) -> Option<u32> {
        self.map.get(&in_offset).copied()
    }
}

pub trait StringRewriter {
    fn rewrite_string<'bump>(
        &'bump self,
        bump: &'bump Bump,
        raw_decoded: &'bump str,
        instr_index: u32,
        instr_offset: u32,
        source: AnyStringSource,
    ) -> Option<&'bump str>;
}

impl StringRewriter for () {
    fn rewrite_string<'bump>(
        &self,
        _bump: &'bump Bump,
        _decoded: &'bump str,
        _instr_index: u32,
        _instr_offset: u32,
        _source: AnyStringSource,
    ) -> Option<&'bump str> {
        None
    }
}

pub trait RewriteMode {
    fn write(&mut self, data: &[u8]);

    fn byte(&mut self, b: u8) -> u8 {
        self.write(&[b]);

        b
    }

    fn short(&mut self, s: u16) -> u16 {
        self.write(&s.to_le_bytes());

        s
    }

    fn uint(&mut self, s: u32) -> u32 {
        self.write(&s.to_le_bytes());

        s
    }

    fn reg(&mut self, r: u16) {
        self.short(r);
    }

    fn offset(&mut self, o: u32);
    fn instr_start(&mut self, in_offset: u32);
}

pub trait RewriteMode2 {
    fn instr_start(&mut self, in_offset: u32, raw_opcode: u8);
    fn put_element(&mut self, element: &OperationElementRepr);
    fn end_of_stream(&mut self);
}

pub struct BuildOffsetMapMode {
    builder: OffsetMapBuilder,
    serializing: InstructionSerializeContext<CountingWriter>,
    instr_index: u32,
}

impl RewriteMode2 for BuildOffsetMapMode {
    fn instr_start(&mut self, in_offset: u32, raw_opcode: u8) {
        let idx = self.instr_index;
        self.builder.orig_to_idx.insert(in_offset, idx);
        self.builder
            .idx_to_out
            .insert(idx, self.serializing.writer_mut().position());

        self.serializing.put_u8(raw_opcode);

        self.instr_index += 1;
    }

    fn put_element(&mut self, element: &OperationElementRepr) {
        self.serializing.put_element(element);
    }

    fn end_of_stream(&mut self) {
        self.serializing.writer_mut().pad_16();
    }
}

pub struct EmitMode {
    map: OffsetMap,
    serializing: InstructionSerializeContext<RealWriter>,
}

impl RewriteMode2 for EmitMode {
    fn instr_start(&mut self, _in_offset: u32, raw_opcode: u8) {
        self.serializing.put_u8(raw_opcode);
    }

    fn put_element(&mut self, element: &OperationElementRepr) {
        match element {
            &OperationElementRepr::Offset(value) => {
                let mapped = self.map.get(value).expect("offset not found in map. this is either a shin-tl bug, or the SNR has an offset that points in a middle of an instruction.");
                self.serializing.put_offset(mapped);
            }
            &OperationElementRepr::OffsetArray(kind, values) => {
                self.serializing.put_length(kind, values.len());
                for &value in values {
                    let mapped = self.map.get(value).expect("offset not found in map. this is either a shin-tl bug, or the SNR has an offset that points in a middle of an instruction.");
                    self.serializing.put_offset(mapped);
                }
            }

            another => self.serializing.put_element(another),
        }
    }

    fn end_of_stream(&mut self) {
        self.serializing.writer_mut().pad_16();
    }
}

struct Stringer<'a, R> {
    bump: Bump,
    snr_style: MessageCommandStyle,
    user_style: MessageCommandStyle,
    has_useless_escapes: bool,
    reflow_mode: MessageReflowMode<'a>,
    policy: StringPolicy,
    rewriter: R,
}

impl<'a, R> Stringer<'a, R> {
    pub fn new(
        snr_style: MessageCommandStyle,
        user_style: MessageCommandStyle,
        has_useless_escapes: bool,
        reflow_mode: MessageReflowMode<'a>,
        policy: StringPolicy,
        rewriter: R,
    ) -> Self {
        Self {
            bump: Bump::new(),
            snr_style,
            user_style,
            has_useless_escapes,
            reflow_mode,
            policy,
            rewriter,
        }
    }

    pub fn reset(mut self) -> Self {
        self.bump.reset();
        self
    }
}

impl<'a, R: StringRewriter> Stringer<'a, R> {
    /// Rewrites a string using the rewriter.
    ///
    /// The string may not actually be rewritten if the rewriter returns None.
    ///
    /// Expects the original string to be encoded in Shift-JIS and to be zero-terminated
    fn rewrite_string<'s>(
        &'s self,
        position: &mut Position,
        original: &'s [u8],
        source: AnyStringSource,
    ) -> &'s [u8] {
        assert_eq!(
            original.last().copied(),
            Some(0),
            "string is not zero-terminated"
        );
        let original_decoded = decode_zstring(
            &self.bump,
            self.policy.encoding(),
            original,
            source.contains_commands(),
        )
        .unwrap();

        let result = if let Some(replacement) = self.rewriter.rewrite_string(
            &self.bump,
            original_decoded,
            position.current_str_index,
            position.current_instr_offset,
            source,
        ) {
            match self.policy {
                StringPolicy::ShiftJis(policy) => {
                    let mut fixup_detect_result = FixupDetectResult::NoFixupCharacters;
                    shin_text::detect_fixup(original, &mut fixup_detect_result).unwrap();

                    let (transformed, fixup_policy) =
                        crate::layout::message_parser::transform_reflow_and_infer_fixup_policy(
                            &self.bump,
                            replacement,
                            self.user_style,
                            self.reflow_mode,
                            self.snr_style,
                            self.has_useless_escapes,
                            policy,
                            fixup_detect_result,
                            source,
                        );
                    encode_sjis_zstring(&self.bump, transformed, fixup_policy).unwrap()
                }
                StringPolicy::Utf8 => {
                    let transformed = crate::layout::message_parser::transform_reflow(
                        &self.bump,
                        replacement,
                        self.user_style,
                        self.reflow_mode,
                        self.snr_style,
                        true,
                        source,
                    );
                    encode_utf8_zstring(&self.bump, transformed)
                }
            }
        } else {
            original
        };

        position.current_str_index += 1;

        result
    }
}

#[derive(Default)]
struct Position {
    current_instr_index: u32,
    current_str_index: u32,
    current_instr_offset: u32,
}

impl<'a, R> RewriteReactor<'a, R, BuildOffsetMapMode> {
    pub fn new(
        number_style: NumberStyle,
        snr_style: MessageCommandStyle,
        user_style: MessageCommandStyle,
        has_useless_escapes: bool,
        reflow_mode: MessageReflowMode<'a>,
        policy: StringPolicy,
        rewriter: R,
        initial_out_position: u32,
    ) -> Self {
        Self {
            position: Default::default(),
            stringer: Stringer::new(
                snr_style,
                user_style,
                has_useless_escapes,
                reflow_mode,
                policy,
                rewriter,
            ),
            mode: BuildOffsetMapMode {
                builder: OffsetMapBuilder::new(),
                serializing: InstructionSerializeContext::new(
                    number_style,
                    CountingWriter::new(initial_out_position),
                ),
                instr_index: 0,
            },
        }
    }

    // TODO: we can get size of non-reacted rewriter. Need one more typestate?
    pub fn output_size(&self) -> u32 {
        self.mode.serializing.writer_ref().position()
    }

    pub fn into_emit(self, buffer: Vec<u8>) -> RewriteReactor<'a, R, EmitMode> {
        let (number_style, _) = self.mode.serializing.into_parts();

        RewriteReactor {
            position: Default::default(),
            stringer: self.stringer.reset(),
            mode: EmitMode {
                map: self.mode.builder.build(),
                serializing: InstructionSerializeContext::new(
                    number_style,
                    RealWriter::new(buffer),
                ),
            },
        }
    }
}

impl<'a, R> RewriteReactor<'a, R, EmitMode> {
    pub fn finish(self) -> Vec<u8> {
        let (_, writer) = self.mode.serializing.into_parts();

        writer.into_buffer()
    }
}

pub struct RewriteReactor<'a, R, M> {
    position: Position,
    stringer: Stringer<'a, R>,
    mode: M,
}

impl<'a, R: StringRewriter, M: RewriteMode2> Reactor for RewriteReactor<'a, R, M> {
    fn react(
        &mut self,
        operation_position: u32,
        raw_opcode: u8,
        opcode: Opcode,
        op_schema: &OperationSchema,
        arena: &OperationArena,
    ) {
        self.position.current_instr_offset = operation_position;

        self.mode.instr_start(operation_position, raw_opcode);

        for element in arena.iter(&op_schema) {
            match element {
                OperationElementRepr::String(kind, string) => {
                    let Some(source) = StringSource::for_operation(opcode, op_schema, arena) else {
                        panic!("Could not determine StringSource for opcode {:?}", opcode)
                    };

                    let rewritten = self.stringer.rewrite_string(
                        &mut self.position,
                        string,
                        AnyStringSource::Singular(source),
                    );

                    self.mode
                        .put_element(&OperationElementRepr::String(kind, rewritten))
                }
                OperationElementRepr::StringArray(kind, string_array) => {
                    let Some(source) = StringArraySource::for_operation(opcode, op_schema, arena)
                    else {
                        panic!(
                            "Could not determine StringArraySource for opcode {:?}",
                            opcode
                        )
                    };

                    let mut rewritten = bumpalo::collections::Vec::new_in(&self.stringer.bump);
                    for (i, s) in (0..).zip(StringArrayIter::new(string_array)) {
                        let s = self.stringer.rewrite_string(
                            &mut self.position,
                            s,
                            AnyStringSource::Array(source, i),
                        );
                        rewritten.extend_from_slice(s);
                    }
                    rewritten.push(0);

                    self.mode
                        .put_element(&OperationElementRepr::StringArray(kind, &rewritten))
                }
                element => self.mode.put_element(&element),
            }
        }

        self.position.current_instr_index += 1;
    }

    fn end_of_stream(&mut self) {
        self.mode.end_of_stream()
    }
}
