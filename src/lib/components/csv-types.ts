export type SelectedCsvFile = {
  path: string;
  name: string;
};

export type ProcessCsvResult = {
  processed_files: number;
  group_count: number;
  total_duplicate_rows: number;
  files: { path: string; headers: string[] }[];
  matching_header_groups: { headers: string[]; file_paths: string[]; duplicate_rows: number }[];
};

export type SchemaGroup = {
  id: string;
  headers: string[];
  files: SelectedCsvFile[];
  duplicateRows: number;
};

export type GroupMergeStatus = {
  isMerging: boolean;
  status: "idle" | "merged" | "canceled";
  progress: number;
};

export type MergeState = {
  groups: Record<string, GroupMergeStatus>;
};

export type ProcessedPayload = {
  processedFiles: number;
  groups: SchemaGroup[];
  singleDecisions: NextStepDecision[];
};

export type MergeCsvGroupResult = {
  input_rows: number;
  merged_rows: number;
  duplicate_rows_removed: number;
  merged_columns: number;
  merged_headers: string[];
  matching_table_name: string | null;
  duplicate_rows_with_db: number;
  requires_user_choice: boolean;
};

export type CsvIngestionWriteResult = {
  table_name: string;
  input_rows: number;
  rows_inserted: number;
  rows_skipped_duplicates: number;
  created_new_table: boolean;
};

export type NextStepDecision = {
  groupId: string;
  fileNames: string[];
  filePaths: string[];
  mergeResult: MergeCsvGroupResult;
};
