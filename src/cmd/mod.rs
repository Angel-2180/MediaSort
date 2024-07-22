mod cmd;
mod profile;
mod sort;

use anyhow::Result;

pub use crate::cmd::cmd::*;

pub trait Run {
    fn run(&self) -> Result<()>;
}

impl Run for Cmd {
    fn run(&self) -> Result<()> {
        match self {
            Cmd::Sort(cmd) => cmd.run(),
            Cmd::Profile(cmd) => cmd.run(),
        }
    }
}
