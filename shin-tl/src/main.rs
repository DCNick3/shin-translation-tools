mod rom;
mod snr;

use camino::Utf8PathBuf;
use clap::{Args, CommandFactory, Parser};
use clap_complete::Shell;
use shin_versions::ShinVersion;
use tracing::{info, level_filters::LevelFilter};
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// A multi-tool for translating shin games
///
/// For usage documentation see https://github.com/DCNick3/shin-translation-tools/blob/master/README.md
#[derive(Parser)]
#[clap(author, version)]
enum Command {
    #[clap(subcommand)]
    Snr(snr::Command),
    #[clap(subcommand)]
    Rom(rom::Command),
    /// Generate shell complete script for the given shell
    GenerateCompletion {
        /// The shell to generate the completion for
        #[clap(value_enum)]
        shell: Shell,
    },
}

#[derive(Args, Clone)]
struct CommonArgs {
    #[clap(value_enum)]
    engine_version: ShinVersion,
    snr_file: Utf8PathBuf,
}

fn main() {
    let indicatif_layer = IndicatifLayer::new();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::filter::EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(indicatif_layer.get_stderr_writer()))
        .with(indicatif_layer)
        .init();

    let command = Command::parse();

    match command {
        Command::Snr(action) => action.run(),
        Command::Rom(action) => action.run(),
        Command::GenerateCompletion { shell } => {
            info!("Generating completion file for {:?}...", shell);

            let cmd = &mut Command::command();

            clap_complete::generate(
                shell,
                cmd,
                cmd.get_name().to_string(),
                &mut std::io::stdout(),
            );
        }
    }
}
