use camino::Utf8PathBuf;
use clap::Subcommand;

use crate::rom::version::RomVersionSpecifier;

mod version;

/// Extract or package .rom files used by shin-based games
///
/// For more usage documentation see https://github.com/DCNick3/shin-translation-tools
#[derive(Subcommand)]
pub enum Command {
    /// Extract all files from a rom.
    Extract {
        /// The path to the rom to extract.
        rom_path: Utf8PathBuf,
        /// The path to the directory to extract to.
        output_path: Utf8PathBuf,
        /// Specify the version of the rom format to use. Will be detected automatically if not specified.
        #[clap(short, long, value_parser = RomVersionSpecifier::parser())]
        rom_version: Option<RomVersionSpecifier>,
    },
    /// Create a rom from a file tree.
    Create {
        /// Directory to package into a ROM file
        source_directory: Utf8PathBuf,
        /// The path to the output ROM file
        output_path: Utf8PathBuf,
        /// Version of the ROM format to use or a game ID
        #[clap(short, long, value_parser = RomVersionSpecifier::parser())]
        rom_version: RomVersionSpecifier,
    },
    /// Print some information about a rom file.
    Info {
        /// The path to the rom to print info on.
        rom_path: Utf8PathBuf,
        /// Specify the version of the rom format to use. Will be detected automatically if not specified.
        #[clap(short, long, value_parser = RomVersionSpecifier::parser())]
        rom_version: Option<RomVersionSpecifier>,
    },
}

impl Command {
    pub fn run(self) {
        match self {
            Command::Extract {
                rom_path,
                output_path,
                rom_version,
            } => shin_rom::rom_extract(rom_path, output_path, rom_version.map(|v| v.rom_version())),
            Command::Info {
                rom_path,
                rom_version,
            } => shin_rom::rom_info(rom_path, rom_version.map(|v| v.rom_version())),
            Command::Create {
                source_directory,
                output_path,
                rom_version,
            } => shin_rom::rom_create(source_directory, output_path, rom_version.rom_version()),
        }
    }
}
