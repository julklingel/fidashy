use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

const CANDIDATE_SEPARATORS: [u8; 4] = [b',', b';', b'\t', b'|'];
const DEFAULT_SEPARATOR: u8 = b',';
const PRESCAN_LINE_LIMIT: usize = 25;
const DEFAULT_INFERENCE_SAMPLE_ROWS: usize = 5000;

fn with_context<T, E: Display>(result: Result<T, E>, context: &str) -> Result<T, String> {
    result.map_err(|e| format!("{context}: {e}"))
}

fn ensure_csv_extension(path: &Path) -> Result<(), String> {
    let ext = path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .ok_or_else(|| format!("Invalid file extension for {}", path.display()))?;

    if ext != "csv" {
        return Err(format!("Only CSV files are supported: {}", path.display()));
    }

    Ok(())
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InferredType {
    Boolean,
    Int64,
    Float64,
    Date,
    Datetime,
    Utf8,
}

impl InferredType {
    pub fn to_duckdb_type(&self) -> &'static str {
        match self {
            InferredType::Boolean => "BOOLEAN",
            InferredType::Int64 => "BIGINT",
            InferredType::Float64 => "DOUBLE",
            InferredType::Date => "DATE",
            InferredType::Datetime => "TIMESTAMP",
            InferredType::Utf8 => "TEXT",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferredColumn {
    pub name: String,
    pub inferred: InferredType,
    pub confidence: f32,
    pub nullable: bool,
    pub observed_non_null_rows: usize,
}

#[derive(Default)]
struct ColumnInferenceStats {
    non_null: usize,
    bool_matches: usize,
    int_matches: usize,
    float_matches: usize,
    date_matches: usize,
    datetime_matches: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TemporalKind {
    Date,
    Datetime,
}

fn strip_wrapping_quotes(value: &str) -> &str {
    if value.len() < 2 {
        return value;
    }

    let bytes = value.as_bytes();
    let first = bytes[0];
    let last = bytes[bytes.len() - 1];

    if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
        &value[1..value.len() - 1]
    } else {
        value
    }
}

fn normalize_cell_value(value: &AnyValue<'_>) -> Option<String> {
    let raw = match value {
        AnyValue::Null => return None,
        _ => value.to_string(),
    };

    let mut normalized = raw.trim().to_string();
    if normalized.is_empty() {
        return None;
    }

    normalized = strip_wrapping_quotes(&normalized).trim().to_string();
    normalized = normalized.replace("\"\"", "\"");

    if normalized.is_empty() {
        return None;
    }

    let lower = normalized.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "null" | "none" | "n/a" | "na" | "nan" | "-"
    ) {
        return None;
    }

    Some(normalized)
}

fn is_boolean_like(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "true" | "false" | "t" | "f" | "yes" | "no" | "y" | "n" | "1" | "0"
    )
}

fn strip_sign(value: &str) -> &str {
    if let Some(stripped) = value.strip_prefix('-') {
        stripped
    } else if let Some(stripped) = value.strip_prefix('+') {
        stripped
    } else {
        value
    }
}

fn is_thousands_grouped_integer(value: &str, separator: char) -> bool {
    let mut parts = value.split(separator);
    let Some(first) = parts.next() else {
        return false;
    };

    if first.is_empty() || first.len() > 3 || !first.chars().all(|ch| ch.is_ascii_digit()) {
        return false;
    }

    let mut has_group = false;
    for part in parts {
        has_group = true;
        if part.len() != 3 || !part.chars().all(|ch| ch.is_ascii_digit()) {
            return false;
        }
    }

    has_group
}

fn is_int_like(value: &str) -> bool {
    let compact = value.replace([' ', '_'], "");
    if compact.is_empty() {
        return false;
    }

    let unsigned = strip_sign(&compact);

    if unsigned.chars().all(|ch| ch.is_ascii_digit()) {
        return compact.parse::<i64>().is_ok();
    }

    if unsigned.contains(',') && !unsigned.contains('.') {
        if is_thousands_grouped_integer(unsigned, ',') {
            return unsigned.replace(',', "").parse::<i64>().is_ok();
        }

        return false;
    }

    if unsigned.contains('.') && !unsigned.contains(',') {
        if is_thousands_grouped_integer(unsigned, '.') {
            return unsigned.replace('.', "").parse::<i64>().is_ok();
        }

        return false;
    }

    false
}

fn normalize_decimal_candidate(value: &str) -> Option<String> {
    let compact = value.replace([' ', '_'], "");
    let has_dot = compact.contains('.');
    let has_comma = compact.contains(',');

    if has_dot && has_comma {
        let last_dot = compact.rfind('.').unwrap_or(0);
        let last_comma = compact.rfind(',').unwrap_or(0);

        if last_dot > last_comma {
            Some(compact.replace(',', ""))
        } else {
            Some(compact.replace('.', "").replace(',', "."))
        }
    } else if has_comma {
        let comma_count = compact.matches(',').count();
        if comma_count > 1 {
            Some(compact.replace(',', ""))
        } else {
            Some(compact.replace(',', "."))
        }
    } else {
        Some(compact)
    }
}

fn is_float_like(value: &str) -> bool {
    let Some(candidate) = normalize_decimal_candidate(value) else {
        return false;
    };

    candidate.parse::<f64>().is_ok()
}

fn parse_date_ymd(segment: &str, separator: char) -> bool {
    let mut parts = segment.split(separator);
    let (Some(year), Some(month), Some(day), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return false;
    };

    if year.len() != 4 {
        return false;
    }

    let Ok(year_num) = year.parse::<u32>() else {
        return false;
    };
    let Ok(month_num) = month.parse::<u32>() else {
        return false;
    };
    let Ok(day_num) = day.parse::<u32>() else {
        return false;
    };

    (1900..=2200).contains(&year_num)
        && (1..=12).contains(&month_num)
        && (1..=31).contains(&day_num)
}

fn parse_date_dmy_or_mdy(segment: &str, separator: char) -> bool {
    let mut parts = segment.split(separator);
    let (Some(first), Some(second), Some(year), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return false;
    };

    if year.len() != 4 {
        return false;
    }

    let Ok(first_num) = first.parse::<u32>() else {
        return false;
    };
    let Ok(second_num) = second.parse::<u32>() else {
        return false;
    };
    let Ok(year_num) = year.parse::<u32>() else {
        return false;
    };

    (1900..=2200).contains(&year_num)
        && (1..=31).contains(&first_num)
        && (1..=12).contains(&second_num)
}

fn is_date_like(value: &str) -> bool {
    parse_date_ymd(value, '-')
        || parse_date_ymd(value, '/')
        || parse_date_dmy_or_mdy(value, '.')
        || parse_date_dmy_or_mdy(value, '/')
}

fn strip_timezone_suffix(time_value: &str) -> &str {
    let trimmed = time_value.trim_end_matches('Z');
    let bytes = trimmed.as_bytes();

    for index in 1..bytes.len() {
        if bytes[index] == b'+' || bytes[index] == b'-' {
            return &trimmed[..index];
        }
    }

    trimmed
}

fn is_time_like(value: &str) -> bool {
    let clean = strip_timezone_suffix(value);
    let mut parts = clean.split(':');
    let (Some(hour), Some(minute), second_opt) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };

    if parts.next().is_some() {
        return false;
    }

    let Ok(hour_num) = hour.parse::<u32>() else {
        return false;
    };
    let Ok(minute_num) = minute.parse::<u32>() else {
        return false;
    };

    if !(0..=23).contains(&hour_num) || !(0..=59).contains(&minute_num) {
        return false;
    }

    if let Some(second) = second_opt {
        let second_part = second.split(['.', ',']).next().unwrap_or(second);
        let Ok(second_num) = second_part.parse::<u32>() else {
            return false;
        };

        (0..=59).contains(&second_num)
    } else {
        true
    }
}

