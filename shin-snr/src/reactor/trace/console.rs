use bumpalo::collections;
use tracing::info;

use crate::reactor::{trace::StringTraceListener, StringSource};

pub struct ConsoleTraceListener;

impl StringTraceListener for ConsoleTraceListener {
    fn on_string(&mut self, instr_offset: u32, source: StringSource, s: collections::String) {
        info!("{:08x} {:?}: {}", instr_offset, source, s)
    }
}
