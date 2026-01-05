use std::{
    fs::File,
    io::{BufWriter, Read, Write},
};

use camino::Utf8PathBuf;
use clap::{Args, Subcommand};
use shin_font::FontMetrics;
use shin_snr::{
    layout::{layouter::GameLayoutInfo, message_parser::MessageReflowMode},
    location_painter,
    operation::schema::{ENGINE_SCHEMAS, EngineSchema},
    reactor::{
        dump_bin::DumpBinReactor,
        offset_validator::OffsetValidatorReactor,
        react_with,
        rewrite::{
            CsvData, CsvRewriter, NoopRewriter, RewriteReactor, StringReplacementMode,
            StringRewriter,
        },
        string_roundrip_validator::StringRoundtripValidatorReactor,
        trace::{ConsoleTraceListener, CsvTraceListener, StringTraceReactor},
    },
    reader::Reader,
};
use shin_versions::{MessageCommandStyle, ShinVersion};
use tracing::{error, info};

#[derive(Args, Clone)]
pub struct CommonArgs {
    /// Version of the engine the SNR file is from
    #[clap(value_enum)]
    engine_version: ShinVersion,
    /// Path to the SNR file
    snr_file: Utf8PathBuf,
}

/// Determines the policy on transforming message commands style
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, clap::ValueEnum)]
pub enum MessageStylePolicy {
    /// Keep the message style that is native to the engine. For engines with unescaped commands, they will be left as-is (`c666.`)
    #[default]
    Keep,
    /// Transform the message style to use the modern escaped style (commands like `@c666.`)
    Modernize,
}

impl MessageStylePolicy {
    pub fn apply(&self, native_style: MessageCommandStyle) -> MessageCommandStyle {
        match self {
            MessageStylePolicy::Keep => native_style,
            MessageStylePolicy::Modernize => MessageCommandStyle::Escaped,
        }
    }
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, clap::ValueEnum)]
pub enum CliMessageReflowMode {
    /// Do not attempt to reflow text to introduce additional line breaks.
    #[default]
    NoReflow,
    /// Use a greedy algorithm to insert hardbreaks (@r) to correctly word-wrap western text.
    ///
    /// This uses UAX#14 to identify possible line break points.
    Greedy,
}

impl CliMessageReflowMode {
    pub fn materialize(
        self,
        shin_version: ShinVersion,
        font: Option<&FontMetrics>,
    ) -> MessageReflowMode<'_> {
        match self {
            CliMessageReflowMode::NoReflow => {
                MessageReflowMode::NoReflow
            }
            CliMessageReflowMode::Greedy => {
                MessageReflowMode::Greedy {
                    metrics: &font.expect("Message reflowing requires a corresponding font file (supplied with --font-file option)"),
                    layout: GameLayoutInfo::for_version(shin_version).expect("This engine version doesn't support text reflowing yet"),
                }
            }
        }
    }
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
/// For more usage documentation see https://github.com/DCNick3/shin-translation-tools/blob/master/README.md
#[derive(Subcommand)]
pub enum Command {
    /// Read strings from an SNR file to a CSV file for translation
    Read {
        #[clap(flatten)]
        common: CommonArgs,
        /// Change the way message commands are transformed
        ///
        /// NOTE: make sure that the same value of this option is used in `shin-tl snr rewrite`
        #[clap(long, value_enum, default_value_t)]
        message_style: MessageStylePolicy,
        /// Path to the output CSV file
        output: Utf8PathBuf,
    },
    /// Read strings from an SNR file and dump them to the console
    ReadConsole {
        #[clap(flatten)]
        common: CommonArgs,
        /// Change the way message commands are transformed
        #[clap(long, value_enum, default_value_t)]
        message_style: MessageStylePolicy,
    },
    /// Read strings from an SNR file and dump messages (MSGSET and LOGSET) to a bin file, separated by null terminators.
    ///
    /// The messages are **not** passed through the Shift-JIS decoder, outputting them as-is. Useful to feed the data to other tools in a simpler format.
    ReadToBin {
        #[clap(flatten)]
        common: CommonArgs,
        /// Path to the output BIN file
        output: Utf8PathBuf,
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
        font_file: Option<Utf8PathBuf>,
        layout_dump_file: Option<Utf8PathBuf>,
    },
    /// Rewrite an SNR file to use translated strings from a CSV file
    Rewrite {
        #[clap(flatten)]
        common: CommonArgs,
        #[clap(long, value_enum, default_value_t)]
        /// Change the way message commands are transformed
        ///
        /// NOTE: make sure that the same value of this option is used in `shin-tl snr read`
        message_style: MessageStylePolicy,
        /// Don't try to detect issues with the CSV file before rewriting
        #[clap(long)]
        no_lint: bool,
        /// Controls which columns from CSV file are used to replace strings
        #[clap(long, value_enum, default_value_t)]
        replacement_mode: StringReplacementMode,
        /// Reflow the text to insert line breaks for line wrapping
        #[clap(long, value_enum, default_value_t)]
        reflow_mode: CliMessageReflowMode,
        /// Path to the font file for --reflow-mode option
        #[clap(long, value_enum)]
        font_file: Option<Utf8PathBuf>,
        /// Path to the CSV file with translations
        ///
        /// A template can be created with `shin-tl snr read`
        translations: Utf8PathBuf,
        /// Path to the output SNR file
        output: Utf8PathBuf,
    },
}

