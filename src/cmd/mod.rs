mod cmd;
mod profile;
mod sort;

use anyhow::Result;

pub use crate::cmd::cmd::*;

pub trait Run {
    fn run(&mut self) -> Result<()>;
}

impl Run for Cmd {
    fn run(&mut self) -> Result<()> {
        match self {
            Cmd::Sort(cmd) => cmd.run(),
            Cmd::Profile(cmd) => cmd.run(),
        }
    }
}
