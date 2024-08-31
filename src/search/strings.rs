// Remove the invalid attribute
/*var (
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

pub static NONALPHA: Lazy<Regex> = Lazy::new(|| Regex::new(r"[^a-z0-9]").unwrap());
pub static YEARSTR: Lazy<Regex> = Lazy::new(|| Regex::new(r"(19\d\d|20\d\d)").unwrap());
pub static ONLYYEAR: Lazy<Regex> = Lazy::new(|| Regex::new(&format!("^{}$", YEARSTR.as_str())).unwrap());
pub static GETYEAR: Lazy<Regex> = Lazy::new(|| Regex::new(&format!(r"\b{}\b", YEARSTR.as_str())).unwrap());
pub static GETDATE: Lazy<Regex> = Lazy::new(|| Regex::new(&format!(r"\b{}-(\d\d)-(\d\d)\b", YEARSTR.as_str())).unwrap());
pub static SAMPLE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\bsample\b").unwrap());
pub static ENCODINGS: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(720p|1080p|hdtv|x264|dts|bluray)\b.*").unwrap());
pub static SPACES: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());
pub static EPISEASON: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(.+?)\bs?(eason)?(\d{1,2})(e|\ |\ e|x|xe)(pisode)?(\d{1,2})\b").unwrap());
pub static EPIDATE: Lazy<Regex> = Lazy::new(|| Regex::new(&format!(r"^(.+?\b)({} \d{{2}} \d{{2}}|\d{{2}} \d{{2}} {})\b", YEARSTR.as_str(), YEARSTR.as_str())).unwrap());
pub static YEAR: Lazy<Regex> = Lazy::new(|| Regex::new(&format!(r"^(.+?\b){}\b", YEARSTR.as_str())).unwrap());
pub static JOINEDEPISEASON: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(.+?\b)(\d)(\d{2})\b").unwrap());
pub static PARTNUM: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(.+?\b)(\d{1,2})\b").unwrap());

pub fn normalize_string(input: &str) -> String {
    let mut output = input.to_lowercase();
    output = NONALPHA.replace_all(&output, " ").to_string();
    output = ENCODINGS.replace_all(&output, "").to_string();
    output = SPACES.replace_all(&output, " ").to_string();
    output = output.trim().to_string();
    output
}


pub fn abs(n: i64) -> i64 {
    if n < 0 {
        return -n
    }
    return n
}

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

pub fn is_near(a: &str, b: &str) -> bool {
    let lendiff = abs(a.chars().count() as i64 - b.chars().count() as i64);
    return dist(a, b) - lendiff <= 5;
}