mod reader;
mod trace;

pub use trace::{ConsoleTraceListener, StringTraceReactor};

pub trait Reactor {
    fn byte(&mut self) -> u8;
    fn short(&mut self) -> u16;
    fn reg(&mut self);
    fn offset(&mut self);
    fn u8string(&mut self, fixup: bool, source: StringSource);
    fn u16string(&mut self, fixup: bool, source: StringSource);
    fn u8string_array(&mut self, fixup: bool, source: StringArraySource);
    fn msgid(&mut self) -> u32;

    fn instr_start(&mut self);

    fn has_instr(&self) -> bool;

    fn debug_loc(&self) -> String;
}

#[derive(Debug)]
pub enum StringSource {
    Saveinfo,
    Select,
    Msgset(u32),
    Dbgout,
    Logset,
    Voiceplay,
}

#[derive(Debug)]
pub enum StringArraySource {
    Select,
}
