use std::fs;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};

extern crate directories;
use directories::BaseDirs;

use serde_json::{json, Value};

use crate::cmd::{Create, Delete, List, Profile, ProfileCommand, Run, Edit, Flags, Init};


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

pub fn get_profile_by_name(name: &str) -> Result<PathBuf> {
    let profiles_dir = get_or_create_profiles_dir()?;

    let profile_path = profiles_dir.join(format!("{}.pms", name));

    if !profile_path.exists() {
        bail!("Profile not found: {}", name);
    }

    Ok(profile_path)
}

pub fn get_profile_properties(path: &PathBuf) -> Result<(String,String, serde_json::Map<String, Value>)> {
    let profile_str = fs::read_to_string(path)?;

    let profile: Value = serde_json::from_str(&profile_str)?;

    let input = profile["input"].as_str().context("Profile has no input")?;
    let output = profile["output"].as_str().context("Profile has no output")?;
    let mut flags: serde_json::Map<String, Value> = profile["flags"].as_object().context("Profile has no flags")?.clone();

    check_or_add_all_flag(&mut flags);

    Ok((input.to_string(), output.to_string(), flags))
}

fn check_or_add_all_flag(flags: &mut serde_json::Map<String, Value>)
{
    if !flags.contains_key("verbose") {
        flags.insert("verbose".to_string(), serde_json::Value::Bool(false));
    }
    if !flags.contains_key("recursive") {
        flags.insert("recursive".to_string(), serde_json::Value::Bool(false));
    }
    if !flags.contains_key("threads") {
        let num_cpus: usize = num_cpus::get() - 2;
        flags.insert("threads".to_string(), serde_json::Value::Number(serde_json::Number::from(num_cpus)));
    }
    if !flags.contains_key("webhook") {
        flags.insert("webhook".to_string(), serde_json::Value::String("default".to_string()));
    }
    if !flags.contains_key("dry-run") {
        flags.insert("dry-run".to_string(), serde_json::Value::Bool(false));
    }
    if !flags.contains_key("tv-template") {
        flags.insert("tv-template".to_string(), serde_json::Value::String("Series".to_string()));
    }
    if !flags.contains_key("movie-template") {
        flags.insert("movie-template".to_string(), serde_json::Value::String("Films".to_string()));
    }
    if !flags.contains_key("search") {
        flags.insert("search".to_string(), serde_json::Value::Bool(false));
    }
    if !flags.contains_key("skip-subtitles") {
        flags.insert("skip-subtitles".to_string(), serde_json::Value::Bool(false));
    }
}

fn get_default_flags() -> serde_json::Map<String, serde_json::Value> {
    let mut flags = serde_json::Map::new();
    flags.insert("verbose".to_string(), serde_json::Value::Bool(false));
    flags.insert("recursive".to_string(), serde_json::Value::Bool(false));
    let num_cpus: usize = num_cpus::get() - 2;
    flags.insert("threads".to_string(), serde_json::Value::Number(serde_json::Number::from(num_cpus)));
    flags.insert("webhook".to_string(), serde_json::Value::String("default".to_string()));
    flags.insert("dry-run".to_string(), serde_json::Value::Bool(false));
    flags.insert("tv-template".to_string(), serde_json::Value::String("Series".to_string()));
    flags.insert("movie-template".to_string(), serde_json::Value::String("Films".to_string()));
    flags.insert("search".to_string(), serde_json::Value::Bool(false));
    flags.insert("skip-subtitles".to_string(), serde_json::Value::Bool(false));

    flags
}

impl Run for Profile {
    fn run(&mut self) -> Result<()> {
        let cmd = self.cmd.as_mut().context("No subcommand provided")?;

        cmd.run()?;

        Ok(())
    }
}

impl Run for ProfileCommand {
    fn run(&mut self) -> Result<()> {
        match self {
            ProfileCommand::Create(cmd) => cmd.run(),
            ProfileCommand::Delete(cmd) => cmd.run(),
            ProfileCommand::List(cmd) => cmd.run(),
            ProfileCommand::Edit(cmd) => cmd.run(),
            ProfileCommand::Flags(cmd) => cmd.run(),
            ProfileCommand::Init(cmd) => cmd.run(),
        }
    }
}

