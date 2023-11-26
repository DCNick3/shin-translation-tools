use std::io;

use bumpalo::{
    collections::{String, Vec},
    Bump,
};

use crate::{
    reactor::{Reactor, StringArraySource, StringSource},
    reader::Reader,
    text::decode_sjis_string,
};

pub trait RewriteListener {
    fn rewrite_string<'bump>(
        &mut self,
        bump: &'bump Bump,
        instr_offset: u32,
        source: StringSource,
    ) -> Option<String<'bump>>;
}

impl RewriteListener for () {
    fn rewrite_string<'bump>(
        &mut self,
        _bump: &'bump Bump,
        _instr_offset: u32,
        _source: StringSource,
    ) -> Option<String<'bump>> {
        None
    }
}

pub struct RewriteReactor<'a, L, W> {
    reader: Reader<'a>,
    current_instr_offset: u32,
    listener: L,
    writer: W,
    bump: Bump,
}

impl<'a, L, W> RewriteReactor<'a, L, W> {
    pub fn new(reader: Reader<'a>, listener: L, writer: W) -> Self {
        Self {
            reader,
            current_instr_offset: 0,
            listener,
            writer,
            bump: Bump::new(),
        }
    }
}

impl<'a, L: RewriteListener, W: io::Write> Reactor for RewriteReactor<'a, L, W> {
    fn byte(&mut self) -> u8 {
        let r = self.reader.byte();
        self.writer.write_all(&[r]).unwrap();
        r
    }

    fn short(&mut self) -> u16 {
        let r = self.reader.short();
        self.writer.write_all(&r.to_le_bytes()).unwrap();
        r
    }

    fn reg(&mut self) {
        let reg = self.reader.reg();
        // TODO: abstract away the logic of register serialization?
        self.writer.write_all(&reg.to_le_bytes()).unwrap();
    }

    fn offset(&mut self) {
        let offset = self.reader.offset();
        // TODO: update the offset according to our delta
        // we will shift data around when we rewrite strings
        // so this is crucial
        self.writer.write_all(&offset.to_le_bytes()).unwrap();
    }

    fn u8string(&mut self, fixup: bool, source: StringSource) {
        let s = self.reader.u8string();

        if let Some(_s) =
            self.listener
                .rewrite_string(&self.bump, self.current_instr_offset, source)
        {
            todo!()
        } else {
            self.writer.write_all(&[(s.len() as u8)]).unwrap();
            self.writer.write_all(s).unwrap();
        }
    }

    fn u16string(&mut self, fixup: bool, source: StringSource) {
        let s = self.reader.u16string();

        if let Some(_s) =
            self.listener
                .rewrite_string(&self.bump, self.current_instr_offset, source)
        {
            todo!()
        } else {
            self.writer
                .write_all(&(s.len() as u16).to_le_bytes())
                .unwrap();
            self.writer.write_all(s).unwrap();
        }
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
            if let Some(_s) = self.listener.rewrite_string(
                &self.bump,
                self.current_instr_offset,
                source_maker(i as u32),
            ) {
                todo!()
            } else {
                res.extend_from_slice(s);
                res.push(0);
            }
        }
        res.push(0);

        self.writer.write_all(&[(res.len() as u8)]).unwrap();
        self.writer.write_all(&res).unwrap();
    }

    fn msgid(&mut self) -> u32 {
        let msgid = self.reader.msgid();
        let [m0, m1, m2, _] = msgid.to_le_bytes();
        self.writer.write_all(&[m0, m1, m2]).unwrap();
        msgid
    }

    fn instr_start(&mut self) {
        self.current_instr_offset = self.reader.position();
    }

    fn has_instr(&self) -> bool {
        self.reader.has_instr()
    }

    fn debug_loc(&self) -> std::string::String {
        format!("{:08x}", self.reader.position())
    }
}
