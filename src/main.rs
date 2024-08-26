#[allow(unused_imports)]
#[allow(dead_code)]
mod cmd;
mod error;
mod search;

mod episode;

use std::io::{self, Write};
use std::process::ExitCode;

use anyhow::Error;
use clap::Parser;

use crate::cmd::{Cmd, Run};
use crate::error::SilentExit;

// -- main code --
// fn main() -> ExitCode {
//     match Cmd::parse().run() {
//         Ok(()) => ExitCode::SUCCESS,
//         Err(e) => match e.downcast::<SilentExit>() {
//             Ok(SilentExit { code }) => code.into(),
//             Err(e) => {
//                 _ = writeln!(io::stderr(), "MediaSort: {e:?}");
//                 return ExitCode::FAILURE;
//             }
//         },
//     };
//     ExitCode::SUCCESS
// }







// -- test code --
fn main() -> Result<(), Error> {
    //     let series = "Mushoku Tensei: Jobless Reincarnation";
    //     let year = "2021";
    //     let media_type = search::result::SERIES.clone();
    //     let results = search::search_tvmaze::search_tvmaze(series, year, media_type).unwrap();
    //     for result in results {
    //         println!("{}", result.string());
    //     }
    let query = "Interstellar";
    let year = Some("");
    let media_type = search::result::MediaType::Movie;

    let debug_mode = true;

    let results = search::search_tmdb::search_movie_db(query, year, media_type, debug_mode)?;

    for result in results {
        if result.accuracy < 95 {
            continue;
        }
        println!("{}, accuracy = {}", result.string(), result.accuracy);
    }

    Ok(())
}