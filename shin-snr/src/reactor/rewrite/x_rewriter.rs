use bumpalo::Bump;

use crate::reactor::{rewrite::StringRewriter, AnyStringSource};

/// Rewrite all strings to "X".
pub struct XRewriter;

impl StringRewriter for XRewriter {
    fn rewrite_string<'bump>(
        &'bump self,
        _bump: &'bump Bump,
        _decoded: &'bump str,
        _instr_index: u32,
        _instr_offset: u32,
        _source: AnyStringSource,
    ) -> Option<&'bump str> {
        Some("X")
    }
}
