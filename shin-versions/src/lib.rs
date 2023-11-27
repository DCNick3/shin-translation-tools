//! A "database" of supported versions of shin engine, listing the properties of each version.
//!
//! The version is specified as a game it was used in, as there doesn't appear to be any available version numbers.

// TODO: maybe generate this from a yaml file or whatnot

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum ShinVersion {
    /// 2016-09-22 PSVita `PCSG00901`
    AstralAir,
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
            AstralAir => Short,
            Konosuba => VarInt,
        }
    }

    pub fn string_style(&self, kind: StringKind) -> StringStyle {
        use LengthSize::*;
        use ShinVersion::*;
        use StringKind::*;

        let (length_size, fixup) = match self {
            AstralAir => match kind {
                Saveinfo | SelectTitle | Dbgout | Voiceplay => (U8Length, false),
                Msgset | Logset => (U16Length, true),
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
            AstralAir => match kind {
                SelectChoices => (U8Length, true),
            },
            Konosuba => todo!(),
        };

        StringStyle { length_size, fixup }
    }
}
