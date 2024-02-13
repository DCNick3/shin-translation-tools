use std::{
    fs::File,
    io::{BufWriter, Seek, Write},
};

use camino::Utf8PathBuf;
use clap::{Args, Subcommand};
use shin_snr::{
    react_with,
    reactor::{
        offset_validator::OffsetValidatorReactor,
        rewrite::{CsvRewriter, RewriteReactor},
        trace::{ConsoleTraceListener, CsvTraceListener, StringTraceReactor},
    },
    reader::Reader,
};
use shin_versions::ShinVersion;
use tracing::{error, info};

#[derive(Args, Clone)]
pub struct CommonArgs {
    /// Version of the engine the SNR file is from
    #[clap(value_enum)]
    engine_version: ShinVersion,
    /// Path to the SNR file
    snr_file: Utf8PathBuf,
}

/// Rewrite shin SNR file with translated strings
///
/// TL;DR:
///
/// 1. shin-tl snr read <engine-version> <main.snr> <strings.csv>
///
/// 2. Translate strings in strings.csv
///
/// 3. shin-tl snr rewrite <engine-version> <main.snr> <strings.csv> <main_translated.snr>
///
/// For more usage documentation see https://github.com/DCNick3/shin-translation-tools/blob/master/shin-tl/README.md
#[derive(Subcommand)]
pub enum Command {
    /// Read strings from an SNR file to a CSV file for translation
    Read {
        #[clap(flatten)]
        common: CommonArgs,
        output: Utf8PathBuf,
    },
    /// Read strings from an SNR file and dump them to the console
    ReadConsole {
        #[clap(flatten)]
        common: CommonArgs,
    },
    /// Read SNR file, while validating jump offsets in the code
    ReadValidateOffsets {
        #[clap(flatten)]
        common: CommonArgs,
    },
    /// Rewrite an SNR file to use translated strings from a CSV file
    Rewrite {
        #[clap(flatten)]
        common: CommonArgs,
        translations: Utf8PathBuf,
        output: Utf8PathBuf,
    },
}

impl Command {
    pub fn run(self) {
        let common = match &self {
            Command::Read { common, .. } => common,
            Command::ReadConsole { common, .. } => common,
            Command::ReadValidateOffsets { common, .. } => common,
            Command::Rewrite { common, .. } => common,
        };

        let snr_file = std::fs::read(&common.snr_file).expect("Reading the SNR file failed");
        let version = common.engine_version;

        assert_eq!(&snr_file[0..4], b"SNR ", "SNR file magic mismatch");
        let code_offset = u32::from_le_bytes(snr_file[0x20..0x24].try_into().unwrap());

        let reader = Reader::new(&snr_file, code_offset as usize);

        match self {
            Command::Read { common: _, output } => {
                let writer = csv::Writer::from_path(output).expect("Opening the CSV file failed");

                let mut reactor = StringTraceReactor::new(reader, CsvTraceListener::new(writer));

                react_with(&mut reactor, version);
            }
            Command::ReadConsole { common: _ } => {
                let mut reactor = StringTraceReactor::new(reader, ConsoleTraceListener);

                react_with(&mut reactor, version);
            }
            Command::ReadValidateOffsets { common: _ } => {
                let mut reactor = OffsetValidatorReactor::new(reader);

                react_with(&mut reactor, version);

                match reactor.validate() {
                    Ok(_) => {
                        info!("All offsets are valid");
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
            Command::Rewrite {
                common: _,
                translations,
                output,
            } => {
                let translations =
                    csv::Reader::from_path(translations).expect("Opening the CSV file failed");
                let rewriter = CsvRewriter::new(translations);

                let mut reactor = RewriteReactor::new(reader, rewriter, code_offset);
                react_with(&mut reactor, version);

                let output_size = reactor.output_size().next_multiple_of(16);

                let output = File::create(output).expect("Opening the output file failed");
                let mut output = BufWriter::new(output);

                // copy the magic
                output
                    .write_all(&snr_file[0..4])
                    .expect("Writing to the output file failed");
                // re-write with the correct file sizes
                // even though some engine versions do not care about this fields, some absolutely do!
                // (for example, DC4)
                output
                    .write_all(&output_size.to_le_bytes())
                    .expect("Writing to the output file failed");
                // copy the rest of the header
                output
                    .write_all(&snr_file[8..code_offset as usize])
                    .expect("Writing to the output file failed");

                assert_eq!(
                    code_offset as u64,
                    output.stream_position().unwrap(),
                    "Written header size does not match the expected size"
                );

                let mut reactor = reactor.into_emit(&mut output);
                react_with(&mut reactor, version);

                // align the file size to 16 bytes
                let mut output = output
                    .into_inner()
                    .expect("Flushing the output file failed");
                let current_size = output
                    .stream_position()
                    .expect("Getting the current file size failed");
                let padding = current_size.next_multiple_of(16) - current_size;
                output
                    .write_all(&vec![0; padding as usize])
                    .expect("Writing to the output file failed");

                assert_eq!(
                    output_size as u64,
                    output.stream_position().unwrap(),
                    "Output file size does not match the expected size"
                );
            }
        }
    }
}
