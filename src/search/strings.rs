// Remove the invalid attribute
/*list of interesting regex (
    nonalpha        = regexp.MustCompile(`[^a-z0-9]`)
    yearstr         = `(19\d\d|20\d\d)`
    onlyYear        = regexp.MustCompile(`^` + yearstr + `$`)
    getYear         = regexp.MustCompile(`\b` + yearstr + `\b`)
    getDate         = regexp.MustCompile(`\b` + yearstr + `-(\d\d)-(\d\d)\b`)
    sample          = regexp.MustCompile(`\bsample\b`)
    encodings       = regexp.MustCompile(`\b(720p|1080p|hdtv|x264|dts|bluray)\b.*`) //strip all junk
    spaces          = regexp.MustCompile(`\s+`)
    episeason       = regexp.MustCompile(`^(.+?)\bs?(eason)?(\d{1,2})(e|\ |\ e|x|xe)(pisode)?(\d{1,2})\b`)
    epidate         = regexp.MustCompile(`^(.+?\b)(` + yearstr + ` \d{2} \d{2}|\d{2} \d{2} ` + yearstr + `)\b`)
    year            = regexp.MustCompile(`^(.+?\b)` + yearstr + `\b`)
    joinedepiseason = regexp.MustCompile(`^(.+?\b)(\d)(\d{2})\b`)
    partnum         = regexp.MustCompile(`^(.+?\b)(\d{1,2})\b`)
) */

use std::{
    fs,
    io::{self, BufRead},
};

use anyhow::{bail, Result};
use directories::BaseDirs;
use once_cell::sync::Lazy;
use regex::Regex;

pub static YEARSTR: Lazy<Regex> = Lazy::new(|| Regex::new(r"(19\d\d|20\d\d)").unwrap());
pub static GETYEAR: Lazy<Regex> =
    Lazy::new(|| Regex::new(&format!(r"\b{}\b", YEARSTR.as_str())).unwrap());
pub static UNWANTED_WORDS_FILE: Lazy<String> = Lazy::new(|| {
    let base_dirs = BaseDirs::new().unwrap();
    let dir_path = base_dirs.data_local_dir().join("MediaSort");
    if !dir_path.exists() {
        fs::create_dir_all(&dir_path).unwrap();
    }
    let file_path = dir_path.join("unwanted_words.txt");
    if !file_path.exists() {
        fs::write(&file_path, "net\nfit\nws\ntv\nTV\nec\nco\nvip\ncc\ncfd\nred\nNanDesuKa\nFANSUB\ntokyo\nWEBRip\nDL\nH264\nLight\ncom\norg\ninfo\nwww\ncom\nvostfree\nVOSTFR\nboats\nuno\nWawacity\nwawacity\nWEB\nTsundereRaws\n1080p\n720p\nx264\nAAC\nTsundere\nRaws\nfit\nws\ntv\nTV\nec\n").unwrap();
    }
    file_path.to_str().unwrap().to_string()
});

pub fn accuracy(a: &str, b: &str) -> i64 {
    return 100 - dist(a, b);
}

pub fn dist(a: &str, b: &str) -> i64 {
    let len_a = a.chars().count();
    let len_b = b.chars().count();
    if len_a < len_b {
        return dist(b, a);
    }
    // handle special case of 0 length
    if len_a == 0 {
        return len_b as i64;
    } else if len_b == 0 {
        return len_a as i64;
    }

    let len_b = len_b + 1;

    let mut pre;
    let mut tmp;
    let mut cur = vec![0; len_b];

    // initialize string b
    for i in 1..len_b {
        cur[i] = i;
    }

    // calculate edit distance
    for (i, ca) in a.chars().enumerate() {
        // get first column for this row
        pre = cur[0];
        cur[0] = i + 1;
        for (j, cb) in b.chars().enumerate() {
            tmp = cur[j + 1];
            cur[j + 1] = std::cmp::min(
                // deletion
                tmp + 1,
                std::cmp::min(
                    // insertion
                    cur[j] + 1,
                    // match or substitution
                    pre + if ca == cb { 0 } else { 1 },
                ),
            );
            pre = tmp;
        }
    }
    cur[len_b - 1] as i64
}

