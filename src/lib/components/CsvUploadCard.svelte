<script lang="ts">
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
  };

  let selectedFiles = $state<SelectedCsvFile[]>([]);
  let processedFiles = $state<{ path: string; name: string; headers: string[] }[]>([]);

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
        <p class="text-sm my-2 font-medium">Process CSV files</p>
        <div class="group relative inline-flex">
          <button
            type="button"
            class="inline-flex h-5 w-5 items-center justify-center rounded-full border border-input text-muted-foreground"
            aria-label="How CSV processing works"
          >
            <Info class="h-3.5 w-3.5" />
          </button>
          <div class="pointer-events-none absolute left-0 top-6 z-10 hidden w-64 rounded-md border bg-popover p-3 text-xs text-popover-foreground shadow-md group-hover:block">
            Pick one or more CSV files. When you press Process, the app reads each provided file path and runs Polars checks on the CSV files.
          </div>
        </div>
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
</Card>
