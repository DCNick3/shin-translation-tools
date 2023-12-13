#[cfg(not(target_pointer_width = "64"))]
// this limitation is due to the use of `usize` for offsets
// and memory-mapping the entire rom, which can be larger than 2GB
compile_error!("shin-rom only supports 64-bit targets");

mod actions;
mod create;
mod header;
mod index;
mod progress;
mod version;

use clap::{CommandFactory, Parser};
use clap_complete::Shell;
use tracing::{info, level_filters::LevelFilter};
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// A tool to extract an package .rom files used by shin-based games
///
/// For more usage documentation see https://github.com/DCNick3/shin-translation-tools
#[derive(Parser)]
#[clap(version, author)]
enum Command {
    /// Extract all files from a rom.
    Extract(actions::Extract),
    /// Create a rom from a directory.
    Create(actions::Create),

    /// Generate shell complete script for the given shell
    GenerateCompletion {
        /// The shell to generate the completion for
        #[clap(value_enum)]
        shell: Shell,
    },
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

    match Command::parse() {
        Command::Extract(action) => action.run(),
        Command::Create(action) => action.run(),
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
