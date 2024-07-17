use core::num;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::Ok;
use anyhow::{bail, Result};

use rayon::{prelude::*, ThreadPoolBuilder};

use crate::cmd::{Run, Sort};
use crate::episode::Episode;

impl Run for Sort {
    fn run(&self) -> Result<()> {
        // input path is a directory
        if !(self.input.is_dir()) {
            bail!("Input path is not a directory");
        } else if !(self.output.is_dir()) {
            bail!("Output path is not a directory");
        }

        // TODO: make default to false. true by default for dev purpose
        if self.multi_threaded.unwrap_or(true) {
            self.sort_medias_threaded()?;
        }

        Ok(())
    }
}

impl Sort {
    fn get_medias_from_input(&self) -> Result<Vec<Episode>> {
        let input_path = self.input.clone();
        let paths: fs::ReadDir = fs::read_dir(input_path).unwrap();
        let mut episodes: Vec<Episode> = Vec::new();

        for path in paths {
            let path: PathBuf = path.unwrap().path();

            if self.is_media(&path) {
                let episode: Episode = Episode::new(&path);
                episodes.push(episode);
            }
        }

        if episodes.is_empty() {
            bail!("No media files found in the input directory");
        }

        if self.verbose.unwrap_or(false) {
            println!("Found {} media files", episodes.len());
        }

        Ok(episodes)
    }

    fn is_media(&self, path: &PathBuf) -> bool {
        let ext = path.extension().unwrap();
        let ext_str = ext.to_str().unwrap();

        match ext_str {
            "mp4" | "mkv" | "avi" | "mov" | "flv" | "wmv" | "webm" => true,
            _ => false,
        }
    }

    fn sort_medias_threaded(&self) -> Result<()> {
        if self.verbose.unwrap_or(false) {
            println!("Getting medias in {:?}", self.input);
        }

        let mut episodes: Vec<Episode> = self.get_medias_from_input()?;
        // let dir_set: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));

        let max_cpu_count: usize = num_cpus::get() - 1;
        let mut num_threads: usize = self.threads.unwrap_or(max_cpu_count);

        if num_threads > max_cpu_count {
            num_threads = max_cpu_count;

            println!(
                "Number of threads is greater than the number of CPUs. Using {} threads",
                num_threads
            );
        }

        if num_threads == 0 {
            bail!("Number of threads must be greater than 0");
        }

        // Configure global thread pool
        ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build_global()
            .unwrap();

        episodes.par_iter_mut().for_each(|_episode| {
            // TODO: move files in parallel
        });

        Ok(())
    }
}
