mod cmd;
mod profile;
#[cfg(test)]
pub(crate) mod sort;

//not test
#[cfg(not(test))]
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
