export type SelectedCsvFile = {
  path: string;
  name: string;
};

export type ProcessCsvResult = {
  processed_files: number;
  files: { path: string; headers: string[] }[];
  matching_header_groups: { headers: string[]; file_paths: string[] }[];
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
};
