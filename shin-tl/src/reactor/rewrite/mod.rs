mod csv_rewriter;
mod x_rewriter;

use std::{collections::HashMap, io};

use bumpalo::{
    collections::{String, Vec},
    Bump,
};
use shin_text::encode_sjis_zstring;

pub use self::{csv_rewriter::CsvRewriter, x_rewriter::XRewriter};
use crate::{
    reactor::{Reactor, StringArraySource, StringSource},
    reader::Reader,
};

pub struct OffsetMapBuilder {
    orig_to_idx: HashMap<u32, u32>,
    idx_to_out: HashMap<u32, u32>,
}

impl OffsetMapBuilder {
    pub fn new() -> Self {
        Self {
            orig_to_idx: HashMap::new(),
            idx_to_out: HashMap::new(),
        }
    }

    pub fn build(self) -> OffsetMap {
        let mut map = OffsetMap::new();
        for (orig, idx) in self.orig_to_idx {
            map.map.insert(orig, self.idx_to_out[&idx]);
        }
        map
    }
}

pub struct OffsetMap {
    map: HashMap<u32, u32>,
}

impl OffsetMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn get(&self, in_offset: u32) -> Option<u32> {
        self.map.get(&in_offset).copied()
    }
}

pub trait Rewriter {
    fn rewrite_string<'bump>(
        &self,
        bump: &'bump Bump,
        instr_index: u32,
        instr_offset: u32,
        source: StringSource,
    ) -> Option<String<'bump>>;
}

impl Rewriter for () {
    fn rewrite_string<'bump>(
        &self,
        _bump: &'bump Bump,
        _instr_index: u32,
        _instr_offset: u32,
        _source: StringSource,
    ) -> Option<String<'bump>> {
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

    fn reg(&mut self, r: u16) {
        self.short(r);
    }

    fn offset(&mut self, o: u32);

    fn msgid(&mut self, msgid: u32) -> u32 {
        let [m0, m1, m2, _] = msgid.to_le_bytes();
        self.write(&[m0, m1, m2]);

        msgid
    }
    fn instr_start(&mut self, in_offset: u32);
}

pub struct BuildOffsetMapMode {
    builder: OffsetMapBuilder,
    initial_in_position: u32,
    out_position: u32,
    instr_index: u32,
}

impl RewriteMode for BuildOffsetMapMode {
    fn write(&mut self, data: &[u8]) {
        self.out_position += data.len() as u32;
    }

    fn offset(&mut self, o: u32) {
        self.write(&o.to_le_bytes());
    }

    fn instr_start(&mut self, in_offset: u32) {
        let idx = self.instr_index;
        self.builder.orig_to_idx.insert(in_offset, idx);
        self.builder.idx_to_out.insert(idx, self.out_position);

        self.instr_index += 1;
    }
}

pub struct EmitMode<W> {
    map: OffsetMap,
    writer: W,
}

impl<W: io::Write> RewriteMode for EmitMode<W> {
    fn write(&mut self, data: &[u8]) {
        self.writer.write_all(data).unwrap()
    }

    fn offset(&mut self, o: u32) {
        let mapped = self.map.get(o).expect("offset not found in map. this is either a shin-tl bug, or the SNR has an offset that points in a middle of an instruction.");
        self.write(&mapped.to_le_bytes());
    }

    fn instr_start(&mut self, _in_offset: u32) {}
}

pub struct RewriteReactor<'a, L, M> {
    reader: Reader<'a>,
    current_instr_index: u32,
    current_str_index: u32,
    current_instr_offset: u32,
    rewriter: L,
    mode: M,
    bump: Bump,
}

impl<'a, R> RewriteReactor<'a, R, BuildOffsetMapMode> {
    pub fn new(reader: Reader<'a>, rewriter: R, initial_out_position: u32) -> Self {
        let initial_in_position = reader.position();
        Self {
            reader,
            current_instr_index: 0,
            current_str_index: 0,
            current_instr_offset: 0,
            rewriter,
            mode: BuildOffsetMapMode {
                builder: OffsetMapBuilder::new(),
                initial_in_position,
                out_position: initial_out_position,
                instr_index: 0,
            },
            bump: Bump::new(),
        }
    }

    pub fn into_emit<W>(self, writer: W) -> RewriteReactor<'a, R, EmitMode<W>> {
        RewriteReactor {
            reader: self.reader.rewind(self.mode.initial_in_position),
            current_instr_index: 0,
            current_str_index: 0,
            current_instr_offset: 0,
            rewriter: self.rewriter,
            mode: EmitMode {
                map: self.mode.builder.build(),
                writer,
            },
            bump: self.bump,
        }
    }
}

