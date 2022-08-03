use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Args {
    #[clap(required = true)]
    pub files: Vec<PathBuf>,

    #[clap(long, help="Initial replacement regex")]
    pub regex: Option<String>
}

pub fn parse_args() -> Args {
    Args::parse()
}
