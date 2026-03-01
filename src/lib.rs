use std::error::Error;
use eframe::egui::Color32;

pub fn load_words_from_str(csv_data: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut words = Vec::new();
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(csv_data.as_bytes());

    for result in rdr.records() {
        let record = result?;
        words.push(record[0].to_lowercase());
    }

    Ok(words)
}

pub fn match_pattern(pattern: &str, words: &[String]) -> Vec<String> {
    words
        .iter()
        .filter(|word| {
            word.chars()
                .zip(pattern.chars())
                .all(|(w_c, p_c)| p_c == '-' || w_c == p_c)
        })
        .cloned()
        .collect()
}

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    ((a as f32) + (b as f32 - a as f32) * t) as u8
}


pub fn blend_color(num: usize) -> Color32 {
    // 1 match = Green, 10+ matches = Yellow
    let t = ((num.saturating_sub(1)) as f32 / 9.0).clamp(0.0, 1.0);
    
    Color32::from_rgb(
        lerp(0, 255, t),   // R: 0 -> 255
        255,               // G: Always 255
        0                  // B: Always 0
    )
}
