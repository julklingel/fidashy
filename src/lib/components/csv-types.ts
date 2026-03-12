export type SelectedCsvFile = {
  path: string;
  name: string;
};

export type GroupProposal = {
  group_id: string;
  paths: string[];
};

export type GroupingResult = {
  processed_files: number;
  group_count: number;
  total_duplicate_rows: number;
  files: { path: string; headers: string[] }[];
  matching_header_groups: { headers: string[]; file_paths: string[]; duplicate_rows: number }[];
};

export type Group = {
  id: string;
  headers: string[];
  files: SelectedCsvFile[];
  duplicateRows: number;
};

export type DeduplicateGroupResult = {
  group_id: string;
  source_file_count: number;
  rows_before: number;
  rows_after: number;
  duplicates_removed: number;
  message: string;
};

export type SkipMergeGroupResult = {
  group_id: string;
  source_file_count: number;
  standalone_paths: string[];
  message: string;
};

export type StandaloneGroup = {
  group_id: string;
  paths: string[];
};

export type MergedGroup = {
  group_id: string;
  paths: string[];
};

export type DbSourceKind = "group" | "standalone";

export type DbMatchProposal = {
  source_kind: DbSourceKind;
  source_id: string;
  source_paths: string[];
  columns: string[];
  matching_tables: string[];
};

export type GroupResolutionSummary = {
  mergedGroups: MergedGroup[];
  mergedGroupIds: string[];
  standaloneGroups: StandaloneGroup[];
};

export type CreateTableFromSourceResult = {
  source_kind: string;
  source_id: string;
  target_table: string;
  rows_before: number;
  rows_after: number;
  duplicates_removed: number;
};

export type MergeSourceIntoTableResult = {
  source_kind: string;
  source_id: string;
  target_table: string;
  rows_before: number;
  rows_after: number;
  rows_inserted: number;
  duplicates_removed: number;
};

