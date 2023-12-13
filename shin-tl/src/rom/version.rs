use clap::builder::TypedValueParser;
use shin_versions::{RomVersion, ShinVersion};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum RomVersionSpecifier {
    RomVersion(RomVersion),
    ShinVersion(ShinVersion),
}

impl RomVersionSpecifier {
    pub fn rom_version(&self) -> RomVersion {
        match *self {
            RomVersionSpecifier::RomVersion(version) => version,
            RomVersionSpecifier::ShinVersion(version) => version
                .rom_version()
                .expect("The specified version of shin version does not use ROMs"),
        }
    }

    fn to_possible_value(self) -> Option<clap::builder::PossibleValue> {
        use clap::ValueEnum;

        match self {
            RomVersionSpecifier::RomVersion(v) => v.to_possible_value(),
            RomVersionSpecifier::ShinVersion(v) => v.to_possible_value(),
        }
    }

    pub(super) fn parser() -> impl TypedValueParser<Value = Self> {
        use clap::ValueEnum;

        let variants = RomVersion::value_variants()
            .iter()
            .copied()
            .map(RomVersionSpecifier::RomVersion)
            .chain(
                ShinVersion::value_variants()
                    .iter()
                    .copied()
                    .map(RomVersionSpecifier::ShinVersion),
            )
            .collect::<Vec<_>>();

        clap::builder::PossibleValuesParser::new(
            variants.iter().copied().flat_map(|v| v.to_possible_value()),
        )
        .map(move |v| {
            variants
                .iter()
                .find_map(|&x| x.to_possible_value()?.matches(&v, true).then_some(x))
                .unwrap()
                .clone()
        })
    }
}
