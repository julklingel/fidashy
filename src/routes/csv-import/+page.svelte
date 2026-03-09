<script lang="ts">
  import CsvGroupResolutionSummary from "$lib/components/CsvGroupResolutionSummary.svelte";
  import CsvFileGroupingResults from "$lib/components/CsvFileGroupingResults.svelte";
  import CsvImportStepper from "$lib/components/CsvImportStepper.svelte";
  import CsvFileSelectionAndGroupingCard from "$lib/components/CsvFileSelectionAndGrouping.svelte";
  import type {
    GroupResolutionSummary,
    GroupWithDuplicates,
  } from "$lib/components/csv-types";

  let groupedPaths = $state<GroupWithDuplicates[]>([]);
  let soloPaths = $state<string[]>([]);
  let noGroupsFound = $state(false);
  const groupedPathsCount = $derived(groupedPaths.length);
  let resolutionSummary = $state<GroupResolutionSummary | null>(null);
  const emptyResolutionSummary: GroupResolutionSummary = {
    mergedGroupIds: [],
    standaloneGroups: [],
  };

  const steps = [
    {
      id: 1,
      title: "Select files",
      description: "Choose the CSV files to inspect.",
    },
    {
      id: 2,
      title: "Resolve groups",
      description: "Merge each detected group or keep it standalone.",
    },
    {
      id: 3,
      title: "Resolve groups with DB",
      description: "Ingest the tables into the database.",
    },
  ];

  const displayedSummary = $derived.by(() => {
    if (resolutionSummary) return resolutionSummary;
    if (noGroupsFound) return emptyResolutionSummary;
    return null;
  });

  const currentStep = $derived.by(() => {
    if (displayedSummary) return 3;
    if (groupedPathsCount > 0) return 2;
    return 1;
  });

  function onAllGroupsResolved(summary: GroupResolutionSummary) {
    resolutionSummary = summary;
  }

  $effect(() => {
    if (groupedPathsCount === 0 && !noGroupsFound) {
      resolutionSummary = null;
    }
  });
</script>

<section
  class="flex min-h-screen flex-col items-center justify-center bg-background px-4 py-8"
>
  <div class="flex w-full max-w-2xl flex-col items-center gap-6">
    <CsvImportStepper {steps} {currentStep} />

    {#if displayedSummary}
      <CsvGroupResolutionSummary summary={displayedSummary} {soloPaths} {noGroupsFound} />
    {:else if groupedPathsCount == 0}
      <CsvFileSelectionAndGroupingCard bind:groupedPaths bind:soloPaths bind:noGroupsFound />
    {:else}
      <CsvFileGroupingResults {groupedPaths} {onAllGroupsResolved} />
    {/if}
  </div>
</section>
