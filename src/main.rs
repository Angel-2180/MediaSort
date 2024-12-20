mod cmd;
mod episode;
mod error;
mod search;
mod subtitle;

mod tui;

use crate::tui::app::*;
use crate::tui::*;
use color_eyre::Result;

//-- main code --
// #[cfg(not(test))]

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = ui::init()?;
    let app_result = App::default().run(&mut terminal);
    if let Err(err) = ui::restore() {
        eprintln!(
            "failed to restore terminal. Run `reset` or restart your terminal to recover: {}",
            err
        );
    }
    app_result
}

#[cfg(test)]
mod tests {
    use crate::cmd::sort::dry_run_sort;
    use crate::subtitle::Subtitle;
    use episode::Episode;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_series_name_extraction() {
        let test_cases = vec![
            ("Breaking.Bad.S01E01.720p.mkv", "Breaking Bad"),
            ("The.Walking.Dead.S05E08.1080p.mp4", "The Walking Dead"),
            ("Game.of.Thrones.S08E06.Final.mp4", "Game of Thrones"),
            ("Rick.and.Morty.S03.E07.mp4", "Rick and Morty"),
            (
                "Better.Call.Saul.Season.1.Episode.1.mp4",
                "Better Call Saul",
            ),
            ("Loki.2021.S01E02.HDR.mkv", "Loki"),
            // Additional test cases
            ("The.Office.US.S04E12.720p.mkv", "The Office US"),
            ("Friends.S02.E15.1080p.mp4", "Friends"),
            ("Stranger.Things.Season.4.Episode.5.mkv", "Stranger Things"),
            ("The.Mandalorian.E13.HDR.mkv", "The Mandalorian"),
            ("Brooklyn.Nine-Nine.S06E12.WEBDL.mp4", "Brooklyn Nine Nine"),
        ];

        for (input, expected) in test_cases {
            let episode = Episode::new_test(input, false);
            assert_eq!(
                episode.name, expected,
                "Failed to extract name from {}",
                input
            );
        }
    }

    #[test]
    fn test_movie_detection() {
        let test_cases = vec![
            ("The.Dark.Knight.2008.1080p.BluRay.mp4", true),
            ("Inception.2010.HDR.2160p.mkv", true),
            ("Breaking.Bad.S01E01.720p.mkv", false),
            ("Avengers.Endgame.2019.IMAX.mkv", true),
            ("The.Mandalorian.S02E05.1080p.mp4", false),
            // Additional test cases
            ("Interstellar.2014.UHD.2160p.mkv", true),
            ("The.Matrix.1999.Remastered.BluRay.mp4", true),
            ("Dune.Part.One.2021.HDR.mkv", true),
            ("Star.Wars.Episode.IV.1977.4K.mkv", true),
            ("The.Crown.S04E10.1080p.mp4", false),
        ];

        for (input, expected) in test_cases {
            let episode = Episode::new_test(input, expected);
            assert_eq!(
                episode.is_movie, expected,
                "Failed to detect movie status for {}",
                input
            );
        }
    }

    #[test]
    fn test_subtitle_matching() {
        let episodes = vec![
            Episode::new_test("Breaking.Bad.S01E01.720p.mkv", false),
            Episode::new_test("The.Dark.Knight.2008.1080p.BluRay.mp4", true),
            Episode::new_test("Inception.2010.HDR.mkv", true),
            Episode::new_test("Friends.S02E15.1080p.mp4", false),
        ];

        let subtitles = vec![
            create_test_subtitle("Breaking.Bad.S01E01.en.srt"),
            create_test_subtitle("Breaking.Bad.S01E01.fr.srt"),
            create_test_subtitle("The.Dark.Knight.2008.en.srt"),
            create_test_subtitle("Inception.2010.en.srt"),
            create_test_subtitle("Friends.S02E15.en.srt"),
            create_test_subtitle("Friends.S02E15.es.srt"),
        ];

        dry_run_sort(
            &episodes,
            &subtitles,
            "Series".to_string(),
            "Movies".to_string(),
        )
        .unwrap();

        assert_eq!(subtitles[0].episode.name, "Breaking Bad");
        assert_eq!(subtitles[2].episode.name, "The Dark Knight");
        assert_eq!(subtitles[3].episode.name, "Inception");
        assert_eq!(subtitles[4].episode.name, "Friends");
    }

