use std::io::BufWriter;

use bumpalo::Bump;
use camino::Utf8PathBuf;
use clap::Parser;

use crate::{
    create::{rom_allocate, rom_write, InputDirectory},
    default_spinner_span,
    progress::{ProgressAction, RomTimingSummary},
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

        let timing_summary = RomTimingSummary::new(ProgressAction::Create);

        let version = version.rom_version();

        let bump = Bump::new();

        let source_directory = {
            let _span = default_spinner_span!("Collecting input files");
            InputDirectory::walk(&bump, &source_directory)
        };

        let allocated = {
            let _span = default_spinner_span!("Allocating file positions");
            rom_allocate(&bump, version, &source_directory)
        };

        let output_file =
            std::fs::File::create(&output_path).expect("Failed to create output file");
        let mut output_writer = BufWriter::new(output_file);

        let total_count = rom_write(version, &source_directory, &allocated, &mut output_writer)
            .expect("Failed to write output file");

        timing_summary.finish(total_count);
    }
}
