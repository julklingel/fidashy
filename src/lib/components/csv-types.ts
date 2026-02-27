export type SelectedCsvFile = {
  path: string;
  name: string;
};

export type GroupWithDuplicates = {
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

