//! A "database" of supported versions of shin engine, listing the properties of each version.
//!
//! The version is specified as a game it was used in, as there doesn't appear to be any available version numbers.

// TODO: maybe generate this from a yaml file or whatnot

use arrayref::array_ref;

/// A version of the shin engine. It uniquely identifies all the file format versions, VM opcode numbers, etc.
///
/// The names of enum variants are based on the developer/publisher's naming scheme.
///
/// For ENTERGRAM it's based on the URL of the game's page on their website, e.g. `https://www.entergram.co.jp/konosuba/` -> `Konosuba`
///
/// For Dramatic Create/FAVORITE it's based on the URL of the game's page on Dramatic Create's website, e.g. `https://dramaticcreate.com/WhiteEternity/` -> `WhiteEternity`
///
/// When referring to these versions in the CLI, use `kebab-case` (e.g. `white-eternity`).
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum ShinVersion {
    /// 2016-09-22 PSVita `PCSG00901`
    WhiteEternity,
    /// 2019-12-19 Switch `0100D8500EE14000`
    DC4,
    /// 2020-08-27 Switch `01004920105FC000`
    Konosuba,
}

/// Describes how `NumberSpec` is encoded in a particular version
pub enum NumberSpecStyle {
    /// `NumberSpec` is stored just as a `u16`. If it's smaller than `0x8000` (I think?) it is a literal, otherwise it is a register reference.
    Short,
    /// `NumberSpec` is encoded as a complicated variable-length encoding. See [https://github.com/DCNick3/shin/blob/4ddd24604c390d50db5191392187ed3ebec39a95/shin-core/src/format/scenario/instruction_elements/number_spec.rs#L38] for full decoding algorithm.
    VarInt,
}

pub enum StringKind {
    Saveinfo,
    // NOTE: this is only for the choice title, not the choices themselves, as they are encoded as an array
    SelectTitle,
    Msgset,
    Dbgout,
    Logset,
    Voiceplay,
    ChatSet,
}

pub enum StringArrayKind {
    SelectChoices,
}

/// Describes how a particular string kind is encoded
pub struct StringStyle {
    pub length_size: LengthSize,
    pub fixup: bool,
}
pub enum LengthSize {
    U8Length,
    U16Length,
}

impl ShinVersion {
    pub fn number_spec_style(&self) -> NumberSpecStyle {
        use NumberSpecStyle::*;
        use ShinVersion::*;

        match self {
            WhiteEternity => Short,
            DC4 | Konosuba => VarInt,
        }
    }

    pub fn string_style(&self, kind: StringKind) -> StringStyle {
        use LengthSize::*;
        use ShinVersion::*;
        use StringKind::*;

        let (length_size, fixup) = match self {
            WhiteEternity => match kind {
                Saveinfo | SelectTitle | Dbgout | Voiceplay => (U8Length, false),
                Msgset | Logset => (U16Length, true),
                ChatSet => {
                    // not in this game
                    unreachable!()
                }
            },
            DC4 => match kind {
                Saveinfo | SelectTitle | Dbgout | Voiceplay => (U16Length, false),
                Msgset => (U16Length, true),
                Logset => {
                    // not in this game
                    unreachable!()
                }
                // I _believe_ the SNR does not use any strings using the fixup encoding with this command
                ChatSet => (U16Length, false),
            },
            Konosuba => todo!(),
        };

        StringStyle { length_size, fixup }
    }

    pub fn string_array_style(&self, kind: StringArrayKind) -> StringStyle {
        use LengthSize::*;
        use ShinVersion::*;
        use StringArrayKind::*;

        let (length_size, fixup) = match self {
            WhiteEternity => match kind {
                SelectChoices => (U8Length, true),
            },
            DC4 => match kind {
                SelectChoices => (U16Length, false),
            },
            Konosuba => todo!(),
        };

        StringStyle { length_size, fixup }
    }

    pub fn rom_version(&self) -> Option<RomVersion> {
        use RomVersion::*;
        use ShinVersion::*;
        Some(match self {
            WhiteEternity => Rom2V1_0,
            DC4 => Rom2V1_1,
            // konosuba doesn't store its assets in the rom, it just uses switch's romfs
            Konosuba => return None,
        })
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum RomVersion {
    /// 'ROM ' magic, version 0x00020001
    Rom1V2_1,
    /// 'ROM2' magic, version 0x00000001
    Rom2V1_0,
    /// 'ROM2' magic, version 0x00010001
    Rom2V1_1,
}

/// Describes how the text is encoded in a particular version
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum RomEncoding {
    Utf8,
    ShiftJIS,
}

/// Describes how the directory offset is calculated in a particular version
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum RomDirectoryOffsetDisposition {
    /// From start of the ROM file
    FromStart,
    /// From start of the index in the ROM file
    FromIndexStart,
}

impl RomVersion {
    pub const HEAD_BYTES_SIZE: usize = 8;

    /// Detects the version of the ROM file from the first 8 bytes and panics if it's unknown
    pub fn detect(head_bytes: &[u8; Self::HEAD_BYTES_SIZE]) -> Self {
        match Self::try_detect(head_bytes) {
            Some(version) => version,
            None => panic!("Unknown ROM version: {:?}", head_bytes),
        }
    }

    /// Tries to detect the version of the ROM file from the first 8 bytes
    pub fn try_detect(head_bytes: &[u8; Self::HEAD_BYTES_SIZE]) -> Option<Self> {
        assert_eq!(Self::HEAD_BYTES_SIZE, 8);
        let h = head_bytes;
        let magic = array_ref![h, 0, 4];
        let version = u32::from_le_bytes(*array_ref![h, 4, 4]);

        Some(match (magic, version) {
            (b"ROM ", 0x00020001) => RomVersion::Rom1V2_1,
            (b"ROM2", 0x00000001) => RomVersion::Rom2V1_0,
            (b"ROM2", 0x00010001) => RomVersion::Rom2V1_1,
            _ => return None,
        })
    }

    /// Returns the 8-byte header that is present at the beginning of every ROM file of this version
    pub fn head_bytes(&self) -> [u8; Self::HEAD_BYTES_SIZE] {
        use RomVersion::*;
        match self {
            Rom1V2_1 => *b"ROM \x01\x00\x02\x00",
            Rom2V1_0 => *b"ROM2\x01\x00\x00\x00",
            Rom2V1_1 => *b"ROM2\x01\x00\x01\x00",
        }
    }

    pub fn encoding(&self) -> RomEncoding {
        use RomEncoding::*;
        use RomVersion::*;
        match self {
            // Either ShiftJIS or no CJK name support at all
            // All the games I have use no non-ASCII characters in their names, so I can't tell
            // Guessing ShiftJIS here to be more lenient, but no idea honestly
            Rom1V2_1 => ShiftJIS,
            // Definitely ShiftJIS, WhiteEternity has some ShiftJIS-encoded CJK characters
            Rom2V1_0 => ShiftJIS,
            // Definitely unicode, Gerokasu has some UTF-8-encoded CJK characters
            Rom2V1_1 => Utf8,
        }
    }

    pub fn directory_offset_disposition(&self) -> RomDirectoryOffsetDisposition {
        use RomDirectoryOffsetDisposition::*;
        use RomVersion::*;
        match self {
            Rom1V2_1 => FromStart,
            Rom2V1_0 | Rom2V1_1 => FromIndexStart,
        }
    }
}
