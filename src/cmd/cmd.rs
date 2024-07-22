use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum, ValueHint};

#[derive(Parser, Debug)]
#[clap(about, author, version)]
pub enum Cmd {
    Sort(Sort),
}

/// Sort input media files into output directories.
#[derive(Parser, Debug)]
#[clap(about, author, version)]
pub struct Sort {
    /// Input media files.
    #[clap(short, long, required(true), value_hint = ValueHint::DirPath)]
    pub input: PathBuf,
    /// Output directory.
    #[clap(short, long, required(true), value_hint = ValueHint::DirPath)]
    pub output: PathBuf,

    /// Verbose mode.
    #[clap(long)]
    pub verbose: bool,

    /// Maximum number of used threads.
    #[clap(short, long)]
    pub threads: Option<usize>,

    /// Webhook URL.
    #[clap(short, long)]
    pub webhook: Option<String>,
}
