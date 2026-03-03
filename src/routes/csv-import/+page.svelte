<script lang="ts">
  import CsvFileGroupingResults from "$lib/components/CsvFileGroupingResults.svelte";
  import CsvFileSelectionAndGroupingCard from "$lib/components/CsvFileSelectionAndGrouping.svelte";
  import { Card } from "$lib/components/ui/card/index.js";
  import type {
    DeduplicateGroupResult,
    GroupWithDuplicates,
  } from "$lib/components/csv-types";

  let groupedPaths = $state<GroupWithDuplicates[]>([]);
  const groupedPathsCount = $derived(groupedPaths.length);
  let hasDeduplicated = $state(false);

  function onDeduplicateCompleted(_result: DeduplicateGroupResult) {
    hasDeduplicated = true;
  }

  $effect(() => {
    if (groupedPathsCount === 0) {
      hasDeduplicated = false;
    }
  });
  
</script>

<section
  class="flex min-h-screen flex-col items-center justify-center bg-background px-4 py-8"
>
  {#if hasDeduplicated}
    <Card
      className="w-full max-w-2xl rounded-lg border bg-card p-6 text-card-foreground shadow-sm"
    >
      <div class="space-y-2">
        <h2 class="text-xl font-semibold tracking-tight">Next step</h2>
      </div>
    </Card>
  {:else if groupedPathsCount < 1}
    <CsvFileSelectionAndGroupingCard bind:groupedPaths />
  {:else}
    <CsvFileGroupingResults {groupedPaths} {onDeduplicateCompleted} />
  {/if}
</section>