impl Run for Create {
    fn run(&mut self) -> Result<()> {
        let profiles_dir = get_or_create_profiles_dir()?;

        let profile_path = profiles_dir.join(format!("{}.pms", self.name));

        if profile_path.exists() {
            bail!("Profile with name {} already exists", self.name);
        }

        let mut flags = get_default_flags();

        if let Some(cmd_flags) = &self.flags {
            for flag in cmd_flags {

                let mut parts = flag.splitn(2, '=');
                let key = parts.next().context("Flag has no key")?.to_string();
                let value = parts.next().context("Flag has no value")?.to_string();
                 // Attempt to parse the value as a bool or number, fallback to string
                 if let Ok(bool_value) = value.parse::<bool>() {
                    flags.insert(key, serde_json::Value::Bool(bool_value));
                } else if let Ok(number_value) = value.parse::<i64>() {
                    flags.insert(key, serde_json::Value::Number(serde_json::Number::from(number_value)));
                } else {
                    flags.insert(key, serde_json::Value::String(value));
                }
            }
        }

        let profile = json!({
          "name": self.name,
          "input": self.input,
          "output": self.output,
          "flags": flags,
        });

        let profile_str = serde_json::to_string_pretty(&profile)?;

        fs::write(&profile_path, profile_str)?;

        println!("Profile {:?} successfully created", self.name);

        Ok(())
    }
}

impl Run for Delete {
    fn run(&mut self) -> Result<()> {
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
    fn run(&mut self) -> Result<()> {
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


impl Edit {
    pub fn run(&mut self) -> Result<()> {
        let profile_path = get_profile_by_name(&self.name)?;

        let mut profile_str = fs::read_to_string(&profile_path)?;

        let mut profile: Value = serde_json::from_str(&profile_str)?;

        if self.key == "flags" && self.value == "reset" {
            profile["flags"] = Value::Object(get_default_flags());
            profile_str = serde_json::to_string_pretty(&profile)?;
            fs::write(&profile_path, profile_str)?;

            println!("Profile {:?} flags successfully reset", self.name);

            return Ok(());
        }

        if self.key == "flags" {

            let mut flags = profile["flags"].as_object().context("Profile has no flags")?.clone();

            let mut parts = self.value.splitn(2, '=');
            let key = parts.next().context("Flag has no key")?.to_string();
            let value = parts.next().context("Flag has no value")?.to_string();

            // Attempt to parse the value as a bool or number, fallback to string
            if let Ok(bool_value) = value.parse::<bool>() {
                flags.insert(key, serde_json::Value::Bool(bool_value));
            } else if let Ok(number_value) = value.parse::<i64>() {
                flags.insert(key, serde_json::Value::Number(serde_json::Number::from(number_value)));
            } else {
                flags.insert(key, serde_json::Value::String(value));
            }

            profile["flags"] = Value::Object(flags);

            profile_str = serde_json::to_string_pretty(&profile)?;

            fs::write(&profile_path, profile_str)?;

            println!("Profile {:?} successfully edited", self.name);

            return Ok(());
        }

        profile[self.key.clone()] = Value::String(self.value.clone());

        profile_str = serde_json::to_string_pretty(&profile)?;

        fs::write(&profile_path, profile_str)?;

        println!("Profile {:?} successfully edited", self.name);

        Ok(())
    }

}


impl Flags {
    pub fn run(&mut self) -> Result<()> {
        let profile_path = get_profile_by_name(&self.name)?;

        let profile_str = fs::read_to_string(&profile_path)?;

        let profile: Value = serde_json::from_str(&profile_str)?;

        let flags = profile["flags"].as_object().context("Profile has no flags")?;

        for (key, value) in flags {
            println!("{}: {}", key, value);
        }

        Ok(())
    }
}

impl Run for Init {

    fn run(&mut self) -> Result<()> {
        let base_dirs = BaseDirs::new().unwrap();
        let dir_path = base_dirs.data_local_dir().join("MediaSort");
        if !dir_path.exists() {
            fs::create_dir_all(&dir_path)?;
        }
        let file_path = dir_path.join("unwanted_words.txt");
        if !file_path.exists() {
            fs::write(&file_path, "net\nfit\nws\ntv\nTV\nec\nco\nvip\ncc\ncfd\nred\nNanDesuKa\nFANSUB\ntokyo\nWEBRip\nDL\nH264\nLight\ncom\norg\ninfo\nwww\ncom\nvostfree\nVOSTFR\nboats\nuno\nWawacity\nwawacity\nWEB\nTsundereRaws\n1080p\n720p\nx264\nAAC\nTsundere\nRaws\nfit\nws\ntv\nTV\nec\n")?;
        }
        Ok(())

    }
}