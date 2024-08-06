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
    if std::env::var("PROFILE").unwrap() == "release" {
        return match Cmd::parse().run() {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => match e.downcast::<SilentExit>() {
                Ok(SilentExit { code }) => code.into(),
                Err(e) => {
                    _ = writeln!(io::stderr(), "MediaSort: {e:?}");
                    return ExitCode::FAILURE;
                }
            },
        };
    }
    else {

        let series = "Mushoku Tensei: Jobless Reincarnation";
        let year = "2021";
        let media_type = search::result::SERIES.clone();
        let results = search::search_tvmaze::search_tvmaze(series, year, media_type).unwrap();
        for result in results {
            println!("{}", result.string());
        }
    }
    ExitCode::SUCCESS
}

fn main() {

}
