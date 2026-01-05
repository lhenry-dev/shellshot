use clap::Parser;
use shellshot::{Args, run_shellshot};
use tracing::warn;

fn main() {
    let args = Args::parse();

    if !args.quiet {
        tracing_subscriber::fmt()
            .without_time()
            .with_target(false)
            .init();
    }

    if let Err(e) = run_shellshot(args) {
        warn!("Error while running shellshot: {e}");
    }
}
