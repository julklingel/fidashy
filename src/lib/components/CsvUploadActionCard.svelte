<script lang="ts">
  import CsvInfoHover from "./CsvInfoHover.svelte";
  import type {
    MergeCsvGroupResult,
    NextStepDecision,
    ProcessCsvResult,
    ProcessedPayload,
    SchemaGroup,
    SelectedCsvFile,
  } from "./csv-types";
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Card } from "$lib/components/ui/card";
  import { toast } from "$lib/components/ui/sonner/sonner";

  let { onProcessed }: { onProcessed: (payload: ProcessedPayload) => void } = $props();

  let selectedFiles = $state<SelectedCsvFile[]>([]);

  const toFileName = (path: string) => path.split(/[\\/]/).pop() ?? path;

  function toSelectedFile(path: string): SelectedCsvFile {
    return {
      path,
      name: toFileName(path),
    };
  }

  function buildGroups(result: ProcessCsvResult): SchemaGroup[] {
    return result.matching_header_groups.map((group, index) => {
      const files = group.file_paths.map(toSelectedFile);

      return {
        id: `group-${index}-${files.map((file) => file.path).join("|")}`,
        headers: group.headers,
        files,
        duplicateRows: group.duplicate_rows,
      };
    });
  }

  async function buildSingleDecisions(
    result: ProcessCsvResult,
    files: SelectedCsvFile[]
  ): Promise<NextStepDecision[]> {
    const groupedPaths = new Set(
      result.matching_header_groups.flatMap((group) => group.file_paths)
    );
    const singleFiles = files.filter((file) => !groupedPaths.has(file.path));

    const decisions = await Promise.all(
      singleFiles.map(async (file) => {
        const mergeResult = await invoke<MergeCsvGroupResult>("merge_csv_group", {
          paths: [file.path],
        });

        return {
          groupId: `single-${file.path}`,
          fileNames: [file.name],
          filePaths: [file.path],
          mergeResult,
        } satisfies NextStepDecision;
      })
    );

    return decisions;
  }

  function addSelectedPaths(paths: string[]) {
    const incomingFiles = paths.map(toSelectedFile);
    const merged = [...selectedFiles, ...incomingFiles];
    const uniqueByPath = new Map(merged.map((file) => [file.path, file]));
    selectedFiles = Array.from(uniqueByPath.values());
  }

  async function pickCsvFiles() {
    const selection = await open({
      multiple: true,
      filters: [{ name: "CSV", extensions: ["csv"] }],
    });

    if (!selection) {
      return;
    }

    const selectedPaths = Array.isArray(selection) ? selection : [selection];
    addSelectedPaths(selectedPaths);
  }

  function removeFile(index: number) {
    selectedFiles = selectedFiles.filter((_, fileIndex) => fileIndex !== index);
  }

  function clearFiles() {
    selectedFiles = [];
  }

  async function uploadFile(event: Event) {
    event.preventDefault();

    if (selectedFiles.length === 0) {
      toast("Please choose at least one CSV file.");
      return;
    }

    try {
      const result = await invoke<ProcessCsvResult>("process_csv_files", {
        paths: selectedFiles.map((file) => file.path),
      });
      const singleDecisions = await buildSingleDecisions(result, selectedFiles);

      onProcessed({
        processedFiles: result.processed_files,
        groups: buildGroups(result),
        singleDecisions,
      });

      toast(`Processed ${result.processed_files} CSV file(s).`);
      clearFiles();
    } catch (error) {
      toast(`Failed to process file: ${String(error)}`);
    }
  }
</script>

<Card className="w-full max-w-md space-y-4 rounded-lg border bg-card p-6 text-card-foreground shadow">
  <form class="space-y-4" onsubmit={uploadFile}>
    <div class="flex flex-col justify-center justify-items-center space-y-2">
      <div class="flex items-center gap-2">
        <CsvInfoHover />
        <p class="my-2 text-sm font-medium">Process CSV files</p>
      </div>

      <Button type="button" variant="outline" class="w-full" onclick={pickCsvFiles}>Choose CSV files</Button>

      {#if selectedFiles.length > 0}
        <div class="space-y-2">
          <div class="flex items-center justify-between">
            <p class="text-sm text-muted-foreground">Selected files: {selectedFiles.length}</p>
            <Button type="button" variant="outline" class="h-7 px-2 text-xs" onclick={clearFiles}>Clear all</Button>
          </div>

          <ul class="space-y-1 text-sm">
            {#each selectedFiles as file, index}
              <li class="flex items-center justify-between rounded-md border px-2 py-1">
                <span class="truncate pr-2">{file.name}</span>
                <Button
                  type="button"
                  variant="ghost"
                  class="h-7 px-2 text-xs"
                  onclick={() => removeFile(index)}
                >
                  Remove
                </Button>
              </li>
            {/each}
          </ul>
        </div>
      {/if}
    </div>

    <Button type="submit" class="w-full">Process</Button>
  </form>
</Card>