impl<'a, R: Rewriter, M: RewriteMode> Reactor for RewriteReactor<'a, R, M> {
    fn byte(&mut self) -> u8 {
        self.mode.byte(self.reader.byte())
    }

    fn short(&mut self) -> u16 {
        self.mode.short(self.reader.short())
    }

    fn reg(&mut self) {
        self.mode.reg(self.reader.reg())
    }

    fn offset(&mut self) {
        self.mode.offset(self.reader.offset())
    }

    fn u8string(&mut self, fixup: bool, source: StringSource) {
        let s = self.reader.u8string();

        // TODO: possible optimization: we don't have to actually encode the string during the map building phase
        // we only care about the size of the string and we have measure_sjis_string for that

        // a poor's man Cow for bumpalo
        // implementing this a separate data type can reduce the amount of duplication
        let mut c = None;
        let s = if let Some(s) = self.rewriter.rewrite_string(
            &self.bump,
            self.current_str_index,
            self.current_instr_offset,
            source,
        ) {
            c = Some(encode_sjis_zstring(&self.bump, &s, fixup).unwrap());
            c.as_deref().unwrap()
        } else {
            s
        };

        self.current_str_index += 1;

        self.mode.byte(s.len() as u8);
        self.mode.write(s);
    }

    fn u16string(&mut self, fixup: bool, source: StringSource) {
        let s = self.reader.u16string();

        // a poor's man Cow for bumpalo
        let mut c = None;
        let s = if let Some(s) = self.rewriter.rewrite_string(
            &self.bump,
            self.current_str_index,
            self.current_instr_offset,
            source,
        ) {
            c = Some(encode_sjis_zstring(&self.bump, &s, fixup).unwrap());
            c.as_deref().unwrap()
        } else {
            s
        };

        self.current_str_index += 1;

        self.mode.short(s.len() as u16);
        self.mode.write(s);
    }

    fn u8string_array(&mut self, fixup: bool, source: StringArraySource) {
        let mut s = self.reader.u8string_array();
        while s.last() == Some(&0) {
            s = &s[..s.len() - 1];
        }

        let source_maker = match source {
            StringArraySource::Select => StringSource::SelectChoice,
        };

        let mut res = Vec::new_in(&self.bump);
        for (i, s) in s.split(|&v| v == 0).enumerate() {
            if let Some(s) = self.rewriter.rewrite_string(
                &self.bump,
                self.current_str_index,
                self.current_instr_offset,
                source_maker(i as u32),
            ) {
                // encode_sjis_zstring already adds a NUL terminator
                res.extend_from_slice(
                    encode_sjis_zstring(&self.bump, &s, fixup)
                        .unwrap()
                        .as_slice(),
                );
            } else {
                res.extend_from_slice(s);
                res.push(0);
            }

            self.current_str_index += 1;
        }
        res.push(0);

        self.mode.byte(res.len() as u8);
        self.mode.write(res.as_slice());
    }

    fn u16string_array(&mut self, fixup: bool, source: StringArraySource) {
        let mut s = self.reader.u16string_array();
        while s.last() == Some(&0) {
            s = &s[..s.len() - 1];
        }

        let source_maker = match source {
            StringArraySource::Select => StringSource::SelectChoice,
        };

        let mut res = Vec::new_in(&self.bump);
        for (i, s) in s.split(|&v| v == 0).enumerate() {
            if let Some(s) = self.rewriter.rewrite_string(
                &self.bump,
                self.current_str_index,
                self.current_instr_offset,
                source_maker(i as u32),
            ) {
                // encode_sjis_zstring already adds a NUL terminator
                res.extend_from_slice(
                    encode_sjis_zstring(&self.bump, &s, fixup)
                        .unwrap()
                        .as_slice(),
                );
            } else {
                res.extend_from_slice(s);
                res.push(0);
            }

            self.current_str_index += 1;
        }
        res.push(0);

        self.mode.short(res.len() as u16);
        self.mode.write(res.as_slice());
    }

    fn msgid(&mut self) -> u32 {
        self.mode.msgid(self.reader.msgid())
    }

    fn instr_start(&mut self) {
        self.current_instr_offset = self.reader.position();
        self.mode.instr_start(self.reader.position());
    }

    fn instr_end(&mut self) {
        self.current_instr_index += 1;
    }

    fn has_instr(&self) -> bool {
        self.reader.has_instr()
    }

    fn debug_loc(&self) -> std::string::String {
        format!("{:08x}", self.reader.position())
    }
}
