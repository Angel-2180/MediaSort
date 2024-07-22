use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum, ValueHint};

#[derive(Parser, Debug)]
#[clap(about, author, version)]
pub enum Cmd {
    Sort(Sort),
    Profile(Profile),
}

/// Sort input media files into output directories.
#[derive(Parser, Debug)]
#[clap(about, author)]
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
    #[clap(long)]
    pub threads: Option<usize>,

    /// Webhook URL.
    #[clap(long)]
    pub webhook: Option<String>,
}

/// Preset profiles
#[derive(Parser, Debug)]
#[clap(about, author)]
pub struct Profile {
    #[clap(subcommand)]
    pub cmd: Option<ProfileCommand>,
}

#[derive(Clone, Debug, Subcommand)]
pub enum ProfileCommand {
    /// Create a new profile.
    Create(Create),
    /// Delete a profile.
    Delete(Delete),
    /// List all profiles.
    List(List),
}

/// Create a new profile.
#[derive(Clone, Parser, Debug)]
pub struct Create {
    /// Profile name.
    #[clap(short, long, required(true))]
    pub name: String,

    #[clap(short, long, required(true), value_hint = ValueHint::DirPath)]
    pub input: PathBuf,

    #[clap(short, long, required(true), value_hint = ValueHint::DirPath)]
    pub output: PathBuf,

    /// Profile description.
    #[clap(short, long)]
    pub description: Option<String>,
}

/// Delete a profile.
#[derive(Clone, Parser, Debug)]
pub struct Delete {
    /// Profile name.
    #[clap(short, long, required(true))]
    pub name: String,
}

/// List all profiles.
#[derive(Clone, Parser, Debug)]
pub struct List {}
