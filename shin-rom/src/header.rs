// this excludes all the magic/version info

use binrw::{BinRead, BinWrite};
use shin_versions::RomVersion;

#[derive(BinRead, BinWrite, Debug)]
pub struct RomHeaderV1 {
    pub index_size: u32,
    pub unk: [u8; 4],
}

impl RomHeaderV1 {
    pub const DEFAULT_FILE_OFFSET_MULTIPLIER: usize = 0x800;
}

#[derive(BinRead, BinWrite, Debug)]
pub struct RomHeaderV2 {
    pub index_size: u32,
    pub file_offset_multiplier: u32,
    pub unk: [u8; 16],
}

impl RomHeaderV2 {
    pub const DEFAULT_FILE_OFFSET_MULTIPLIER: usize = 0x200;
}

#[derive(BinRead, BinWrite, Debug)]
#[brw(little)]
#[br(import(version: RomVersion))]
pub enum RomHeader {
    #[br(pre_assert(version == RomVersion::RomV2_1))]
    V1(RomHeaderV1),
    #[br(pre_assert(version == RomVersion::Rom2V0_1 || version == RomVersion::Rom2V1_1))]
    V2(RomHeaderV2),
}

impl RomHeader {
    pub fn index_size(&self) -> usize {
        match self {
            RomHeader::V1(h) => h.index_size as usize,
            RomHeader::V2(h) => h.index_size as usize,
        }
    }

    pub fn file_offset_multiplier(&self) -> usize {
        match self {
            // in rom v1, the file offset multiplier is constant
            RomHeader::V1(_) => RomHeaderV1::DEFAULT_FILE_OFFSET_MULTIPLIER,
            // in newer roms, it's stored in the header (though in practice it's always 0x200?)
            RomHeader::V2(h) => h.file_offset_multiplier as usize,
        }
    }
}