mod ctx;
mod instruction;
mod reactor;
mod reader;

use std::{
    fs::File,
    io::{BufWriter, Seek, SeekFrom, Write},
};

use camino::Utf8PathBuf;
use clap::{Args, CommandFactory, Parser};
use clap_complete::Shell;
use shin_versions::ShinVersion;

use crate::{
    ctx::Ctx,
    reactor::{
        offset_validator::OffsetValidatorReactor,
        rewrite::{CsvRewriter, RewriteReactor},
        trace::{ConsoleTraceListener, CsvTraceListener, StringTraceReactor},
        Reactor,
    },
    reader::Reader,
};

fn react_impl<R: Reactor>(ctx: &mut Ctx<R>) {
    while ctx.has_instr() {
        ctx.instr_start();
        let opcode = ctx.byte();
        let Some(instr) = instruction::decode_instr(opcode) else {
            panic!(
                "Unknown opcode 0x{opcode:02x} ({opcode}) @ {}",
                ctx.debug_loc()
            );
        };
        instruction::react_instr(ctx, instr);
        ctx.instr_end();
    }
}

fn react_with<R: Reactor>(reactor: &mut R, version: ShinVersion) {
    let mut ctx = Ctx::new(reactor, version);
    react_impl(&mut ctx);
}

#[derive(Args, Clone)]
struct CommonArgs {
    #[clap(value_enum)]
    engine_version: ShinVersion,
    snr_file: Utf8PathBuf,
}

/// Rewrite shin SNR file with translated strings
///
/// TL;DR:
///
/// 1. shin-tl read <engine-version> <main.snr> <strings.csv>
///
/// 2. Translate strings in strings.csv
///
/// 3. shin-tl rewrite <engine-version> <main.snr> <strings.csv> <main_translated.snr>
///
/// For more usage documentation see https://github.com/DCNick3/shin-translation-tools/blob/master/shin-tl/README.md
#[derive(Parser)]
#[clap(version, author)]
enum Command {
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

    /// Generate shell complete script for the given shell
    GenerateCompletion {
        /// The shell to generate the completion for
        #[clap(value_enum)]
        shell: Shell,
    },
}

fn main() {
    let command = Command::parse();

    let common = match &command {
        &Command::GenerateCompletion { shell } => {
            eprintln!("Generating completion file for {:?}...", shell);

            let cmd = &mut Command::command();

            clap_complete::generate(
                shell,
                cmd,
                cmd.get_name().to_string(),
                &mut std::io::stdout(),
            );

            return;
        }
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

    match command {
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
                    println!("All offsets are valid");
                }
                Err(e) => {
                    println!("{}", e);
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

            let output = File::create(output).expect("Opening the output file failed");
            let mut output = BufWriter::new(output);
            // write the headers that are before the code
            // we don't currently do anything with them
            output
                .write_all(&snr_file[0..code_offset as usize])
                .expect("Writing to the output file failed");

            let mut reactor = RewriteReactor::new(reader, rewriter, code_offset);
            react_with(&mut reactor, version);

            let mut reactor = reactor.into_emit(&mut output);
            react_with(&mut reactor, version);

            // align the file size to 16 bytes
            let mut output = output
                .into_inner()
                .expect("Flushing the output file failed");
            let current_size = output
                .seek(SeekFrom::Current(0))
                .expect("Getting the current file size failed");
            let padding = current_size.next_multiple_of(16) - current_size;
            output
                .write_all(&vec![0; padding as usize])
                .expect("Writing to the output file failed");
        }
        Command::GenerateCompletion { .. } => {
            unreachable!()
        }
    }
}
