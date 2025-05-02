use std::io;

use bumpalo::Bump;
use shin_versions::StringEncoding;

pub fn decode_utf8_zstring(mut s: &[u8]) -> io::Result<&str> {
    while s.last() == Some(&0) {
        s = &s[..s.len() - 1];
    }

    std::str::from_utf8(s).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("{}", e)))
}

pub fn encode_utf8_zstring<'bump>(bump: &'bump Bump, s: &str) -> &'bump [u8] {
    let mut output = bumpalo::collections::Vec::with_capacity_in(s.len() + 1, bump);
    output.extend_from_slice(s.as_bytes());
    output.push(0);

    output.into_bump_slice()
}

pub fn decode_zstring<'s>(
    bump: &'s Bump,
    encoding: StringEncoding,
    s: &'s [u8],
    fixup: bool,
) -> io::Result<&'s str> {
    match encoding {
        StringEncoding::ShiftJis => shin_text::decode_sjis_zstring(bump, s, fixup),
        StringEncoding::Utf8 => decode_utf8_zstring(s),
    }
}
