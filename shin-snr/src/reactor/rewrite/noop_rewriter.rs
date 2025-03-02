use bumpalo::Bump;

use crate::reactor::{rewrite::StringRewriter, StringSource};

/// Exercises the string encoder and decoder, but does not actually rewrite strings.
pub struct NoopRewriter {}

impl NoopRewriter {
    pub fn new() -> Self {
        Self {}
    }
}

impl StringRewriter for NoopRewriter {
    fn rewrite_string<'bump>(
        &'bump self,
        _bump: &'bump Bump,
        decoded: &'bump str,
        _instr_index: u32,
        _instr_offset: u32,
        _source: StringSource,
    ) -> Option<&'bump str> {
        Some(decoded)
    }
}
