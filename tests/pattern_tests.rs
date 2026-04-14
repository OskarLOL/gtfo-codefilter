use gtfo_codefilter::{load_words_from_str, match_pattern};

// Use the embedded CSV for tests
const CSV_DATA: &str = include_str!("../data/gtfo-possible-codes.csv");

#[test]
fn test_all_words_match_themselves() {
    let words = load_words_from_str(CSV_DATA).expect("Failed to load CSV");

    for word in &words {
        let result = match_pattern(&word, &words);
        assert!(result.contains(word), "Word '{}' should match itself", word);
    }
}

#[test]
fn test_wildcard_matching() {
    let words = load_words_from_str(CSV_DATA).expect("Failed to load CSV");

    for word in words.iter().take(10) {
        let mut pattern = String::from("----");
        pattern.replace_range(0..1, &word[0..1]);
        let result = match_pattern(&pattern, &words);
        assert!(
            result.contains(word),
            "Pattern '{}' should match '{}'",
            pattern,
            word
        );
    }
}

#[test]
fn test_no_match_for_invalid_pattern() {
    let words = load_words_from_str(CSV_DATA).expect("Failed to load CSV");
    let pattern = "zzzz"; // unlikely to exist
    let result = match_pattern(pattern, &words);
    assert!(
        result.is_empty(),
        "Pattern '{}' should not match any word",
        pattern
    );
}

#[test]
fn test_partial_wildcard_matching() {
    let words = load_words_from_str(CSV_DATA).expect("Failed to load CSV");

    for word in words.iter().take(10) {
        if word.len() < 4 {
            continue;
        }
        let mut pattern = String::from("----");
        pattern.replace_range(1..2, &word[1..2]);
        let result = match_pattern(&pattern, &words);
        assert!(
            result.contains(word),
            "Pattern '{}' should match '{}'",
            pattern,
            word
        );
    }
}

#[test]
fn test_full_wildcard_matches_all() {
    let words = load_words_from_str(CSV_DATA).expect("Failed to load CSV");
    let pattern = "----";
    let result = match_pattern(pattern, &words);
    assert_eq!(
        result.len(),
        words.len(),
        "Full wildcard should match all words"
    );
}

// --- Partial patterns: zip truncation enables live-filter-as-you-type ---

#[test]
fn test_short_pattern_filters_matching_prefix() {
    let words = vec!["abcd".to_string(), "abzz".to_string(), "xxxx".to_string()];
    let result = match_pattern("ab", &words);
    assert_eq!(result, vec!["abcd", "abzz"]);
}

#[test]
fn test_long_pattern_matches_shorter_words() {
    let words = vec!["ab".to_string()];
    let result = match_pattern("ab--xx", &words);
    assert_eq!(result, vec!["ab"]);
}

#[test]
fn test_empty_pattern_matches_everything() {
    let words = vec!["abcd".to_string(), "wxyz".to_string()];
    let result = match_pattern("", &words);
    assert_eq!(result, words);
}

#[test]
fn test_empty_word_list_returns_empty() {
    let result = match_pattern("abcd", &[]);
    assert!(result.is_empty(), "No words means no matches");
}

#[test]
fn test_empty_pattern_and_empty_words() {
    let result = match_pattern("", &[]);
    assert!(result.is_empty());
}

#[test]
fn test_load_words_from_empty_csv() {
    let words = load_words_from_str("").expect("Empty CSV should not error");
    assert!(words.is_empty(), "Empty CSV input produces no words");
}

#[test]
fn test_load_words_from_whitespace_only_csv() {
    let words = load_words_from_str("\n").expect("Whitespace CSV should not error");
    assert!(words.is_empty(), "A bare newline produces no records");
}

#[test]
fn test_load_words_lowercases_input() {
    let words = load_words_from_str("ABCD\nEFGH\n").expect("Should parse");
    assert_eq!(words, vec!["abcd", "efgh"]);
}

#[test]
fn test_load_words_strips_bom() {
    let words = load_words_from_str("\u{FEFF}able\nacid\n").expect("Should parse");
    assert_eq!(words, vec!["able", "acid"]);
}
