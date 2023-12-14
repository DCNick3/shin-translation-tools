mod allocate;
mod source;
mod visit;
mod write;

use std::io::BufWriter;

use bumpalo::Bump;
use camino::Utf8PathBuf;
use shin_versions::RomVersion;
use source::InputDirectory;
use tracing::info;

use crate::{
    default_spinner_span,
    progress::{ProgressAction, RomTimingSummary},
};

// FIXME: the API only allowing the use of filesystem paths is a bit limiting. We should be able to abstract away from concrete source and destination types here
pub fn rom_create(source_directory: Utf8PathBuf, output_path: Utf8PathBuf, version: RomVersion) {
    let timing_summary = RomTimingSummary::new(ProgressAction::Create);

    let bump = Bump::new();

    let source_directory = {
        let _span = default_spinner_span!("Collecting input files");
        InputDirectory::walk(&bump, version.encoding(), &source_directory)
    };

    let allocated = {
        let _span = default_spinner_span!("Allocating file positions");
        allocate::rom_allocate(&bump, version, &source_directory)
    };

    let output_file = std::fs::File::create(output_path).expect("Failed to create output file");
    let mut output_writer = BufWriter::new(output_file);

    let total_count = write::rom_write(version, &source_directory, &allocated, &mut output_writer)
        .expect("Failed to write output file");

    timing_summary.finish(total_count);

    info!(
        "Used bump memory: {}",
        bytesize::ByteSize(bump.allocated_bytes() as u64)
    );
}
