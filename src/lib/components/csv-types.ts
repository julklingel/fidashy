export type SelectedCsvFile = {
  path: string;
  name: string;
};

export type GroupWithDuplicates = {
  group_id: string;
  paths: string[];
  duplicate_count: number;
  total_entries: number;
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

export type GroupResolutionSummary = {
  mergedGroups: MergedGroup[];
  mergedGroupIds: string[];
  standaloneGroups: StandaloneGroup[];
};

export type FileDbCacheMatch = {
  file_path: string;
  matched_table_names: string[];
  matched_cache_group_ids: string[];
};

export type FindGroupsBetweenDbAndFilesResult = {
  matched_files: FileDbCacheMatch[];
  matched_groups: CachedGroupDbMatch[];
};

export type CachedGroupDbMatch = {
  group_id: string;
  paths: string[];
  matched_table_names: string[];
};

export type MergeFileIntoTableResult = {
  source_path: string;
  target_table: string;
  rows_written: number;
  message: string;
};

export type MergeCachedGroupIntoTableResult = {
  group_id: string;
  target_table: string;
  source_file_count: number;
  rows_written: number;
  message: string;
};

export type CreateTableFromCachedGroupResult = {
  group_id: string;
  created_table: string;
  source_file_count: number;
  rows_written: number;
  message: string;
};