fn parse_temporal_kind(value: &str) -> Option<TemporalKind> {
    let trimmed = value.trim();

    let mut split = trimmed.split(['T', ' ']);
    let date_part = split.next()?;

    if let Some(time_part) = split.next() {
        if split.next().is_some() {
            return None;
        }

        if is_date_like(date_part) && is_time_like(time_part) {
            return Some(TemporalKind::Datetime);
        }

        return None;
    }

    if is_date_like(date_part) {
        Some(TemporalKind::Date)
    } else {
        None
    }
}

fn is_amount_like_column(column_name: &str) -> bool {
    let lower = column_name.to_ascii_lowercase();
    ["betrag", "amount", "price", "cost"]
        .iter()
        .any(|needle| lower.contains(needle))
}

fn choose_inferred_type(stats: &ColumnInferenceStats, column_name: &str) -> (InferredType, f32) {
    if stats.non_null == 0 {
        return (InferredType::Utf8, 1.0);
    }

    let non_null = stats.non_null as f32;
    let bool_conf = stats.bool_matches as f32 / non_null;
    let int_conf = stats.int_matches as f32 / non_null;
    let float_conf = stats.float_matches as f32 / non_null;
    let date_conf = stats.date_matches as f32 / non_null;
    let datetime_conf = stats.datetime_matches as f32 / non_null;
    let temporal_conf = (stats.date_matches + stats.datetime_matches) as f32 / non_null;
    let numeric_conf_delta = (int_conf - float_conf).abs();

    if is_amount_like_column(column_name)
        && int_conf >= 0.90
        && float_conf >= 0.90
        && numeric_conf_delta <= 0.08
    {
        return (InferredType::Float64, float_conf.max(int_conf));
    }

    if bool_conf >= 0.95 {
        return (InferredType::Boolean, bool_conf);
    }

    if int_conf >= 0.95 {
        return (InferredType::Int64, int_conf);
    }

    if float_conf >= 0.95 {
        return (InferredType::Float64, float_conf);
    }

    if stats.datetime_matches > 0 && temporal_conf >= 0.90 {
        return (InferredType::Datetime, temporal_conf.max(datetime_conf));
    }

    if date_conf >= 0.90 {
        return (InferredType::Date, date_conf);
    }

    let max_match = bool_conf
        .max(int_conf)
        .max(float_conf)
        .max(date_conf)
        .max(datetime_conf)
        .max(temporal_conf);
    (InferredType::Utf8, 1.0 - max_match)
}

