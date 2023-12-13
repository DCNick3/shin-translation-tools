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

use clap::Parser;
use tracing::level_filters::LevelFilter;
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
enum Command {
    Extract(actions::Extract),
    Create(actions::Create),
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
    }
}
