<script lang="ts">
  import CsvFileGroupingResults from "$lib/components/CsvFileGroupingResults.svelte";
import CsvFileSelectionAndGroupingCard from "$lib/components/CsvFileSelectionAndGrouping.svelte";
  import type { GroupWithDuplicates } from "$lib/components/csv-types";

  let groupedPaths = $state<GroupWithDuplicates[]>([]);
  const groupedPathsCount = $derived(groupedPaths.length);



  

</script>

<section class="flex min-h-screen flex-col items-center justify-center bg-background px-4 py-8">
  

    {#if groupedPathsCount < 1}
    <CsvFileSelectionAndGroupingCard bind:groupedPaths />
    {:else}
    <CsvFileGroupingResults {groupedPaths} /> 
    {/if}
    


</section>
