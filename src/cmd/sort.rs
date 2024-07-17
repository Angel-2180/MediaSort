use anyhow::Ok;
#[macro_use]
use anyhow::{bail, Result};

use crate::cmd::{Run, Sort};

impl Run for Sort {
    fn run(&self) -> Result<()> {
        // input path is a directory
        if !(self.input.is_dir()) {
            bail!("Input path is not a directory");
        } else if !(self.output.is_dir()) {
            bail!("Output path is not a directory");
        }

        Ok(())
    }
}
