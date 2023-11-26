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
    pub fn take(&mut self, size: usize) -> &[u8] {
        let res = &self.data[self.pos..self.pos + size];
        self.pos += size;
        res
    }

    pub fn byte(&mut self) -> u8 {
        self.take(1)[0]
    }

    pub fn short(&mut self) -> u16 {
        u16::from_le_bytes(self.take(2).try_into().unwrap())
    }

    pub fn reg(&mut self) -> u16 {
        self.short()
    }

    pub fn offset(&mut self) -> u32 {
        u32::from_le_bytes(self.take(4).try_into().unwrap())
    }

    pub fn u8string(&mut self) -> &[u8] {
        let len = self.byte();
        self.take(len as usize)
    }

    pub fn u16string(&mut self) -> &[u8] {
        let len = self.short();
        self.take(len as usize)
    }

    pub fn u8string_array(&mut self) -> &[u8] {
        let len = self.byte();
        self.take(len as usize)
    }
    pub fn msgid(&mut self) -> u32 {
        let data = self.take(3);
        u32::from_le_bytes([data[0], data[1], data[2], 0])
    }

    pub fn has_instr(&self) -> bool {
        // a hacky way to check EOF, respecting the possible padding at the end of file

        // The file is always 16-byte aligned by appending 0x00 bytes to the end
        // So we can check if there are no more instructions by checking if
        // 1. we are 16 bytes or less from the end of the file
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
}
