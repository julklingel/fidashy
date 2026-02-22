<script lang="ts">
  import CsvInfoHover from './CsvInfoHover.svelte';

  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import Info from "@lucide/svelte/icons/info";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Card } from "$lib/components/ui/card";
  import { toast } from "$lib/components/ui/sonner/sonner";

  type SelectedCsvFile = {
    path: string;
    name: string;
  };

  type ProcessCsvResult = {
    processed_files: number;
    files: { path: string; headers: string[] }[];
    matching_header_groups: { headers: string[]; file_paths: string[] }[];
  };

  let selectedFiles = $state<SelectedCsvFile[]>([]);
  let processedFiles = $state<{ path: string; name: string; headers: string[] }[]>([]);
  let matchingHeaderGroups = $state<{ headers: string[]; fileNames: string[] }[]>([]);

  async function pickCsvFiles() {
    const selection = await open({
      multiple: true,
      filters: [{ name: "CSV", extensions: ["csv"] }],
    });

    if (!selection) {
      return;
    }

    const selectedPaths = Array.isArray(selection) ? selection : [selection];
    const incomingFiles = selectedPaths.map((path) => ({
      path,
      name: path.split(/[\\/]/).pop() ?? path,
    }));

    const merged = [...selectedFiles, ...incomingFiles];
    const uniqueByPath = new Map(merged.map((file) => [file.path, file]));
    selectedFiles = Array.from(uniqueByPath.values());
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

      processedFiles = result.files.map((file) => ({
        path: file.path,
        name: file.path.split(/[\\/]/).pop() ?? file.path,
        headers: file.headers,
      }));

      matchingHeaderGroups = result.matching_header_groups.map((group) => ({
        headers: group.headers,
        fileNames: group.file_paths.map((path) => path.split(/[\\/]/).pop() ?? path),
      }));

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
      <CsvInfoHover/>
        <p class="text-sm my-2 font-medium">Process CSV files</p>
    
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

  {#if processedFiles.length > 0}
    <div class="space-y-2 border-t pt-4">
      <p class="text-sm font-medium">Detected headers</p>
      <ul class="space-y-2 text-sm">
        {#each processedFiles as file}
          <li class="rounded-md border px-3 py-2">
            <p class="font-medium">{file.name}</p>
            {#if file.headers.length > 0}
              <p class="text-muted-foreground wrap-break-word">{file.headers.join(", ")}</p>
            {:else}
              <p class="text-muted-foreground">No headers found.</p>
            {/if}
          </li>
        {/each}
      </ul>
    </div>
  {/if}

  {#if matchingHeaderGroups.length > 0}
    <div class="space-y-2 border-t pt-4">
      <p class="text-sm font-medium">Matching header groups</p>
      <ul class="space-y-2 text-sm">
        {#each matchingHeaderGroups as group}
          <li class="rounded-md border px-3 py-2">
            <p class="text-muted-foreground wrap-break-word">Headers: {group.headers.join(", ")}</p>
            <p class="font-medium">Files: {group.fileNames.join(", ")}</p>
          </li>
        {/each}
      </ul>
    </div>
  {/if}
</Card>
