use polars::prelude::*;
use std::collections::HashMap;


pub fn lazy_grouping_csv_many(
    paths: Vec<String>,
) -> PolarsResult<Vec<Vec<String>>> {
    let mut groups: HashMap<Vec<String>, Vec<String>> = HashMap::new();

    for path in paths {
        let mut lf = LazyCsvReader::new(PlRefPath::from(path.as_str()))
            .with_has_header(true)
            .finish()?;

        let mut columns = lf
            .collect_schema()?
            .iter_names()
            .map(|s| s.to_string().to_lowercase())
            .collect::<Vec<_>>();

        columns.sort();
        groups.entry(columns).or_default().push(path);
    }

    let grouped_paths: Vec<Vec<String>> = groups.into_values().collect();
    Ok(grouped_paths)
}


// pub fn vertical_concatenation( paths: Vec<String>) -> PolarsResult<Vec<String>> {

// }



