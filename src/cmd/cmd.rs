use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueHint};

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
    #[clap(long, action)]
    pub verbose: bool,

    /// Maximum number of used threads.
    #[clap(long)]
    pub threads: Option<usize>,

    /// Webhook URL.
    #[clap(long)]
    pub webhook: Option<String>,

    /// Recursive folders scan.
    #[arg(long, default_value = "false")]
    pub recursive: bool,

    /// Dry run.
    /// Perform sort but don't actually move any files.
    #[clap(long = "dry-run", short = 'd')]
    pub dry_run: bool,

    /// TV series path template.
    /// Default: {Series}/{Name}/{Season}/{Title} - {Episode}.{Extension}
    #[arg(long, default_value = "Series")]
    pub tv_template: Option<String>,

    /// Movie path template.
    /// Default: {Films}/{Name} ({Year}).{Extension}
    #[arg(long, default_value = "Films")]
    pub movie_template: Option<String>,

    /// Search Database
    /// Search for the media in the database of TVMaze and TheMovieDB
    /// and return the best result. (default: false)
    #[clap(long, action)]
    pub search: bool,

    /// Skip Subtitles
    /// Skip the search for subtitles. (default: false)
    /// If the subtitles are disabled, this option will be ignored.
    #[clap(long, action)]
    pub skip_subtitles: bool,
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

    ///List flags of a profile.
    Flags(Flags),

    /// Init default bad keyword
    Init(Init),
}

/// Init default bad keyword
#[derive(Clone, Parser, Debug)]
pub struct Init {}

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

/// List flags of a profile.
#[derive(Clone, Parser, Debug)]
pub struct Flags {
    /// Profile name.
    #[clap(short, long, required(true))]
    pub name: String,
}
