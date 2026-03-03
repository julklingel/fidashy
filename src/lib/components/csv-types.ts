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

export type GroupResolutionSummary = {
  mergedGroupIds: string[];
  standaloneGroups: StandaloneGroup[];
};

