use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Args {
    #[clap(help = "files to rename. If none provided, the files list will be read from stdin")]
    pub files: Vec<PathBuf>,

    #[clap(long, help = "Initial regex")]
    pub regex: Option<String>,

    #[clap(long, help = "Initial replacement string")]
    pub replace: Option<String>,

    #[clap(long, action, help = "only print shell commands w/o executing them")]
    pub dry_run: bool,
}

pub fn parse_args() -> Args {
    Args::parse()
}
