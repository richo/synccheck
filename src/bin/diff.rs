use std::path::PathBuf;

use clap::{Parser};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    path: PathBuf,

    #[arg(short, long, default_value_t = 1)]
    output_file: Option<PathBuf>,

}

fn main() -> Result<(), ()> {
    let args = Args::parse();

    let iter = synccheck::walk(args.path);

    let mut db = synccheck::Db::default();

    for item in iter  {
        println!("{:?}", item);
        db.insert(item);
    }

    Ok(())
}
