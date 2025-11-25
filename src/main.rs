use clap::Parser;
use shellshot::{run_shellshot, Args};

fn main() {
    let args = Args::parse();
    if let Err(e) = run_shellshot(args) {
        eprintln!("Error while running shellshot: {e}");
    }
}
