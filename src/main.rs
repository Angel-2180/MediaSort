#[allow(unused_imports)]
#[allow(dead_code)]
mod cmd;
mod error;
mod search;

mod episode;

use std::io::{self, Write};
use std::process::ExitCode;

use clap::Parser;

use crate::cmd::{Cmd, Run};
use crate::error::SilentExit;

fn main() -> ExitCode {
    match Cmd::parse().run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => match e.downcast::<SilentExit>() {
            Ok(SilentExit { code }) => code.into(),
            Err(e) => {
                _ = writeln!(io::stderr(), "MediaSort: {e:?}");
                ExitCode::FAILURE
            }
        },
    }
}
