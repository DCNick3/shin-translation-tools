use bumpalo::Bump;
use camino::Utf8PathBuf;
use clap::Parser;

use crate::{
    create::{allocate_index, InputDirectory},
    version::RomVersionSpecifier,
};

/// Create a ROM file from a file tree
#[derive(Parser)]
pub struct Create {
    source_directory: Utf8PathBuf,
    output_path: Utf8PathBuf,
    #[clap(short, long, value_parser = RomVersionSpecifier::parser())]
    version: RomVersionSpecifier,
}

impl Create {
    pub fn run(self) {
        let Create {
            source_directory,
            output_path,
            version,
        } = self;

        let version = version.rom_version();

        let bump = Bump::new();
        let source_directory = InputDirectory::walk(&bump, &source_directory);
        allocate_index(&bump, version, &source_directory);

        todo!()
    }
}
