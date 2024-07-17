mod cmd;
mod test;

use anyhow::Result;

pub use crate::cmd::cmd::*;

pub trait Run {
    fn run(&self) -> Result<()>;
}

impl Run for Cmd {
    fn run(&self) -> Result<()> {
        match self {
            Cmd::Test(cmd) => cmd.run(),
        }
    }
}
