use bumpalo::{collections, Bump};

use crate::reactor::{rewrite::Rewriter, StringSource};

/// Rewrite all strings to "X".
pub struct XRewriter;

impl Rewriter for XRewriter {
    fn rewrite_string<'bump>(
        &self,
        bump: &'bump Bump,
        _instr_index: u32,
        _instr_offset: u32,
        _source: StringSource,
    ) -> Option<collections::String<'bump>> {
        Some(collections::String::from_str_in("X", bump))
    }
}
