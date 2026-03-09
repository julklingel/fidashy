<script lang="ts">
  import type { GroupResolutionSummary } from "$lib/components/csv-types";
  import {
    Accordion,
    AccordionContent,
    AccordionItem,
    AccordionTrigger,
  } from "$lib/components/ui/accordion/index.js";
  import { Card } from "$lib/components/ui/card/index.js";

    let { mergedGroupIds, standaloneGroups, passthroughPaths, allFilePaths, noGroupsFound } = $props();


</script>


  <div class="space-y-4">
    <div class="space-y-1">
      <h2 class="text-xl font-semibold tracking-tight">Resolve tables against the database</h2>
      <p class="text-sm text-muted-foreground">
        {#if noGroupsFound}
          No matching CSV groups were found, so only files that passed through without grouping will continue.
        {:else}
          Review the grouping outcome before continuing with the database step.
        {/if}
      </p>
    </div>

    <div class="rounded-md border p-3 text-sm text-muted-foreground">
      <p>
        Merged groups: {mergedGroupIds.length} · Standalone groups: {standaloneGroups.length} · Passed through without grouping:
        {passthroughPaths.length}
      </p>
      <p>Total files continuing from grouping: {allFilePaths.length}</p>
    </div>

    <Accordion type="single">
      <AccordionItem value="group-resolution-summary">
        <AccordionTrigger>Group resolving summary</AccordionTrigger>
        <AccordionContent>
          <div class="space-y-4 text-sm">
            <div>
              <h3 class="font-semibold">Merged groups</h3>
              {#if mergedGroupIds.length === 0}
                <p class="text-muted-foreground">No groups were merged.</p>
              {:else}
                <ul class="list-disc space-y-1 pl-5 text-muted-foreground">
                  {#each mergedGroupIds as groupId}
                    <li>{groupId}</li>
                  {/each}
                </ul>
              {/if}
            </div>

            <div>
              <h3 class="font-semibold">Treated as standalone</h3>
              {#if standaloneGroups.length === 0}
                <p class="text-muted-foreground">No grouped files were kept standalone.</p>
              {:else}
                <ul class="space-y-3 text-muted-foreground">
                  {#each standaloneGroups as group}
                    <li>
                      <p class="font-medium text-foreground">{group.group_id}</p>
                      <ul class="list-disc space-y-1 pl-5">
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
              <h3 class="font-semibold">Passed through without grouping</h3>
              {#if passthroughPaths.length === 0}
                <p class="text-muted-foreground">No files skipped grouping.</p>
              {:else}
                <ul class="list-disc space-y-1 pl-5 text-muted-foreground">
                  {#each passthroughPaths as path}
                    <li>{path}</li>
                  {/each}
                </ul>
              {/if}
            </div>
          </div>
        </AccordionContent>
      </AccordionItem>
    </Accordion>
  </div>


