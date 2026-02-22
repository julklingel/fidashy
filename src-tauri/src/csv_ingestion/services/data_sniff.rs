use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

const CANDIDATE_SEPARATORS: [u8; 4] = [b',', b';', b'\t', b'|'];
const DEFAULT_SEPARATOR: u8 = b',';
const PRESCAN_LINE_LIMIT: usize = 25;


// Data Sniff needs to be refactored it needs to fullfill the following tasks:
// - sniff the separator in csv files 
// - inference what data types are used 
// - format the dataframes accordingly

fn with_context<T, E: Display>(result: Result<T, E>, context: &str) -> Result<T, String> {
    result.map_err(|e| format!("{context}: {e}"))
}

fn separator_count_outside_quotes(line: &str, separator: u8) -> usize {
    let mut in_quotes = false;
    let mut count = 0usize;
    let bytes = line.as_bytes();
    let mut index = 0usize;

    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'"' {
            if in_quotes && index + 1 < bytes.len() && bytes[index + 1] == b'"' {
                index += 2;
                continue;
            }

            in_quotes = !in_quotes;
            index += 1;
            continue;
        }

        if !in_quotes && byte == separator {
            count += 1;
        }

        index += 1;
    }

    count
}

fn score_separator(lines: &[String], separator: u8) -> (usize, usize, usize) {
    let mut rows_with_separator = 0usize;
    let mut total_separator_count = 0usize;
    let mut column_count_freq: HashMap<usize, usize> = HashMap::new();

    for line in lines {
        let separator_count = separator_count_outside_quotes(line, separator);
        if separator_count == 0 {
            continue;
        }

        rows_with_separator += 1;
        total_separator_count += separator_count;
        *column_count_freq.entry(separator_count + 1).or_insert(0) += 1;
    }

    let mode_frequency = column_count_freq.values().copied().max().unwrap_or(0);
    (rows_with_separator, mode_frequency, total_separator_count)
}

pub fn sniff_separator(source_path: &Path) -> Result<u8, String> {
    let file = with_context(
        File::open(source_path),
        "Failed to open CSV file for separator sniffing",
    )?;
    let mut reader = BufReader::new(file);

    let mut sample_lines = Vec::with_capacity(PRESCAN_LINE_LIMIT);
    let mut line_buffer = Vec::new();

    while sample_lines.len() < PRESCAN_LINE_LIMIT {
        line_buffer.clear();
        let bytes_read = with_context(
            reader.read_until(b'\n', &mut line_buffer),
            "Failed to read CSV line while sniffing separator",
        )?;

        if bytes_read == 0 {
            break;
        }

        let line = String::from_utf8_lossy(&line_buffer);
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let cleaned = if sample_lines.is_empty() {
            trimmed.trim_start_matches('\u{feff}').to_string()
        } else {
            trimmed.to_string()
        };
        sample_lines.push(cleaned);
    }

    if sample_lines.is_empty() {
        return Ok(DEFAULT_SEPARATOR);
    }

    let mut best_separator = DEFAULT_SEPARATOR;
    let mut best_score = (0usize, 0usize, 0usize);

    for separator in CANDIDATE_SEPARATORS {
        let score = score_separator(&sample_lines, separator);
        if score > best_score {
            best_score = score;
            best_separator = separator;
        }
    }

    Ok(best_separator)
}
