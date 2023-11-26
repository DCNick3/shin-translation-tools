use bumpalo::collections;

use crate::reactor::{trace::StringTraceListener, StringArraySource, StringSource};

pub struct ConsoleTraceListener;

impl StringTraceListener for ConsoleTraceListener {
    fn on_string(&mut self, instr_offset: u32, source: StringSource, s: collections::String) {
        println!("{:08x} {:?}: {}", instr_offset, source, s)
    }

    fn on_string_array(
        &mut self,
        instr_offset: u32,
        source: StringArraySource,
        s: collections::Vec<collections::String>,
    ) {
        println!("{:08x} {:?}: {:?}", instr_offset, source, s)
    }
}
