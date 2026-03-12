<script lang="ts">
  import CsvMergeGroupsWithDb from "$lib/components/CsvMergeGroupsWithDb.svelte";
  import CsvFileGroupingProposalCard from "$lib/components/CsvFileGroupingProposal.svelte";
  import CsvImportStepper from "$lib/components/CsvImportStepper.svelte";
  import CsvFileSelectionCard from "$lib/components/CsvFileSelection.svelte";
  import type {
    GroupProposal,
    GroupResolutionSummary,
  } from "$lib/components/csv-types";

  let groupedPaths = $state<GroupProposal[]>([]);
  let soloPaths = $state<string[]>([]);
  let noGroupsFound = $state(false);
  const groupedPathsCount = $derived(groupedPaths.length);
  const hasGroupingWorkspace = $derived(groupedPathsCount > 0 || soloPaths.length > 0 || noGroupsFound);
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

  const displayedSummary = $derived.by(() => resolutionSummary);

  const currentStep = $derived.by(() => {
    if (displayedSummary) return 3;
    if (hasGroupingWorkspace) return 2;
    return 1;
  });

  function onAllGroupsResolved(summary: GroupResolutionSummary) {
    resolutionSummary = summary;
  }

  $effect(() => {
    if (!hasGroupingWorkspace) {
      resolutionSummary = null;
    }
  });
</script>

<section
  class="flex min-h-screen flex-col items-center justify-center bg-background px-4 py-8"
>
  <h1 class=" text-3xl font-semibold p-4">CSV - Database Import</h1>
  <div class="flex w-full max-w-5xl flex-col items-center gap-6">
    <CsvImportStepper {steps} {currentStep} />

    {#if displayedSummary}
      <CsvMergeGroupsWithDb summary={displayedSummary} />
    {:else if !hasGroupingWorkspace}
      <CsvFileSelectionCard bind:groupedPaths bind:soloPaths bind:noGroupsFound />
    {:else}
      <CsvFileGroupingProposalCard {groupedPaths} {soloPaths} {onAllGroupsResolved} />
    {/if}
  </div>
</section>
