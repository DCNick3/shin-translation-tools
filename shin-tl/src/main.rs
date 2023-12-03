mod ctx;
mod instruction;
mod reactor;
mod reader;

use std::{
    fs::File,
    io::{BufWriter, Seek, SeekFrom, Write},
};

use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
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

#[derive(Parser)]
struct Cli {
    #[clap(value_enum)]
    version: ShinVersion,
    snr_file: Utf8PathBuf,
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Read strings from an SNR file to a CSV file for translation
    Read { output: Utf8PathBuf },
    /// Read strings from an SNR file and dump them to the console
    ReadConsole {},
    /// Read SNR file, while validating jump offsets in the code
    ReadValidateOffsets {},
    /// Rewrite an SNR file to use translated strings from a CSV file
    Rewrite {
        translations: Utf8PathBuf,
        output: Utf8PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    let snr_file = std::fs::read(cli.snr_file).expect("Reading the SNR file failed");
    let version = cli.version;

    assert_eq!(&snr_file[0..4], b"SNR ", "SNR file magic mismatch");
    let code_offset = u32::from_le_bytes(snr_file[0x20..0x24].try_into().unwrap());

    let reader = Reader::new(&snr_file, code_offset as usize);

    match cli.command {
        Command::Read { output } => {
            let writer = csv::Writer::from_path(output).expect("Opening the CSV file failed");

            let mut reactor = StringTraceReactor::new(reader, CsvTraceListener::new(writer));

            react_with(&mut reactor, version);
        }
        Command::ReadConsole {} => {
            let mut reactor = StringTraceReactor::new(reader, ConsoleTraceListener);

            react_with(&mut reactor, version);
        }
        Command::ReadValidateOffsets {} => {
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
    }
}
