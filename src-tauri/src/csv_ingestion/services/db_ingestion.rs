use crate::csv_ingestion::models::{CachedDataFrame, GroupWithDuplicates, MergeCache};
use polars::prelude::*;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};


pub fn find_groups_between_db_and_files(
    paths: Vec<String>,
    cache: &MergeCache,
) -> Result<(), String>{

    let mut groups: HashMap<Vec<String>, Vec<String>> = HashMap::new();



    println!("service: got {} paths", paths.len());
    println!("service: cache available");

    Ok(())
    


}