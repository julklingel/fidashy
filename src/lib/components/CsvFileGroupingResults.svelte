<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
	import { Card } from "$lib/components/ui/card/index.js";
	import type { GroupWithDuplicates } from "$lib/components/csv-types";
	import { toast } from "$lib/components/ui/sonner/sonner.js";
	import Button from "./ui/button/button.svelte";

	type Props = {
		groupedPaths: GroupWithDuplicates[];
	};

	let { groupedPaths }: Props = $props();
	const groupedPathsCount = $derived(groupedPaths.length);
	let isMerging = $state(false);

	async function mergeGroup(paths: string[]) {
		if (isMerging) return;
		isMerging = true;
		try {
			const result = await invoke<string>("merge_csv", { paths });
			if (result === "ok") {
				toast.success("Merge completed.");
				return;
			}
			toast.success(`Merge finished: ${result}`);
		} catch (error) {
			const message = error instanceof Error ? error.message : String(error);
			toast.error(`Merge failed: ${message}`);
		} finally {
			isMerging = false;
		}
	}

	
</script>

<Card>
	{#if groupedPathsCount > 0}
		<div class="space-y-2 rounded-md border p-3">
			<h2 class="class=text-xl font-semibold tracking-tight">
				Grouping results ({groupedPathsCount} groups)
			</h2>

			<ul class="space-y-2">
				{#each groupedPaths as group, index}
					<li class="rounded-sm border px-2 py-2 text-xs">
						<div class="mb-1 flex items-start justify-between gap-2">
							<div class="font-medium">
								<span class="font-semibold">Group {index + 1}</span>
								- {group.paths.length} files - {group.duplicate_count}
								duplicates / {group.total_entries} total entries
							</div>
							<Button size="sm" onclick={() => mergeGroup(group.paths)} disabled={isMerging}>
								{isMerging ? "Merging..." : "Merge"}
							</Button>
						</div>
						<ul class="space-y-0.5 text-muted-foreground">
							{#each group.paths as path}
								<li class="truncate">{path}</li>
							{/each}
						</ul>
					</li>
				{/each}
			</ul>
		</div>
	{/if}
</Card>
