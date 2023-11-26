use clap::ValueEnum;

use crate::reactor::{Reactor, StringArraySource, StringSource};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, ValueEnum)]
pub enum Version {
    AstralAir,
}

enum NumberImpl {
    Short,
    #[allow(unused)]
    VarInt,
}

impl Version {
    fn number_impl(&self) -> NumberImpl {
        match self {
            Version::AstralAir => NumberImpl::Short,
        }
    }
}

pub struct Ctx<R> {
    reactor: R,
    version: Version,
}

impl<R: Reactor> Ctx<R> {
    pub fn new(reactor: R, version: Version) -> Self {
        Self { reactor, version }
    }

    pub fn byte(&mut self) -> u8 {
        self.reactor.byte()
    }

    pub fn short(&mut self) -> u16 {
        self.reactor.short()
    }

    pub fn reg(&mut self) {
        self.reactor.reg()
    }

    pub fn number(&mut self) {
        match self.version.number_impl() {
            NumberImpl::Short => {
                self.short();
            }
            NumberImpl::VarInt => {
                todo!()
            }
        }
    }

    pub fn offset(&mut self) {
        self.reactor.offset()
    }

    pub fn string(&mut self, source: StringSource) {
        // TODO: switch on version for string size
        match source {
            StringSource::Saveinfo
            | StringSource::Select
            | StringSource::Dbgout
            | StringSource::Voiceplay => self.reactor.u8string(false, source),
            StringSource::Msgset(_) | StringSource::Logset => self.reactor.u16string(true, source),
            // only emitted by the tracer
            StringSource::SelectChoice(_) => unreachable!(),
        }
    }

    pub fn string_array(&mut self, source: StringArraySource) {
        // TODO: switch on string source/version combinations
        self.reactor.u8string_array(true, source)
    }

    pub fn bitmask_number_array(&mut self) {
        let t = self.reactor.byte();
        for _ in 0..t.count_ones() {
            self.number();
        }
    }

    pub fn msgid(&mut self) -> u32 {
        self.reactor.msgid()
    }

    pub fn instr_start(&mut self) {
        self.reactor.instr_start()
    }
    pub fn has_instr(&self) -> bool {
        self.reactor.has_instr()
    }

    pub fn debug_loc(&self) -> String {
        self.reactor.debug_loc()
    }
}
