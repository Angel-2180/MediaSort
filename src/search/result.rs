
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MediaType {
    Series,
    Movie,
}

#[derive(Debug, Clone)]
pub struct MediaResult{
    pub title: String,
    pub year: String,
    pub media_type: MediaType,
    pub accuracy: i64,
}

impl MediaResult {
    pub fn new(title: String, year: String, media_type: MediaType, accuracy: i64) -> MediaResult {
      MediaResult {
            title,
            year,
            media_type,
            accuracy,
        }
    }

    #[allow(dead_code)]
    pub fn string(&self) -> String {
        format!("{} ({})", self.title, self.year)
    }
}

pub fn get_highest_accuracy(results: Vec<MediaResult>) -> Option<MediaResult> {
    let mut highest_accuracy: i64 = 0;
    let mut closest_result: Option<MediaResult> = None;
    for result in results {
        if result.accuracy > highest_accuracy {
            highest_accuracy = result.accuracy;
            closest_result = Some(result);
        }
    }
    closest_result
}