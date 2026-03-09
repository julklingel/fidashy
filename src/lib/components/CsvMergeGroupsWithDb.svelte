<script lang="ts">
  import FileProcessingResults from "./FileProcessingResults.svelte";
  import type { GroupResolutionSummary } from "$lib/components/csv-types";
  import { Card } from "$lib/components/ui/card/index.js";
  import { invoke } from "@tauri-apps/api/core";
  import { toast } from "svelte-sonner";
  import Button from "./ui/button/button.svelte";

  type Props = {
    summary: GroupResolutionSummary;
    soloPaths?: string[];
    noGroupsFound?: boolean;
  };

  let { summary, soloPaths = [], noGroupsFound = false }: Props = $props();

  const mergedGroupIds = $derived(summary.mergedGroupIds);
  const standaloneGroups = $derived(summary.standaloneGroups);
  const standalonePaths = $derived(
    summary.standaloneGroups.flatMap((group) => group.paths),
  );
  const passthroughPaths = $derived(Array.from(new Set(soloPaths)));
  const allFilePaths = $derived.by(() =>
    Array.from(new Set([...passthroughPaths, ...standalonePaths])),
  );
  let isProcessing = $state(false);

  $inspect(summary);


  async function findGroupsWithDb(allFilePaths: string[], mergedGroupIds: string[]) {
    isProcessing = true;
    const paths = allFilePaths;
    const cacheIds = mergedGroupIds;



try {
  const response = await invoke("find_groups_between_db_and_files", {
    paths,
    cacheIds: cacheIds,
  });

      console.log(response);
      toast.success("success");
    } catch (error) {
      toast.error(`CSV processing failed: ${error}`);
    } finally {
      isProcessing = false;
    }
  }





let tableNames = $state<Record<string, string>>({});
let creatingByPath = $state<Record<string, boolean>>({});

  async function createTableFromPath(path: string) {
    const preferredTableName = (tableNames[path] ?? "").trim();

    if (!preferredTableName) {
      toast.error("Please enter a table name.");
      return;
    }

    creatingByPath[path] = true;
    try {
      await invoke("create_new_table_from_source", {
        sourcePath: path,
        preferredTableName,
      });
      toast.success(`Table '${preferredTableName}' created.`);
    } catch (error) {
      toast.error(`Create table failed: ${error}`);
    } finally {
      creatingByPath[path] = false;
    }
  }
</script>

<Card
  className="w-full max-w-2xl rounded-lg border bg-card p-6 text-card-foreground shadow-sm"
>
  <FileProcessingResults
    {mergedGroupIds}
    {standaloneGroups}
    {passthroughPaths}
    {allFilePaths}
    {noGroupsFound}
  />

<ul class="space-y-3">
  {#each allFilePaths as path}
    <li class="rounded border p-3">
      <div class="mb-2 text-sm">{path}</div>

      <div class="flex items-center gap-2">
        <input
          class="h-9 w-full rounded border px-2 text-sm"
          placeholder="Enter table name"
          bind:value={tableNames[path]}
        />

        <Button
          class="shrink-0"
          onclick={() => createTableFromPath(path)}
          disabled={creatingByPath[path]}
        >
          {creatingByPath[path] ? "Creating..." : "Create table"}
        </Button>
      </div>
    </li>
  {/each}
</ul>

      <Button class="w-full" onclick={() => findGroupsWithDb(allFilePaths, mergedGroupIds)} disabled={ isProcessing}>
      {isProcessing ? "Processing..." : "Process"}
    </Button>
</Card>
