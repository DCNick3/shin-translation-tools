use std::{fs::File, io::Read as _};

use binrw::BinRead as _;
use bumpalo::Bump;
use camino::Utf8PathBuf;
use clap::Parser;
use memmap2::Advice;
use shin_versions::{RomEncoding, RomVersion};

use crate::{
    header::RomHeader,
    index::{DirectoryIterCtx, EntryContent},
    iter_rom,
    progress::ExtractProgress,
    version::RomVersionSpecifier,
};

/// Extract all files from a rom.
#[derive(Parser)]
pub struct Extract {
    /// The path to the rom to extract.
    rom_path: Utf8PathBuf,
    /// The path to the directory to extract to.
    output_path: Utf8PathBuf,
    /// Specify the version of the rom format to use. Will be detected automatically if not specified.
    #[clap(short, long, value_parser = RomVersionSpecifier::parser())]
    version: Option<RomVersionSpecifier>,
}

impl Extract {
    pub fn run(self) {
        let Extract {
            rom_path,
            output_path,
            version,
        } = self;
        eprintln!("Extracting {:?} to {:?}", rom_path, output_path);

        let version = version.map(|v| v.rom_version());
        let rom_file = File::open(&rom_path).expect("Failed to open rom file");
        let rom_file = unsafe { memmap2::Mmap::map(&rom_file) }.expect("Failed to mmap rom file");

        let rom = rom_file.as_ref();
        let mut cursor = std::io::Cursor::new(&rom);

        let mut head_bytes = [0; RomVersion::HEAD_BYTES_SIZE];
        cursor.read_exact(&mut head_bytes).unwrap();

        let version = version.unwrap_or_else(|| RomVersion::detect(&head_bytes));
        eprintln!("Extracting ROM as {:?}", version);

        let header = RomHeader::read_args(&mut cursor, (version,)).unwrap();
        eprintln!("Header: {:x?}", header);

        #[cfg(unix)]
        rom_file
            .advise_range(
                Advice::WillNeed,
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
            directory_offset_multiplier: 16,
            index,
            rom,
        };

        let data_start = cursor.position() as usize + header.index_size();

        // TODO: measure perf impact of this
        #[cfg(unix)]
        rom_file
            .advise_range(Advice::WillNeed, data_start, rom.len() - data_start)
            .expect("Failed to advise rom file");

        std::fs::create_dir_all(&output_path).expect("Failed to create output directory");

        // change the current directory so that we can allocate less
        std::env::set_current_dir(&output_path).expect("Failed to set current directory");

        // first, create all the directories
        iter_rom(&ctx, |path, entry| match entry {
            EntryContent::File(_) => {}
            EntryContent::Directory(_) => {
                if let Err(e) = std::fs::create_dir_all(path) {
                    panic!("Failed to create directory {:?}: {}", path, e)
                }
            }
        });

        let mut progress = ExtractProgress::new(&ctx);

        iter_rom(&ctx, |path, entry| {
            progress.add(entry);
            match entry {
                EntryContent::File(content) => {
                    if let Err(e) = std::fs::write(path, content) {
                        panic!("Failed to write file {:?}: {}", path, e)
                    }
                }
                EntryContent::Directory(_) => {}
            }
        });

        progress.finish();

        if version.encoding() != RomEncoding::Utf8 {
            let used_memory = ctx.bump.allocated_bytes();
            eprintln!("Used string memory: {} bytes", used_memory);
        }
    }
}
