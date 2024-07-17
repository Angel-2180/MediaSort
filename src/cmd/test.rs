use anyhow::Result;

use crate::cmd::{Run, Test};

impl Run for Test {
    fn run(&self) -> Result<()> {
        println!("Test: {:?}", self);
        Ok(())
    }
}
