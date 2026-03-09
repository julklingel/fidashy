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
  const groupedPathsCount = $derived(groupedPaths.length);
  let resolutionSummary = $state<GroupResolutionSummary | null>(null);

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

  const currentStep = $derived.by(() => {
    if (resolutionSummary) return 3;
    if (groupedPathsCount > 0) return 2;
    return 1;
  });

  function onAllGroupsResolved(summary: GroupResolutionSummary) {
    resolutionSummary = summary;
  }

  $effect(() => {
    if (groupedPathsCount === 0) {
      resolutionSummary = null;
    }
  });
</script>

<section
  class="flex min-h-screen flex-col items-center justify-center bg-background px-4 py-8"
>
  <div class="flex w-full max-w-2xl flex-col items-center gap-6">
    <CsvImportStepper {steps} {currentStep} />

    {#if resolutionSummary}
      <CsvGroupResolutionSummary summary={resolutionSummary} {soloPaths} />
    {:else if groupedPathsCount < 1}
      <CsvFileSelectionAndGroupingCard bind:groupedPaths bind:soloPaths />
    {:else}
      <CsvFileGroupingResults {groupedPaths} {onAllGroupsResolved} />
    {/if}
  </div>
</section>
