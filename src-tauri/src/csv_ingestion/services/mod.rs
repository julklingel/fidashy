pub mod data_sniff;
pub mod dedup;
pub mod grouping;
pub mod pipeline;
pub mod schema;

pub use pipeline::process_csv_files;
pub use pipeline::merge_csv_group;
pub use pipeline::create_table_from_csv_group;
pub use pipeline::merge_csv_group_into_existing_table;
