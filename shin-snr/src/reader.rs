use shin_versions::{LengthKind, NumberStyle};

pub enum VarIntSize {
    Zero,
    One,
    Two,
    Three,
}

impl From<VarIntSize> for usize {
    fn from(size: VarIntSize) -> Self {
        match size {
            VarIntSize::Zero => 0,
            VarIntSize::One => 1,
            VarIntSize::Two => 2,
            VarIntSize::Three => 3,
        }
    }
}

/// Determine the amount of additional bytes needed to parse a variable integer based on the first read byte
pub fn var_int_size(t: u8) -> VarIntSize {
    // t=TXXXXXXX
    // T=0 => XXXXXXX is a 7-bit signed constant
    // T=1 => futher processing needed
    if t & 0x80 != 0 {
        // t=1PPPKKKK
        let p = (t & 0x70) >> 4;
        // P=0 => 12-bit signed constant (KKKK denotes the upper 4 bits, lsb is read from the next byte)
        // P=1 => 20-bit signed constant (KKKK denotes the upper 4 bits, 2 lower bytes are read from the stream)
        // P=2 => 28-bit signed constant (KKKK denotes the upper 4 bits, 3 lower bytes are read from the stream)
        // P=3 => 4-bit regular register, KKKK is the index
        // P=4 => 12-bit regular register, KKKK denotes the upper 4 bits, lsb is read from the next byte
        // P=5 => 4-bit argument register, KKKK is the index
        // P=6 => constant 0x80000000 aka -2147483648 aka MIN_INT (NOTE: this is not implemented in `shin-core`, as it isn't used in umineko)
        match p {
            0 => VarIntSize::One,
            1 => VarIntSize::Two,
            2 => VarIntSize::Three,
            3 => VarIntSize::Zero,
            4 => VarIntSize::One,
            5 => VarIntSize::Zero,
            6 => VarIntSize::Zero,
            _ => {
                // TODO: fallible parsing?
                panic!("Unknown NumberSpec type: t=0x{:02x}, P={}", t, p)
            }
        }
    } else {
        // signed 7-bit integer, nothing more to read
        VarIntSize::Zero
    }
}

#[derive(Clone)]
pub struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8], pos: usize) -> Self {
        Self { data, pos }
    }
}

impl<'a> Reader<'a> {
    #[must_use]
    pub fn rewound(&self, position: u32) -> Self {
        Self {
            data: self.data,
            pos: position as usize,
        }
    }

    pub fn take(&mut self, size: usize) -> &'a [u8] {
        let res = &self.data[self.pos..self.pos + size];
        self.pos += size;
        res
    }

    pub fn take_u8(&mut self) -> u8 {
        self.take(1)[0]
    }

    pub fn take_u16(&mut self) -> u16 {
        u16::from_le_bytes(self.take(2).try_into().unwrap())
    }

    pub fn take_u32(&mut self) -> u32 {
        u32::from_le_bytes(self.take(4).try_into().unwrap())
    }

    pub fn take_length(&mut self, kind: LengthKind) -> u16 {
        match kind {
            LengthKind::U8Length => self.take_u8() as u16,
            LengthKind::U16Length => self.take_u16(),
        }
    }

    pub fn take_reg(&mut self) -> u16 {
        self.take_u16()
    }
    /// Read a Number, which can be either a register reference or an immediate constant. The number is not fully parsed, but instead stored in a compressed representation as u32.
    pub fn take_number(&mut self, style: NumberStyle) -> u32 {
        match style {
            NumberStyle::U16 => self.take_u16() as u32,
            NumberStyle::VarInt => {
                let t = self.take_u8();

                match var_int_size(t) {
                    VarIntSize::Zero => u32::from_le_bytes([t, 0, 0, 0]),
                    VarIntSize::One => u32::from_le_bytes([t, self.take_u8(), 0, 0]),
                    VarIntSize::Two => u32::from_le_bytes([t, self.take_u8(), self.take_u8(), 0]),
                    VarIntSize::Three => {
                        u32::from_le_bytes([t, self.take_u8(), self.take_u8(), self.take_u8()])
                    }
                }
            }
        }
    }

    pub fn take_offset(&mut self) -> u32 {
        self.take_u32()
    }

    pub fn take_string(&mut self, length_kind: LengthKind) -> &'a [u8] {
        let len = self.take_length(length_kind);
        self.take(len as usize)
    }
    pub fn take_string_array(&mut self, length_kind: LengthKind) -> &'a [u8] {
        let len = self.take_length(length_kind);
        self.take(len as usize)
    }

    /// Reacts to a string prefixed with u8 length. The returned string is not decoded to utf-8. Zero terminator is included in the returned slice.
    pub fn take_u8string(&mut self) -> &'a [u8] {
        self.take_string(LengthKind::U8Length)
    }

    /// Reacts to a string prefixed with u16 length. The returned string is not decoded to utf-8. Zero terminator is included in the returned slice.
    pub fn take_u16string(&mut self) -> &'a [u8] {
        self.take_string(LengthKind::U16Length)
    }

    /// Reacts to a string array prefixed with u8 length. Zero terminators are included in the returned slice.
    ///
    /// String array consists of zero-terminated strings written back-to-back. The array itself is also zero-terminated.
    ///
    /// Example: "foo\0bar\0baz\0\0" -> ["foo", "bar", "baz"]
    pub fn take_u8string_array(&mut self) -> &'a [u8] {
        self.take_string_array(LengthKind::U8Length)
    }

    /// Reacts to a string array prefixed with u16 length. Zero terminators are included in the returned slice.
    ///
    /// String array consists of zero-terminated strings written back-to-back. The array itself is also zero-terminated.
    ///
    /// Example: "foo\0bar\0baz\0\0" -> ["foo", "bar", "baz"]
    pub fn take_u16string_array(&mut self) -> &'a [u8] {
        self.take_string_array(LengthKind::U16Length)
    }

    pub fn has_instr(&self) -> bool {
        // a hacky way to check EOF, respecting the possible padding at the end of file

        // The file is always 16-byte aligned by appending 0x00 bytes to the end
        // So we can check if there are no more instructions by checking if
        // 1. we are 16 bytes or fewer from the end of the file
        // 2. the remaining bytes are all 0x00

        if self.pos + 16 < self.data.len() {
            return true;
        }

        if self.data[self.pos..].iter().all(|&b| b == 0x00) {
            return false;
        }

        true
    }

    pub fn position(&self) -> u32 {
        self.pos as u32
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }
}