    #[test]
    fn test_season_episode_extraction() {
        let test_cases = vec![
            ("Show.S01E02.mp4", (1, 2)),
            ("Series.S05E10.720p.mkv", (5, 10)),
            ("Show.Season.2.Episode.3.mp4", (2, 3)),
            ("Show.S02.E07.1080p.mp4", (2, 7)),
            // Additional test cases
            ("Series.S10E23.HDR.mkv", (10, 23)),
            ("Show.Season.12.Episode.05.mp4", (12, 5)),
            ("Series.S01.Episode.15.1080p.mkv", (1, 15)),
        ];

        for (input, (expected_season, expected_episode)) in test_cases {
            let episode = Episode::new_test(input, false);
            assert_eq!(
                (episode.season, episode.episode),
                (expected_season, expected_episode),
                "Failed to extract season/episode from {}",
                input
            );
        }
    }

    #[test]
    fn test_year_extraction() {
        let test_cases = vec![
            ("Movie.2020.mp4", Some(2020)),
            ("Show.S01E01.2019.mp4", Some(2019)),
            ("Series.1080p.mp4", None),
            ("Film.1999.BluRay.mp4", Some(1999)),
            // Additional test cases
            ("Movie.2023.HDR.mkv", Some(2023)),
            ("Classic.1977.Remastered.mp4", Some(1977)),
            ("Show.From.2021.1080p.mkv", Some(2021)),
            ("Series.S01.2022.HDR.mp4", Some(2022)),
            ("Old.Movie.1950.Restored.mkv", Some(1950)),
            ("Modern.Film.2024.UHD.mp4", Some(2024)),
        ];

        for (input, expected_year) in test_cases {
            let episode = Episode::new_test(input, false);
            assert_eq!(
                episode.year, expected_year,
                "Failed to extract year from {}",
                input
            );
        }
    }

    fn create_test_subtitle(filename: &str) -> Subtitle {
        Subtitle::new(PathBuf::from(filename))
    }

    #[test]
    fn test_complex_filenames() {
        let test_cases = vec![
            (
                "The.100.Girlfriends.Who.Really.Really.Really.Really.Really.Love.You.S01E07.VOSTFR.mkv",
                ("The 100 Girlfriends Who Really Really Really Really Really Love You", 1, 7, false)
            ),
            (
                "Youkoso.Jitsuryoku.Shijou.Shugi.no.Kyoushitsu.e.S2.01.VOSTFR.1080p.mp4",
                ("Youkoso Jitsuryoku Shijou Shugi no Kyoushitsu e", 2, 1, false)
            ),
            (
                "Spider.Man.Across.the.Spider.Verse.2023.IMAX.2160p.HDR.mp4",
                ("Spider Man Across the Spider Verse", 0, 0, true)
            ),
            // Additional test cases
            (
                "That.Time.I.Got.Reincarnated.as.a.Slime.Season.2.Episode.12.1080p.mkv",
                ("That Time I Got Reincarnated as a Slime", 2, 12, false)
            ),
            (
                "The.Lord.of.the.Rings.The.Fellowship.of.the.Ring.Extended.2001.2160p.mkv",
                ("The Lord of the Rings The Fellowship of the Ring Extended", 0, 0, true)
            ),
            (
                "Dr.Stone.New.World.Part.2.S03E08.HDR.1080p.mkv",
                ("Dr Stone New World Part 2", 3, 8, false)
            )
        ];

        for (input, (expected_name, expected_season, expected_episode, is_movie)) in test_cases {
            let episode = Episode::new_test(input, is_movie);
            assert_eq!(episode.name, expected_name, "Name mismatch for {}", input);
            assert_eq!(
                episode.season, expected_season,
                "Season mismatch for {}",
                input
            );
            assert_eq!(
                episode.episode, expected_episode,
                "Episode mismatch for {}",
                input
            );
            assert_eq!(
                episode.is_movie, is_movie,
                "Movie flag mismatch for {}",
                input
            );
        }
    }
}
