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

  <ul>
    {#each allFilePaths as path}
      <li>
        {path}
      </li>
    {/each}
  </ul>

      <Button class="w-full" onclick={() => findGroupsWithDb(allFilePaths, mergedGroupIds)} disabled={ isProcessing}>
      {isProcessing ? "Processing..." : "Process"}
    </Button>
</Card>
