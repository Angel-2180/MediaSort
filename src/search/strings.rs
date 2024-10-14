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

use once_cell::sync::Lazy;
use regex::Regex;
use anyhow::{Result, bail};

pub static YEARSTR: Lazy<Regex> = Lazy::new(|| Regex::new(r"(19\d\d|20\d\d)").unwrap());
pub static GETYEAR: Lazy<Regex> = Lazy::new(|| Regex::new(&format!(r"\b{}\b", YEARSTR.as_str())).unwrap());
pub static YEAR: Lazy<Regex> = Lazy::new(|| Regex::new(&format!(r"^(.+?\b){}\b", YEARSTR.as_str())).unwrap());


pub fn accuracy(a: &str, b: &str) -> i64 {
    return 100 - dist(a, b)
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

pub fn clean_filename(filename_to_clean: &str) -> String {
    let mut cleaned = filename_to_clean.to_string();
    cleaned = cleaned[..cleaned.len() - 4].to_owned();
    cleaned = cleaned.replace(&['.', '_', '-', '+'][..], " ");

    //remove unwanted patterns as [] and () content
    cleaned = Regex::new(r"\[.*?\]").unwrap().replace_all(&cleaned, "").to_string();
    cleaned = Regex::new(r"\(.*?\)").unwrap().replace_all(&cleaned, "").to_string();
    //remove unwanted words
    cleaned = Regex::new(r"\b(net|fit|ws|tv|TV|ec|co|vip|cc|cfd|red|NanDesuKa|FANSUB|tokyo|WEBRip|DL|H264|Light|com|org|info|www|com|vostfree|VOSTFR|boats|uno|Wawacity|wawacity|WEB|TsundereRaws|1080p|720p|x264|AAC|Tsundere|Raws|fit|ws|tv|TV|ec)\b").unwrap().replace_all(&cleaned, "").to_string();
    //remove unwanted spaces
    cleaned = cleaned.split_whitespace().collect::<Vec<&str>>().join(" ");

    cleaned = cleaned.trim().to_string();

    cleaned
}

pub fn extract_series_name(filename_clean : &String) -> Result<String> {

    //use first string operation if possible to avoid regex
    let name: Vec<&str> = filename_clean.split_whitespace().collect();

    for i in 0..name.len() {
        if name[i].starts_with('S') && name[i].len() > 1 && name[i].chars().skip(1).all(char::is_numeric) {
            return Ok(name[..i].join(" ").trim().to_string());
        } else if name[i].starts_with('E') && name[i].len() > 1 && name[i].chars().skip(1).all(char::is_numeric) {
            return Ok(name[..i].join(" ").trim().to_string());
        }
    }

    let name_patterns = vec![
        r"(?i)(.+?)\s[S](\d{1,2})[E](\d{1,2})",         // Matches series with season and episode (e.g., S01E02)
        r"(?i)(.+?)\s[S](\d{1,2})",                     // Matches series with only season (e.g., S01)
        r"(?i)(.+?)\s[E](\d{1,2})",                     // Matches series with only episode (e.g., E02)
        r"(?i)(.+?)\s(\d{2})",                          // Matches series with only episode (e.g., 01)
        r"(?i)(.+?)\s(\d{4})",                          // Matches the title followed by a 4-digit year
        r"(?i)(.+?)\s(Part|Pt)\s?\d+",                  // Matches parts like "Part 2"
        r"(?i)(.+?)(\.\d+)?$",                          // Matches a title optionally followed by a number at the end
    ];


    for pattern in name_patterns {
        let re = Regex::new(pattern).unwrap();
        if let Some(captures) = re.captures(&filename_clean) {
            if let Some(name) = captures.get(1) {
                return Ok(name.as_str().trim().to_string());
            }
        }
    }

    bail!("Name not found")
}