use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum, ValueHint};

#[derive(Parser, Debug)]
#[clap(about, author, version)]
pub enum Cmd {
    Test(Test),
    Sort(Sort),
}

/// Test subcommand.
#[derive(Parser, Debug)]
#[clap(author)]
pub struct Test {
    pub temp: String,
}

/// Sort input media files into output directories.
#[derive(Parser, Debug)]
#[clap(about, author, version)]
pub struct Sort {
    /// Input media files.
    #[clap(short, long, value_hint = ValueHint::DirPath)]
    pub input: PathBuf,
    /// Output directory.
    #[clap(short, long, value_hint = ValueHint::DirPath)]
    pub output: PathBuf,
}
