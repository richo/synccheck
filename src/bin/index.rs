use std::path::PathBuf;

use clap::{Parser};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    path: PathBuf,

    #[arg(short, long)]
    output_file: Option<PathBuf>,

    #[arg(short='x', long)]
    exclude: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // TODO(richo) better ergos
    let cfg = synccheck::WalkerConfig {
        exclude: args.exclude,
    };

    let iter = synccheck::walk(args.path, cfg);

    let mut db = synccheck::Db::default();

    for item in iter  {
        db.insert(item)?;
    }

    match args.output_file {
        None => { // stdout
            db.write_to_file(std::io::stdout())?;
        },
        Some(path) => {
            let fh = std::fs::File::create(path)?;
            db.write_to_file(fh)?;
        }
    }

    Ok(())
}
