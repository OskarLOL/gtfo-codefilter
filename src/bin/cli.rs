use gtfo_codefilter::{load_words_from_str, match_pattern};
use std::io::{self, Write};

// Embed the CSV directly into the binary
const CSV_DATA: &str = include_str!("../../data/gtfo-possible-codes.csv");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let words = load_words_from_str(CSV_DATA)?;

    loop {
        print!("Enter 4-letter pattern (use '-' as wildcard, '#' to quit): ");
        io::stdout().flush()?;

        let mut pattern = String::new();
        io::stdin().read_line(&mut pattern)?;
        let pattern = pattern.trim().to_lowercase();

        if pattern == "#" {
            println!("Exiting...");
            break;
        }

        if pattern.len() != 4 {
            println!("Pattern must be exactly 4 characters.");
            continue;
        }

        let matches = match_pattern(&pattern, &words);
        if matches.is_empty() {
            println!("No matches found.");
        } else {
            println!("Matching words:");
            for word in matches {
                println!("{}", word);
            }
        }
    }

    Ok(())
}
