mod iter;

use std::{fs::File, io::Read as _};

use binrw::BinRead as _;
use bumpalo::Bump;
use camino::Utf8PathBuf;
use shin_versions::{RomEncoding, RomVersion};
use tracing::info;

use self::iter::{DirectoryIterCtx, EntryContent};
use crate::{
    header::RomHeader,
    progress::{ProgressAction, RomProgress, RomTimingSummary},
};

// FIXME: the API only allowing the use of filesystem paths is a bit limiting. We should be able to abstract away from concrete source and destination types here
// an API to access the ROM files individually as in the game would also be nice
pub fn rom_extract(rom_path: Utf8PathBuf, output_path: Utf8PathBuf, version: Option<RomVersion>) {
    info!("Extracting {:?} to {:?}", rom_path, output_path);

    let timing_summary = RomTimingSummary::new(ProgressAction::Extract);

    let rom_file = File::open(&rom_path).expect("Failed to open rom file");
    let rom_file = unsafe { memmap2::Mmap::map(&rom_file) }.expect("Failed to mmap rom file");

    let rom = rom_file.as_ref();
    let mut cursor = std::io::Cursor::new(&rom);

    let mut head_bytes = [0; RomVersion::HEAD_BYTES_SIZE];
    cursor.read_exact(&mut head_bytes).unwrap();

    let version = version.unwrap_or_else(|| RomVersion::detect(&head_bytes));
    info!("Extracting ROM as {:?}", version);

    let header = RomHeader::read_args(&mut cursor, (version,)).unwrap();
    info!("Header: {:x?}", header);

    #[cfg(unix)]
    rom_file
        .advise_range(
            memmap2::Advice::WillNeed,
            cursor.position() as usize,
            header.index_size(),
        )
        .expect("Failed to advise rom index");

    let index = &rom[cursor.position() as usize..][..header.index_size()];

    let ctx = DirectoryIterCtx {
        bump: Bump::new(),
        version,
        index_start_offset: cursor.position() as usize,
        file_offset_multiplier: header.file_offset_multiplier(),
        index,
        rom,
    };

    let data_start = cursor.position() as usize + header.index_size();

    // TODO: measure perf impact of this
    #[cfg(unix)]
    rom_file
        .advise_range(
            memmap2::Advice::WillNeed,
            data_start,
            rom.len() - data_start,
        )
        .expect("Failed to advise rom file");

    std::fs::create_dir_all(&output_path).expect("Failed to create output directory");

    // change the current directory so that we can allocate less
    std::env::set_current_dir(&output_path).expect("Failed to set current directory");

    // first, create all the directories
    iter::walk_rom(&ctx, |path, entry| match entry {
        EntryContent::File(_) => {}
        EntryContent::Directory(_) => {
            if let Err(e) = std::fs::create_dir_all(path) {
                panic!("Failed to create directory {:?}: {}", path, e)
            }
        }
    });

    let total_counts = iter::rom_count_total(&ctx);
    {
        let mut progress = RomProgress::new(total_counts);

        iter::walk_rom(&ctx, |path, entry| match entry {
            EntryContent::File(content) => {
                progress.add_file(content.len() as u64);
                if let Err(e) = std::fs::write(path, content) {
                    panic!("Failed to write file {:?}: {}", path, e)
                }
            }
            EntryContent::Directory(_) => {}
        });
    }

    timing_summary.finish(total_counts);

    if version.encoding() != RomEncoding::Utf8 {
        let used_memory = ctx.bump.allocated_bytes();
        info!("Used string memory: {} bytes", used_memory);
    }
}
