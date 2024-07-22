use crate::cmd::{Create, Delete, List, Profile, ProfileCommand, Run};

use anyhow::Result;

impl Run for Profile {
    fn run(&self) -> Result<()> {
        self.cmd.as_ref().unwrap().run()?;

        Ok(())
    }
}

impl Run for ProfileCommand {
    fn run(&self) -> Result<()> {
        match self {
            ProfileCommand::Create(cmd) => cmd.run(),
            ProfileCommand::Delete(cmd) => cmd.run(),
            ProfileCommand::List(cmd) => cmd.run(),
        }
    }
}

impl Run for Create {
    fn run(&self) -> Result<()> {
        println!("Creating profile {:?}", self.name);

        Ok(())
    }
}

impl Run for Delete {
    fn run(&self) -> Result<()> {
        println!("Deleting profile {:?}", self.name);

        Ok(())
    }
}

impl Run for List {
    fn run(&self) -> Result<()> {
        println!("Listing profiles");

        Ok(())
    }
}