fn rewrite_snr<'a, R, O>(
    snr_file: &[u8],
    reader: Reader,
    code_offset: u32,
    schema: &EngineSchema,
    version: ShinVersion,
    user_style: MessageCommandStyle,
    reflow_mode: MessageReflowMode<'a>,
    rewriter: R,
    output: &mut O,
) where
    R: StringRewriter,
    O: Write,
{
    let mut reactor = RewriteReactor::new(
        version.number_style(),
        version.message_command_style(),
        user_style,
        reflow_mode,
        version.string_policy(),
        rewriter,
        code_offset,
    );
    react_with(reader.clone(), schema, &mut reactor);

    let output_size = reactor.output_size();

    assert!(output_size.is_multiple_of(16));

    let mut output_buffer = Vec::new();

    // copy the magic
    output_buffer
        .write_all(&snr_file[0..4])
        .expect("Writing to the output file failed");
    // re-write with the correct file sizes
    // even though some engine versions do not care about this fields, some absolutely do!
    // (for example, DC4)
    output_buffer
        .write_all(&output_size.to_le_bytes())
        .expect("Writing to the output file failed");
    // copy the rest of the header
    output_buffer
        .write_all(&snr_file[8..code_offset as usize])
        .expect("Writing to the output file failed");

    assert_eq!(
        code_offset,
        // this check requires Seek on the output
        // can we do it without Seek?
        output_buffer.len() as u32,
        "Written header size does not match the expected size"
    );

    let mut reactor = reactor.into_emit(output_buffer);
    react_with(reader, schema, &mut reactor);

    let output_buffer = reactor.finish();

    assert_eq!(
        output_size,
        output_buffer.len() as u32,
        "Output file size does not match the expected size"
    );

    output
        .write_all(&output_buffer)
        .expect("Writing to the output file failed")
}

