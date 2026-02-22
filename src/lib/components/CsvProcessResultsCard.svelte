<script lang="ts">
  import type { MergeState, SchemaGroup } from "./csv-types";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Card } from "$lib/components/ui/card";
  import { Progress } from "$lib/components/ui/progress";
  import {
    Accordion,
    AccordionItem,
    AccordionTrigger,
    AccordionContent,
  } from "$lib/components/ui/accordion";

  let {
    groups,
    mergeState,
    onMerge,
    onCancel,
    onCancelStep,
  }: {
    groups: SchemaGroup[];
    mergeState: MergeState;
    onMerge: (groupId: string) => void;
    onCancel: (groupId: string) => void;
    onCancelStep: () => void;
  } = $props();

  const totalGroups = $derived(groups.length);
  const totalDuplicates = $derived(groups.reduce((sum, group) => sum + group.duplicateRows, 0));

  const getGroupState = (groupId: string) => mergeState.groups[groupId] ?? { isMerging: false, status: "idle", progress: 0 };
</script>

{#snippet fileNames(group: SchemaGroup)}
  <p class="text-muted-foreground wrap-break-word">Files: {group.files.map((file) => file.name).join(", ")}</p>
{/snippet}

{#snippet headerList(group: SchemaGroup)}
  <p class="text-muted-foreground wrap-break-word">Headers: {group.headers.join(", ") || "No headers found"}</p>
{/snippet}

{#snippet groupRow(group: SchemaGroup)}
  {@const groupState = getGroupState(group.id)}
  <li class="space-y-2 rounded-md border px-3 py-3">
    {@render fileNames(group)}

    <div class="flex items-center gap-2">
      {#if groupState.isMerging}
        <div class="min-w-32 flex-1">
          <Progress value={groupState.progress} class="h-2" />
        </div>
      {:else}
        <Button type="button" class="h-8 px-3 text-xs" onclick={() => onMerge(group.id)}>Merge Files</Button>
      {/if}
      <Button type="button" variant="outline" class="h-8 px-3 text-xs" onclick={() => onCancel(group.id)}>Cancel</Button>
    </div>

    {#if group.files.length > 1}
      <Accordion type="single" class="pt-1">
        <AccordionItem>
          <AccordionTrigger>Summary</AccordionTrigger>
          <AccordionContent>
            <div class="space-y-2">
              <div class="space-y-1">
                <p class="text-xs font-medium">Duplicate rows</p>
                <p class="text-muted-foreground">{group.duplicateRows}</p>
              </div>
              <div class="border-t pt-2 space-y-1">
                <p class="text-xs font-medium">Identical headers</p>
                {@render headerList(group)}
              </div>
            </div>
          </AccordionContent>
        </AccordionItem>
      </Accordion>
    {/if}
  </li>
{/snippet}

<Card className="w-full max-w-md space-y-4 rounded-lg border bg-card p-6 text-card-foreground shadow">
  <div class="space-y-3">
    <div class="flex items-center justify-between gap-2">
      <p class="text-sm font-medium">Process file result cards</p>
      <Button type="button" variant="outline" class="h-8 px-3 text-xs" onclick={onCancelStep}>Cancel step</Button>
    </div>
    <p class="text-xs text-muted-foreground">Schema groups found: {totalGroups}</p>
    <p class="text-xs text-muted-foreground">Total duplicates found: {totalDuplicates}</p>

    <ul class="space-y-2 text-sm">
      {#each groups as group}
        {@render groupRow(group)}
      {/each}
    </ul>
  </div>
</Card>
