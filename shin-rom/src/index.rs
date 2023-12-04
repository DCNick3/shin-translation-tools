use std::io;

use arrayref::array_ref;
use binrw::{BinRead, BinWrite};
use bumpalo::Bump;
use proc_bitfield::bitfield;
use shin_versions::{RomEncoding, RomVersion};

pub struct DirectoryIterCtx<'rom> {
    pub bump: Bump,
    pub version: RomVersion,
    pub index_start_offset: usize,
    pub file_offset_multiplier: usize,
    pub directory_offset_multiplier: usize,
    pub index: &'rom [u8],
    pub rom: &'rom [u8],
}

pub struct DirectoryIter<'rom, 'ctx> {
    ctx: &'ctx DirectoryIterCtx<'rom>,
    start_offset: usize,
    cur: io::Cursor<&'rom [u8]>,
}

impl<'rom, 'ctx> DirectoryIter<'rom, 'ctx> {
    pub fn new(
        ctx: &'ctx DirectoryIterCtx<'rom>,
        start_offset: usize,
    ) -> DirectoryIter<'rom, 'ctx> {
        let count = u32::from_le_bytes(*array_ref![ctx.index, start_offset, 4]);
        let cur = io::Cursor::new(&ctx.index[start_offset + 4..][..count as usize * 0xc]);

        DirectoryIter {
            ctx,
            start_offset,
            cur,
        }
    }
}

impl<'rom, 'ctx> Iterator for DirectoryIter<'rom, 'ctx> {
    type Item = Entry<'rom, 'ctx>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.cur.position() as usize >= self.cur.get_ref().len() {
                return None;
            }
            let entry = RawEntry::read_args(&mut self.cur, (self.ctx.version,)).unwrap();
            let name_offset = self.start_offset + entry.name_and_flags.name_offset() as usize;
            let name_len = self.ctx.index[name_offset..]
                .iter()
                .position(|&c| c == 0)
                .unwrap();

            let name = &self.ctx.index[name_offset..][..name_len];

            if matches!(name, b"." | b"..") {
                continue;
            }

            let name = match self.ctx.version.encoding() {
                RomEncoding::Utf8 => shin_text::Cow::Borrowed(
                    std::str::from_utf8(name).expect("invalid utf8 in rom filename"),
                ),
                RomEncoding::ShiftJIS => shin_text::Cow::Owned(
                    shin_text::decode_sjis_zstring(&self.ctx.bump, name, false)
                        .expect("invalid shift-jis in rom filename"),
                ),
            };

            let offset_multiplier = if entry.name_and_flags.is_directory() {
                self.ctx.directory_offset_multiplier
            } else {
                self.ctx.file_offset_multiplier
            };
            let mut data_offset = entry.data_offset as usize * offset_multiplier;

            break Some(Entry {
                name,
                content: if entry.name_and_flags.is_directory() {
                    // weirdly, v1 specifies offset relative to the start of the ROM
                    // and v2 specifies it relative to the start of the index
                    // we handle the v2 case generally and special-case v1 by subtracting the index start offset
                    if let RomVersion::Rom1V2_1 = self.ctx.version {
                        data_offset -= self.ctx.index_start_offset;
                    }
                    EntryContent::Directory(DirectoryIter::new(self.ctx, data_offset))
                } else {
                    EntryContent::File(&self.ctx.rom[data_offset..][..entry.data_size as usize])
                },
            });
        }
    }
}

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, BinRead, BinWrite)]
    pub struct NameOffsetAndFlagsV1(pub u32) : Debug {
        pub name_offset: u32 @ 0..24,
        pub is_directory: bool @ 31,
        // there are some other flags, maybe we care about them?
    }
}

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, BinRead, BinWrite)]
    pub struct NameOffsetAndFlagsV2(pub u32) : Debug {
        pub name_offset: u32 @ 0..31,
        pub is_directory: bool @ 31,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, BinRead, BinWrite)]
#[br(import(version: RomVersion))]
enum NameOffsetAndFlags {
    #[br(pre_assert(version == RomVersion::Rom1V2_1))]
    V1(NameOffsetAndFlagsV1),
    #[br(pre_assert(version == RomVersion::Rom2V0_1 || version == RomVersion::Rom2V1_1))]
    V2(NameOffsetAndFlagsV2),
}

impl NameOffsetAndFlags {
    pub fn name_offset(&self) -> u32 {
        match self {
            NameOffsetAndFlags::V1(v) => v.name_offset(),
            NameOffsetAndFlags::V2(v) => v.name_offset(),
        }
    }

    pub fn is_directory(&self) -> bool {
        match self {
            NameOffsetAndFlags::V1(v) => v.is_directory(),
            NameOffsetAndFlags::V2(v) => v.is_directory(),
        }
    }
}

#[derive(BinRead, BinWrite, Debug)]
#[br(import(version: RomVersion))]
#[brw(little)]
struct RawEntry {
    // name offset is from the beginning of the entry
    #[br(args(version))]
    pub name_and_flags: NameOffsetAndFlags,
    // data offset is from from the beginning of the archive file
    pub data_offset: u32,
    pub data_size: u32,
}

pub struct Entry<'rom, 'bump> {
    pub name: shin_text::Cow<'bump, 'rom>,
    pub content: EntryContent<'rom, 'bump>,
}

pub enum EntryContent<'rom, 'bump> {
    File(&'rom [u8]),
    Directory(DirectoryIter<'rom, 'bump>),
}
