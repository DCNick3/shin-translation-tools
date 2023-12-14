//! Definitions for some of the ROM index structures, including reading and writing routines thanks to [`binrw`].

use binrw::{BinRead, BinWrite};
use proc_bitfield::bitfield;

pub const DIRECTORY_OFFSET_MULTIPLIER: usize = 0x10;

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, BinRead, BinWrite)]
    pub struct NameOffsetAndFlags(pub u32) : Debug {
        /// Offset to the name of the entry, in bytes
        ///
        /// This is relative to the beginning of the directory data.
        pub name_offset: u32 @ 0..31,
        // blabla.py assums that the whole top byte is flags, but I don't think that's true?
        // No games from PS3 and onwards that I saw use anything besides the `is_directory` flag
        pub is_directory: bool @ 31,
    }
}

/// One directory entry in the ROM index describing a file or a directory
#[derive(BinRead, BinWrite, Debug)]
#[brw(little)]
pub struct RawEntry {
    /// Combined name offset and `is_directory` flag
    pub name_and_flags: NameOffsetAndFlags,
    /// Offset to the entry data, divided by offset multiplier (so to get an actual offset, multiply this by the offset multiplier)
    ///
    /// For files, the offset multiplier can be known from [`crate::header::RomHeader::file_offset_multiplier`] (it's fixed for V1 and stored in the header for V2).
    ///
    /// For directories, the offset multiplier is always [`DIRECTORY_OFFSET_MULTIPLIER`].
    ///
    /// For files, offset is calculated relative to the beginning of the file.
    ///
    /// For directories, depending on [`shin_versions::RomVersion::directory_offset_disposition`], offset is either relative to the beginning of the file, or relative to the beginning of the index.
    pub data_offset: u32,
    /// Size of the entry data, in bytes
    ///
    /// No multipliers are applied to this value.
    pub data_size: u32,
}

impl RawEntry {
    pub const SIZE: usize = 0xc;
}
