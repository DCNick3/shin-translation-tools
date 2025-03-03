use bumpalo::Bump;
use shin_versions::MessageCommandStyle;

use crate::reactor::{rewrite::StringRewriter, StringSource};

/// Exercises the string encoder and decoder, but does not actually rewrite strings.
pub struct NoopRewriter {
    snr_style: MessageCommandStyle,
    user_style: MessageCommandStyle,
}

impl NoopRewriter {
    pub fn new(snr_style: MessageCommandStyle, user_style: MessageCommandStyle) -> Self {
        Self {
            snr_style,
            user_style,
        }
    }
}

impl StringRewriter for NoopRewriter {
    fn rewrite_string<'bump>(
        &'bump self,
        bump: &'bump Bump,
        raw_decoded: &'bump str,
        _instr_index: u32,
        _instr_offset: u32,
        source: StringSource,
    ) -> Option<&'bump str> {
        let transformed = crate::message_parser::transform(
            bump,
            raw_decoded,
            self.snr_style,
            self.user_style,
            source,
        );

        Some(transformed)
    }
}
