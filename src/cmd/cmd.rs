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
    /// Profile name.
    #[clap(short, long, exclusive(true))]
    pub profile: Option<String>,

    /// Input media files.
    #[clap(short, long, value_hint = ValueHint::DirPath)]
    pub input: Option<PathBuf>,

    /// Output directory.
    #[clap(short, long, value_hint = ValueHint::DirPath)]
    pub output: Option<PathBuf>,

    /// Verbose mode.
    #[clap(long)]
    pub verbose: bool,

    /// Maximum number of used threads.
    #[clap(long)]
    pub threads: Option<usize>,

    /// Webhook URL.
    #[clap(long)]
    pub webhook: Option<String>,

    /// Recursive folders scan.
    #[clap(long)]
    pub recursive: bool,

    /// Dry run.
    /// Perform sort but don't actually move any files.
    #[clap(long = "dry-run", short = 'd')]
    pub dry_run: bool,

    /// Overwrite duplicates.
    /// Overwrites duplicates if the new file is larger.
    #[clap(long)]
    pub overwrite: bool,

    /// Overwrite duplicates if the new file is larger.
    #[clap(long = "overwrite-if-larger")]
    pub overwrite_if_larger: bool,

    /// TV series path template.
    /// Default: {Series}/{Name}/{Season}/{Title} - {Episode}.{Extension}
    #[clap(long)]
    pub tv_template: Option<String>,

    /// Movie path template.
    /// Default: {Films}/{Name} ({Year}).{Extension}
    #[clap(long)]
    pub movie_template: Option<String>,

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
    /// Edit a profile.
    Edit(Edit),
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

    /// Profile flags.
    #[clap(short, long)]
    pub flags: Option<Vec<String>>,


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

/// Edit a profile.
#[derive(Clone, Parser, Debug)]
pub struct Edit {
    /// Profile name.
    #[clap(short, long, required(true))]
    pub name: String,

    #[clap(long)]
    pub key: String,

    #[clap(long)]
    pub value: String,
}
