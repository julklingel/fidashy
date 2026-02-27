<script lang="ts">
  import CsvFileGroupingResults from "$lib/components/CsvFileGroupingResults.svelte";
import CsvFileSelectionAndGroupingCard from "$lib/components/CsvFileSelectionAndGrouping.svelte";

  let groupedPaths = $state<string[][]>([]);
  const groupedPathsCount = $derived(groupedPaths.length);
  const hasGroupBiggerThanTwo = $derived(
    groupedPaths.some((group) => group.length > 1)
  );


  

</script>

<section class="flex min-h-screen flex-col items-center justify-center bg-background px-4 py-8">
  

    {#if !hasGroupBiggerThanTwo}
    <CsvFileSelectionAndGroupingCard bind:groupedPaths />
    {:else}
    <CsvFileGroupingResults {groupedPaths} /> 
    {/if}
    


</section>
