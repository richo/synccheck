use std::path::PathBuf;

use clap::{Parser};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    from: PathBuf,
    #[arg(long)]
    to: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let from = std::fs::File::open(args.from)?;
    let from_db = synccheck::Db::read_from_file(from)?;

    let r#to = std::fs::File::open(args.to)?;
    let to_db = synccheck::Db::read_from_file(r#to)?;

    let diffs = to_db.diffs_from(&from_db)?;

    println!("Results");
    println!("=======");

    println!("Missing:");
    println!("--------");
    for missing in diffs.missing() {
        println!("  {:?}", missing.relative_path);
    }

    println!("Mismatched Size:");
    println!("----------------");
    for mismatched in diffs.mismatched_size() {
        println!("  {:?}", mismatched.relative_path);
    }

    Ok(())
}