pub fn clean_filename(filename_to_clean: &str) -> Result<String> {
    // Read the unwanted words from the file and build a regex pattern
    let unwanted_words = read_unwanted_words(&UNWANTED_WORDS_FILE)?;
    let unwanted_words_pattern = format!(r"\b({})\b", unwanted_words.join("|"));
    let unwanted_words_regex = Regex::new(&unwanted_words_pattern).unwrap();

    // Start cleaning the filename
    let mut cleaned = filename_to_clean.to_string();

    // Remove file extension by splitting on the last dot
    if let Some(idx) = cleaned.rfind('.') {
        cleaned = cleaned[..idx].to_string();
    }

    // Replace certain characters with spaces
    cleaned = cleaned.replace(&['.', '_', '-', '+'][..], " ");

    // Remove unwanted patterns like [] and () content
    cleaned = Regex::new(r"\[.*?\]")
        .unwrap()
        .replace_all(&cleaned, "")
        .to_string();
    cleaned = Regex::new(r"\(.*?\)")
        .unwrap()
        .replace_all(&cleaned, "")
        .to_string();

    // Remove unwanted words using the dynamically created regex
    cleaned = unwanted_words_regex.replace_all(&cleaned, "").to_string();

    // Clean up extra spaces
    cleaned = cleaned.split_whitespace().collect::<Vec<&str>>().join(" ");
    cleaned = cleaned.trim().to_string();

    Ok(cleaned)
}

fn read_unwanted_words(file_path: &str) -> io::Result<Vec<String>> {
    // Read lines from the file and collect them into a vector
    let file = fs::File::open(file_path)?;
    let reader = io::BufReader::new(file);
    let words = reader
        .lines()
        .filter_map(|line| line.ok())
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect::<Vec<String>>();

    Ok(words)
}

pub fn extract_series_name(filename_clean: &String) -> Result<String> {
    // First try string operations to avoid regex
    let name: Vec<&str> = filename_clean.split_whitespace().collect();

    // Look for season/episode markers
    for i in 0..name.len() {
        // Check for "Season" or "Episode" keywords
        if name[i].eq_ignore_ascii_case("season") || name[i].eq_ignore_ascii_case("episode") {
            return Ok(name[..i].join(" ").trim().to_string());
        }

        // Check for SXX or EXX patterns
        if (name[i].starts_with('S') || name[i].starts_with('E'))
            && name[i].len() > 1
            && name[i].chars().skip(1).all(char::is_numeric)
        {
            return Ok(name[..i].join(" ").trim().to_string());
        }

        // Check for SXXEXX pattern
        if name[i].starts_with('S') && name[i].contains('E') {
            return Ok(name[..i].join(" ").trim().to_string());
        }

        // Check for year pattern (for movies)
        if let Some(_year_match) = YEARSTR.find(name[i]) {
            return Ok(name[..i].join(" ").trim().to_string());
        }
    }

    // Fallback to regex patterns
    let name_patterns = vec![
        r"(?i)(.+?)(?:\s+S(?:eason)?\s*\d{1,2}|E(?:pisode)?\s*\d{1,2})", // Matches before Season/Episode
        r"(?i)(.+?)\s+S\d{1,2}E\d{1,2}",                                 // Matches before SXXEXX
        r"(?i)(.+?)(?:\s+\d{4}|\s*$)",    // Matches before year or end
        r"(?i)(.+?)\s+(?:Part|Pt)\s*\d+", // Matches before Part X
    ];

    // Try each pattern in order
    for pattern in name_patterns {
        let re = Regex::new(pattern).unwrap();
        if let Some(captures) = re.captures(filename_clean) {
            if let Some(name) = captures.get(1) {
                let clean_name = name.as_str().trim().to_string();
                if !clean_name.is_empty() {
                    // Return the full name without truncating
                    return Ok(clean_name);
                }
            }
        }
    }

    bail!("name not found")
}