fn bindiff_snr(schema: &EngineSchema, snr1: &[u8], snr2: &[u8]) {
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
    let colors1 = location_painter::paint_locations(reader1, schema);
    let colors2 = location_painter::paint_locations(reader2, schema);

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
            Command::ReadToBin { common, .. } => common,
            Command::ReadValidateOffsets { common, .. } => common,
            Command::Test { common, .. } => common,
            Command::Rewrite { common, .. } => common,
        };

        let snr_file = std::fs::read(&common.snr_file).expect("Reading the SNR file failed");
        let version = common.engine_version;

        let schema = &ENGINE_SCHEMAS[version];

        assert_eq!(&snr_file[0..4], b"SNR ", "SNR file magic mismatch");
        let code_offset = u32::from_le_bytes(snr_file[0x20..0x24].try_into().unwrap());

        let reader = Reader::new(&snr_file, code_offset as usize);

        match self {
            Command::Read {
                common: _,
                message_style,
                output,
            } => {
                let writer = csv::Writer::from_path(output).expect("Opening the CSV file failed");

                let encoding = version.string_encoding();
                let snr_style = version.message_command_style();
                let user_style = message_style.apply(snr_style);

                let mut reactor = StringTraceReactor::new(
                    encoding,
                    snr_style,
                    user_style,
                    CsvTraceListener::new(writer),
                );

                react_with(reader, schema, &mut reactor);
            }
            Command::ReadConsole {
                common: _,
                message_style,
            } => {
                let encoding = version.string_encoding();
                let snr_style = version.message_command_style();
                let user_style = message_style.apply(snr_style);

                let mut reactor =
                    StringTraceReactor::new(encoding, snr_style, user_style, ConsoleTraceListener);

                react_with(reader, schema, &mut reactor);
            }
            Command::ReadToBin { common: _, output } => {
                let mut output = File::create(&output).expect("Opening the BIN file failed");

                let mut reactor = DumpBinReactor::new(&mut output);

                react_with(reader, schema, &mut reactor);
            }
            Command::ReadValidateOffsets { common: _ } => {
                let mut reactor = OffsetValidatorReactor::new();

                react_with(reader, schema, &mut reactor);

                match reactor.validate() {
                    Ok(_) => {
                        info!("All offsets are valid");
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
            Command::Test {
                common: _,
                font_file,
                layout_dump_file,
            } => {
                // test roundtrips through two command styles
                // this also tests a case that we currently do not expose to the user: converting an escaped SNR into an unescaped CSV
                // but it doesn't hurt to test it
                let snr_style = version.message_command_style();
                for user_style in [MessageCommandStyle::Unescaped, MessageCommandStyle::Escaped] {
                    // 1. test string roundtrip
                    let mut reactor = StringRoundtripValidatorReactor::new(
                        snr_style,
                        user_style,
                        version.string_policy(),
                    );

                    react_with(reader.clone(), schema, &mut reactor);

                    // 2. test SNR roundtrip
                    // NOTE: we have to pass in out_style to both NoopRewriter and RewriteReactor
                    // this is a bit ugly, but currently necessary
                    // because the RewriteReactor only passes the untransformed string to the rewriter
                    // (it would be wasteful to do otherwise)
                    // while it expects a transformed string back (because that's what the user supplies in CSV)
                    // maybe there would be a better way to handle this, but idk...
                    let rewriter = NoopRewriter::new(snr_style, user_style);

                    let mut output = std::io::Cursor::new(Vec::new());
                    rewrite_snr(
                        &snr_file,
                        reader.clone(),
                        code_offset,
                        schema,
                        version,
                        user_style,
                        MessageReflowMode::NoReflow,
                        rewriter,
                        &mut output,
                    );

                    // and, finally, compare the original and the rewritten SNR files
                    let output = output.into_inner();

                    if snr_file.as_slice() != output.as_slice() {
                        let path = std::env::temp_dir().join("main_rewritten.snr");
                        std::fs::write(&path, &output)
                            .expect("Writing the rewritten SNR file failed");

                        println!(
                            "Rewritten SNR does not match the original, written to {}",
                            path.display()
                        );

                        bindiff_snr(schema, snr_file.as_slice(), output.as_slice());
                    }
                }

                // test the layouter against the provided layout dump (if present)
                if let Some(font_file) = font_file
                    && let Some(layout_dump_file) = layout_dump_file
                {
                    let mut decoder = ruzstd::decoding::StreamingDecoder::new(
                        File::open(&layout_dump_file).expect("Failed to read layout dump file"),
                    )
                    .expect("Failed to create streaming zstd decoder");
                    let mut layout_dump_file = Vec::new();
                    decoder
                        .read_to_end(&mut layout_dump_file)
                        .expect("Failed to decompress the layout dump");

                    let layout_dump = shin_snr::layout::layout_dump::parse_dump(&layout_dump_file);
                    drop(layout_dump_file);

                    let mut font = File::open(&font_file).expect("Failed to open font file");
                    let font = shin_font::FontMetrics::from_font0(&mut font)
                        .expect("Failed to parse font");

                    shin_snr::layout::layouter::validate_light_layouter_against_dump(
                        &font,
                        &layout_dump,
                    );
                }
            }
            Command::Rewrite {
                common: _,
                message_style,
                no_lint,
                replacement_mode,
                reflow_mode,
                font_file,
                translations,
                output,
            } => {
                let snr_style = version.message_command_style();
                let user_style = message_style.apply(snr_style);

                let font_file = font_file.map(|path| {
                    let mut font_file = File::open(&path).expect("Opening the font file failed");
                    // TODO: support font formats other than font0
                    FontMetrics::from_font0(&mut font_file).expect("Failed to read font file")
                });

                let reflow_mode = reflow_mode.materialize(version, font_file.as_ref());

                let translations =
                    csv::Reader::from_path(translations).expect("Opening the CSV file failed");
                let data = CsvData::new(translations);
                if !no_lint {
                    if let Err(e) = data.lint(replacement_mode, user_style) {
                        println!("There are some issues with strings in the provided CSV file");

                        for report in e {
                            let report = miette::Report::from(report);

                            println!("{:?}", report);
                        }
                        println!("NOTE: You can disable linting by passing --no-lint");
                        std::process::exit(1);
                    }
                }

                let rewriter = CsvRewriter::new(data, replacement_mode);

                let output = File::create(output).expect("Opening the output file failed");
                let mut output = BufWriter::new(output);

                rewrite_snr(
                    &snr_file,
                    reader,
                    code_offset,
                    schema,
                    version,
                    user_style,
                    reflow_mode,
                    rewriter,
                    &mut output,
                );

                output.flush().unwrap();
            }
        }
    }
}
