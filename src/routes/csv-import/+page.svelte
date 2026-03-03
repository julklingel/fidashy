<script lang="ts">
  import CsvFileGroupingResults from "$lib/components/CsvFileGroupingResults.svelte";
  import CsvFileSelectionAndGroupingCard from "$lib/components/CsvFileSelectionAndGrouping.svelte";
  import { Card } from "$lib/components/ui/card/index.js";
  import type {
    GroupResolutionSummary,
    GroupWithDuplicates,
    StandaloneGroup,
  } from "$lib/components/csv-types";

  let groupedPaths = $state<GroupWithDuplicates[]>([]);
  let soloPaths = $state<string[]>([]);
  const groupedPathsCount = $derived(groupedPaths.length);
  let hasResolvedGroups = $state(false);
  let mergedGroupIds = $state<string[]>([]);
  let standaloneGroups = $state<StandaloneGroup[]>([]);

  function onAllGroupsResolved(summary: GroupResolutionSummary) {
    hasResolvedGroups = true;
    mergedGroupIds = summary.mergedGroupIds;
    standaloneGroups = summary.standaloneGroups;
  }

  $effect(() => {
    if (groupedPathsCount === 0) {
      hasResolvedGroups = false;
      mergedGroupIds = [];
      standaloneGroups = [];
    }
  });
  
</script>

<section
  class="flex min-h-screen flex-col items-center justify-center bg-background px-4 py-8"
>
  {#if hasResolvedGroups}
    <Card
      className="w-full max-w-2xl rounded-lg border bg-card p-6 text-card-foreground shadow-sm"
    >
      <div class="space-y-2">
        <h2 class="text-xl font-semibold tracking-tight">Next step</h2>

        <div>
          <h3 class="text-sm font-semibold">Merged groups (cached DataFrames)</h3>
          <ul class="list-disc pl-5 text-sm text-muted-foreground">
            {#if mergedGroupIds.length === 0}
              <li>None</li>
            {:else}
              {#each mergedGroupIds as groupId}
                <li>{groupId}</li>
              {/each}
            {/if}
          </ul>
        </div>

        <div>
          <h3 class="text-sm font-semibold">Standalone groups (paths per group)</h3>
          {#if standaloneGroups.length === 0}
            <ul class="list-disc pl-5 text-sm text-muted-foreground"><li>None</li></ul>
          {:else}
            <ul class="space-y-2 text-sm text-muted-foreground">
              {#each standaloneGroups as group}
                <li>
                  <p class="font-medium text-foreground">{group.group_id}</p>
                  <ul class="list-disc pl-5">
                    {#each group.paths as path}
                      <li>{path}</li>
                    {/each}
                  </ul>
                </li>
              {/each}
            </ul>
          {/if}
        </div>

        <div>
          <h3 class="text-sm font-semibold">Solo files from first step (never grouped)</h3>
          <ul class="list-disc pl-5 text-sm text-muted-foreground">
            {#if soloPaths.length === 0}
              <li>None</li>
            {:else}
              {#each soloPaths as path}
                <li>{path}</li>
              {/each}
            {/if}
          </ul>
        </div>
      </div>
    </Card>
  {:else if groupedPathsCount < 1}
    <CsvFileSelectionAndGroupingCard bind:groupedPaths bind:soloPaths />
  {:else}
    <CsvFileGroupingResults {groupedPaths} {onAllGroupsResolved} />
  {/if}
</section>
