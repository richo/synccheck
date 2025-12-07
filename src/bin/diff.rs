use std::path::PathBuf;

use clap::{Parser};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    path: PathBuf,
}

fn main() -> Result<(), ()> {
    let args = Args::parse();

    let iter = synccheck::walk(args.path);

    for item in iter  {
        println!("{:?}", item);
    }

    Ok(())
}
