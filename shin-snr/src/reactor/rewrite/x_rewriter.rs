use bumpalo::Bump;

use crate::reactor::{rewrite::StringRewriter, StringSource};

/// Rewrite all strings to "X".
pub struct XRewriter;

impl StringRewriter for XRewriter {
    fn rewrite_string<'a>(
        &'a self,
        _bump: &'a Bump,
        _instr_index: u32,
        _instr_offset: u32,
        _source: StringSource,
    ) -> Option<&'a str> {
        Some("X")
    }
}
