use csv::ReaderBuilder;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

pub fn extract_non_empty_first_column(path: &Path) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(BufReader::new(file));
    let mut result = Vec::new();

    for record in reader.records() {
        if let Ok(record) = record {
            if let Some(value) = record.get(0) {
                if !value.trim().is_empty() {
                    result.push(value.to_string());
                }
            }
        }
    }

    Ok(result)
}
