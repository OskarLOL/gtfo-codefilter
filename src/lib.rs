use std::error::Error;

#[must_use]
pub fn load_words_from_str(data: &str) -> Result<Vec<String>, Box<dyn Error>> {
    Ok(data
        .lines()
        .map(|line| line.trim().trim_start_matches('\u{FEFF}').to_lowercase())
        .filter(|line| !line.is_empty())
        .collect())
}

#[must_use]
pub fn match_pattern(pattern: &str, words: &[String]) -> Vec<String> {
    words
        .iter()
        .filter(|word| {
            word.chars()
                .zip(pattern.chars())
                .all(|(w, p)| p == '-' || w == p)
        })
        .cloned()
        .collect()
}
