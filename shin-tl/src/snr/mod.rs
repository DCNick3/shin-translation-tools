use std::{
    fs::File,
    io::{BufWriter, Seek, Write},
};

use camino::Utf8PathBuf;
use clap::{Args, Subcommand};
use shin_snr::{
    react_with,
    reactor::{
        location_painter::LocationPainterReactor,
        offset_validator::OffsetValidatorReactor,
        rewrite::{CsvRewriter, NoopRewriter, RewriteReactor, StringRewriter},
        string_roundrip_validator::StringRoundtripValidatorReactor,
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
    /// Run shin-tl tests on an SNR file
    ///
    /// This command is only intended to be used for testing shin-tl itself, not of any use to end users
    Test {
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

fn rewrite_snr<R, O>(
    snr_file: &[u8],
    reader: Reader,
    code_offset: u32,
    version: ShinVersion,
    rewriter: R,
    mut output: &mut O,
) where
    R: StringRewriter,
    O: Write + Seek,
{
    let mut reactor = RewriteReactor::new(
        reader,
        version.message_command_style(),
        version.message_fixup_policy(),
        rewriter,
        code_offset,
    );
    react_with(&mut reactor, version);

    let output_size = reactor.output_size().next_multiple_of(16);

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
        // this check requires Seek on the output
        // can we do it without Seek?
        output.stream_position().unwrap(),
        "Written header size does not match the expected size"
    );

    let mut reactor = reactor.into_emit(&mut output);
    react_with(&mut reactor, version);

    // align the file size to 16 bytes
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

fn bindiff_snr(shin_version: ShinVersion, snr1: &[u8], snr2: &[u8]) {
    let code_offset1 = u32::from_le_bytes(snr1[0x20..0x24].try_into().unwrap());
    let code_offset2 = u32::from_le_bytes(snr2[0x20..0x24].try_into().unwrap());

    if code_offset1 != code_offset2 {
        println!(
            "Code offsets do not match: 0x{:08x} vs 0x{:08x}",
            code_offset1, code_offset2
        );
        return;
    }

    let code_offset = code_offset1 as usize;

    // 4..8 is file size, so it will often trivially not match; it's not useful for diagnostics though
    if snr1[8..code_offset] != snr2[8..code_offset] {
        println!("Headers do not match");
        return;
    }

    let reader1 = Reader::new(snr1, code_offset);
    let reader2 = Reader::new(snr2, code_offset);

    // get positions of instructions and offsets so that we can compare them easier
    let mut reactor = LocationPainterReactor::new(reader1);
    react_with(&mut reactor, shin_version);
    let colors1 = reactor.finish();

    let mut reactor = LocationPainterReactor::new(reader2);
    react_with(&mut reactor, shin_version);
    let colors2 = reactor.finish();

    for i in 0..std::cmp::min(colors1.len(), colors2.len()) {
        if colors1[i] != colors2[i] {
            println!(
                "Colors at 0x{:08x} do not match: {:?} vs {:?}",
                i, colors1[i], colors2[i]
            );
            break;
        }
    }

    todo!()
}

impl Command {
    pub fn run(self) {
        let common = match &self {
            Command::Read { common, .. } => common,
            Command::ReadConsole { common, .. } => common,
            Command::ReadValidateOffsets { common, .. } => common,
            Command::Test { common, .. } => common,
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
            Command::Test { common: _ } => {
                // 1. test string roundtrip
                let mut reactor = StringRoundtripValidatorReactor::new(
                    version.message_command_style(),
                    version.message_fixup_policy(),
                    reader.clone(),
                );

                react_with(&mut reactor, version);

                // 2. test SNR roundtrip
                let rewriter = NoopRewriter::new();

                let mut output = std::io::Cursor::new(Vec::new());
                rewrite_snr(
                    &snr_file,
                    reader,
                    code_offset,
                    version,
                    rewriter,
                    &mut output,
                );

                // and, finally, compare the original and the rewritten SNR files
                let output = output.into_inner();

                if snr_file.as_slice() != output.as_slice() {
                    let path = std::env::temp_dir().join("main_rewritten.snr");
                    std::fs::write(&path, &output).expect("Writing the rewritten SNR file failed");

                    println!(
                        "Rewritten SNR does not match the original, written to {}",
                        path.display()
                    );

                    bindiff_snr(version, snr_file.as_slice(), output.as_slice());
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

                let output = File::create(output).expect("Opening the output file failed");
                let mut output = BufWriter::new(output);

                rewrite_snr(
                    &snr_file,
                    reader,
                    code_offset,
                    version,
                    rewriter,
                    &mut output,
                );

                output.flush().unwrap();
            }
        }
    }
}
