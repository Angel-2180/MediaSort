use std::fs;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};

extern crate directories;
use directories::BaseDirs;

use serde_json::{json, Deserializer, Serializer};

use crate::cmd::{Create, Delete, List, Profile, ProfileCommand, Run};

fn get_or_create_profiles_dir() -> Result<PathBuf> {
    let base_dirs = BaseDirs::new().context("Could not get base directories")?;

    let profiles_dir = base_dirs
        .data_local_dir()
        .join("MediaSort")
        .join("profiles");

    if !profiles_dir.try_exists()? {
        fs::create_dir_all(&profiles_dir).context("Could not create profiles directory")?;
    }

    Ok(profiles_dir)
}

fn get_profile_by_name(name: &str) -> Result<PathBuf> {
    let profiles_dir = get_or_create_profiles_dir()?;

    let profile_path = profiles_dir.join(format!("{}.pms", name));

    if !profile_path.exists() {
        bail!("Profile not found: {}", name);
    }

    Ok(profile_path)
}

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
        let profiles_dir = get_or_create_profiles_dir()?;

        let profile_path = profiles_dir.join(format!("{}.pms", self.name));

        if profile_path.exists() {
            bail!("Profile with name {} already exists", self.name);
        }

        let profile = json!({
          "name": self.name,
          "input": self.input,
          "output": self.output,
        });

        let profile_str = serde_json::to_string_pretty(&profile)?;

        fs::write(&profile_path, profile_str)?;

        println!("Profile {:?} successfully created", self.name);

        Ok(())
    }
}

impl Run for Delete {
    fn run(&self) -> Result<()> {
        let profile_path = get_profile_by_name(&self.name)?;

        println!(
            "Are you sure you want to delete the profile {:?}? (y/n)",
            self.name
        );
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        match input.trim() {
            "y" | "Y" => {
                fs::remove_file(profile_path)?;
                println!("Profile {:?} successfully deleted", self.name);
            }
            "n" | "N" => {
                println!("Abort deletion");
            }
            _ => {
                print!("\x1b[1A\x1b[2K"); // clear the line
                self.run()?;
            }
        }

        Ok(())
    }
}

impl Run for List {
    fn run(&self) -> Result<()> {
        let profiles_dir = get_or_create_profiles_dir()?;

        let profiles = fs::read_dir(profiles_dir)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()?.to_str()? == "pms" {
                    Some(path.file_stem()?.to_str()?.to_string())
                } else {
                    None
                }
            })
            .collect::<Vec<String>>();

        if profiles.is_empty() {
            println!("No profiles found!");
        } else {
            println!("Profiles:");
            for profile in profiles {
                println!("  - {}", profile);
            }
        }

        Ok(())
    }
}