pub fn infer_dataframe_schema(df: &DataFrame, sample_rows: usize) -> Vec<InferredColumn> {
    let sampled_row_count = if sample_rows == 0 {
        df.height().min(DEFAULT_INFERENCE_SAMPLE_ROWS)
    } else {
        sample_rows.min(df.height())
    };

    let mut inferred_columns = Vec::with_capacity(df.width());
    for column in df.columns() {
        let series = column.as_materialized_series();
        let mut stats = ColumnInferenceStats::default();

        for (index, value) in series.iter().enumerate() {
            if index >= sampled_row_count {
                break;
            }

            let Some(cell) = normalize_cell_value(&value) else {
                continue;
            };

            stats.non_null += 1;
            if is_boolean_like(&cell) {
                stats.bool_matches += 1;
            }
            if is_int_like(&cell) {
                stats.int_matches += 1;
            }
            if is_float_like(&cell) {
                stats.float_matches += 1;
            }
            if let Some(kind) = parse_temporal_kind(&cell) {
                match kind {
                    TemporalKind::Date => stats.date_matches += 1,
                    TemporalKind::Datetime => stats.datetime_matches += 1,
                }
            }
        }

        let (inferred, confidence) = choose_inferred_type(&stats, series.name());
        inferred_columns.push(InferredColumn {
            name: series.name().to_string(),
            inferred,
            confidence,
            nullable: stats.non_null < sampled_row_count,
            observed_non_null_rows: stats.non_null,
        });
    }

    inferred_columns
}

pub fn infer_csv_schema(
    path: &str,
    separator: u8,
    sample_rows: usize,
) -> Result<Vec<InferredColumn>, String> {
    let source_path = PathBuf::from(path);
    ensure_csv_extension(&source_path)?;

    if !source_path.exists() {
        return Err(format!("CSV file not found: {}", source_path.display()));
    }

    let source_path_str = source_path.to_string_lossy();
    let row_limit = if sample_rows == 0 {
        DEFAULT_INFERENCE_SAMPLE_ROWS
    } else {
        sample_rows
    };

    let sample_df = with_context(
        LazyCsvReader::new(PlRefPath::new(source_path_str.as_ref()))
            .with_has_header(true)
            .with_separator(separator)
            .with_encoding(CsvEncoding::LossyUtf8)
            .with_ignore_errors(true)
            .with_n_rows(Some(row_limit))
            .finish()
            .and_then(|lf| lf.collect()),
        "Failed to load CSV sample for schema inference",
    )?;

    Ok(infer_dataframe_schema(&sample_df, row_limit))
}

pub fn infer_csv_schema_with_sniffed_separator(
    path: &str,
    sample_rows: usize,
) -> Result<Vec<InferredColumn>, String> {
    let source_path = Path::new(path);
    let separator = sniff_separator(source_path)?;
    infer_csv_schema(path, separator, sample_rows)
}