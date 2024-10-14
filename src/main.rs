mod cmd;
mod error;
mod search;
mod episode;
mod subtitle;



// -- main code --
#[cfg(not(test))]
fn main() -> std::process::ExitCode {
    use std::io::{self, Write};
    use std::process::ExitCode;

    use clap::Parser;

    use crate::cmd::{Cmd, Run};
    use crate::error::SilentExit;

    match Cmd::parse().run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => match e.downcast::<SilentExit>() {
            Ok(SilentExit { code }) => code.into(),
            Err(e) => {
                _ = writeln!(io::stderr(), "MediaSort: {e:?}");
                return ExitCode::FAILURE;
            }
        },
    };
    ExitCode::SUCCESS
}


#[cfg(test)]
mod tests {
    use crate::cmd::sort::dry_run_sort;
    use anyhow::Error;
    use episode::Episode;

    use super::*;

    #[test]
    fn test_search_tvmaze() {
        let series = "Breaking Bad";
        let year = "2008";
        let media_type = search::result::MediaType::Series;

        let results = search::search_tvmaze::search_tvmaze(series, Some(year), media_type).unwrap();

        let closest_result = search::result::get_highest_accuracy(results);
        assert_eq!(closest_result.unwrap().string(), "Breaking Bad (2008)");



        let series = "Mushoku Tensei: Jobless Reincarnation";
        let year = None;
        let media_type = search::result::MediaType::Series;

        let results = search::search_tvmaze::search_tvmaze(series, year, media_type).unwrap();
        let closest_result = search::result::get_highest_accuracy(results);
        assert_eq!(closest_result.unwrap().string(), "Mushoku Tensei: Jobless Reincarnation (2021)");

    }

    #[test]
    fn test_search_tmdb_serie() {
        let query = "Breaking Bad";
        let year = Some("2008");
        let media_type = search::result::MediaType::Series;
        let debug_mode = true;

        let results = search::search_tmdb::search_movie_db(query, year, media_type, debug_mode).unwrap();

        let closest_result = search::result::get_highest_accuracy(results);
        assert_eq!(closest_result.unwrap().string(), "Breaking Bad (2008)");

        let query = "Mushoku Tensei: Jobless Reincarnation";
        let year = Some("2021");
        let media_type = search::result::MediaType::Series;
        let debug_mode = true;

        let results = search::search_tmdb::search_movie_db(query, year, media_type, debug_mode).unwrap();
        let closest_result = search::result::get_highest_accuracy(results);

        assert_eq!(closest_result.unwrap().string(), "Mushoku Tensei: Jobless Reincarnation (2021)");

    }

    #[test]
    fn test_search_tmdb_movie() {
        let query = "The Dark Knight";
        let year = Some("2008");
        let media_type = search::result::MediaType::Movie;
        let debug_mode = true;

        let results = search::search_tmdb::search_movie_db(query, year, media_type, debug_mode).unwrap();

        let closest_result = search::result::get_highest_accuracy(results);
        assert_eq!(closest_result.unwrap().string(), "The Dark Knight (2008)");
    }

    #[test]
    fn test_dry_run() -> Result<(), Error> {
        let episodes = create_rand_episode_vector();
        let subtitles = Vec::new();

        dry_run_sort(&episodes, &subtitles, "Series".to_string(), "Film".to_string())?;
        Ok(())
    }


    fn create_rand_episode_vector() -> Vec<Episode> {
        let mut episodes = Vec::new();
        episodes.push(Episode::new_test("Bocchi the Rock - E01 - The Beginning.mkv", false));
        episodes.push(Episode::new_test("DanMachi.S01E02.VOSTFR.1080p.x264.AAC-wawacity.ec.mp4", false));
        episodes.push(Episode::new_test("Speed.Racer.2008.MULTI.VFF.FRforced.HDLight.1080P.x264.AC3.5.1.Wawacity.blue.mkv", true));
        episodes.push(Episode::new_test("The.Dark.Knight.2008.MULTI.VFF.FRforced.HDLight.1080P.x264.AC3.5.1.Wawacity.blue.mkv", true));
        episodes.push(Episode::new_test("The.100.Girlfriends.S01E01.VOSTFR.mkv", false));
        episodes.push(Episode::new_test("The.100.Girlfriends.Who.Really.Really.Really.Really.Really.Love.You.S01E07.VOSTFR.mkv", false));
        episodes.push(Episode::new_test("Youkoso.Jitsuryoku.Shijou.Shugi.no.Kyoushitsu.e.S2.01 VOSTFR.1080p.www.vostfree.tv.mp4", false));
        episodes
    }
}
