use clap::{Parser, Subcommand, ValueEnum, ValueHint};

#[derive(Parser, Debug)]
#[clap(about, author, version)]
pub enum Cmd {
    Test(Test),
}

/// Test subcommand.
#[derive(Parser, Debug)]
#[clap(author)]
pub struct Test {
    pub temp: String,
}
