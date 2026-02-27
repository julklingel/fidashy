<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
	import { open } from "@tauri-apps/plugin-dialog";
	import { Card } from "$lib/components/ui/card/index.js";
	import { Button } from "$lib/components/ui/button/index.js";


	type Props = {
    groupedPaths: string[][];
  };

  let { groupedPaths }: Props = $props();
  const groupedPathsCount = $derived(groupedPaths.length);


</script>


<Card>
		{#if groupedPathsCount > 0}
			<div class="space-y-2 rounded-md border p-3">
				<h2 class="class=text-xl font-semibold tracking-tight">Grouping results ({groupedPathsCount} groups)</h2>
				<ul class="space-y-2">
					{#each groupedPaths as group, index}
                    {#if group.length > 1}
						<li class="rounded-sm border px-2 py-1 text-xs">
							<div class="mb-1 font-medium">Group {index + 1} ({group.length} file(s))</div>
							<ul class="space-y-0.5 text-muted-foreground">
								{#each group as path}
									<li class="truncate">{path}</li>
								{/each}
							</ul>
						</li>
                        {/if}
					{/each}
				</ul>
			</div>
		{/if}
</Card>
