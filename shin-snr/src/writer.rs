use shin_versions::{LengthKind, NumberStyle};

use crate::reader::var_int_size;

pub trait Writer {
    fn put(&mut self, data: &[u8]);
    fn position(&self) -> u32;

    fn put_u8(&mut self, value: u8) {
        self.put(&[value])
    }

    fn put_u16(&mut self, value: u16) {
        self.put(&value.to_le_bytes());
    }

    fn put_u32(&mut self, value: u32) {
        self.put(&value.to_le_bytes());
    }

    fn put_length(&mut self, kind: LengthKind, value: usize) {
        match kind {
            LengthKind::U8Length => self.put_u8(
                value
                    .try_into()
                    .expect("The passed length does not fit into the specified LengthKind"),
            ),
            LengthKind::U16Length => self.put_u16(
                value
                    .try_into()
                    .expect("The passed length does not fit into the specified LengthKind"),
            ),
        }
    }

    fn put_reg(&mut self, value: u16) {
        self.put_u16(value)
    }
    fn put_number(&mut self, style: NumberStyle, number: u32) {
        match style {
            NumberStyle::U16 => self.put_u16(number.try_into().unwrap()),
            NumberStyle::VarInt => {
                let bytes = number.to_le_bytes();

                let size = 1 + usize::from(var_int_size(bytes[0]));
                self.put(&bytes[..size]);
            }
        }
    }

    fn put_offset(&mut self, value: u32) {
        self.put_u32(value)
    }

    fn put_string(&mut self, length_kind: LengthKind, string: &[u8]) {
        self.put_length(length_kind, string.len());
        self.put(string);
    }
    fn put_string_array(&mut self, length_kind: LengthKind, string_array: &[u8]) {
        self.put_length(length_kind, string_array.len());
        self.put(string_array);
    }

    fn put_u8string(&mut self, string: &[u8]) {
        self.put_string(LengthKind::U8Length, string)
    }

    fn put_u16string(&mut self, string: &[u8]) {
        self.put_string(LengthKind::U16Length, string)
    }

    fn put_u8string_array(&mut self, string_array: &[u8]) {
        self.put_string_array(LengthKind::U8Length, string_array);
    }

    fn put_u16string_array(&mut self, string_array: &[u8]) {
        self.put_string_array(LengthKind::U16Length, string_array);
    }

    fn pad_16(&mut self) {
        for _ in 0..self.position().next_multiple_of(16) - self.position() {
            self.put_u8(0);
        }
    }
}

pub struct CountingWriter {
    position: u32,
}

impl CountingWriter {
    pub fn new(position: u32) -> Self {
        Self { position }
    }

    pub fn into_position(self) -> u32 {
        self.position
    }
}

impl Writer for CountingWriter {
    #[inline]
    fn put(&mut self, data: &[u8]) {
        self.position += data.len() as u32;
    }

    #[inline]
    fn position(&self) -> u32 {
        self.position
    }
}

pub struct RealWriter {
    buffer: Vec<u8>,
}

impl RealWriter {
    pub fn new(buffer: Vec<u8>) -> Self {
        Self { buffer }
    }

    pub fn into_buffer(self) -> Vec<u8> {
        self.buffer
    }
}

impl Writer for RealWriter {
    #[inline]
    fn put(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    #[inline]
    fn position(&self) -> u32 {
        self.buffer.len().try_into().unwrap()
    }
}
