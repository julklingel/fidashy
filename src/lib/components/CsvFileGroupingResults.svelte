<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
	import { Card } from "$lib/components/ui/card/index.js";
	import type {
		DeduplicateGroupResult,
		GroupResolutionSummary,
		GroupWithDuplicates,
		StandaloneGroup,
		SkipMergeGroupResult,
	} from "$lib/components/csv-types";
	import { toast } from "$lib/components/ui/sonner/sonner.js";
	import Button from "./ui/button/button.svelte";

	type GroupDecision = "merged" | "standalone";

	type Props = {
		groupedPaths: GroupWithDuplicates[];
		onAllGroupsResolved?: (summary: GroupResolutionSummary) => void;
	};

	let { groupedPaths, onAllGroupsResolved }: Props = $props();
	const groupedPathsCount = $derived(groupedPaths.length);
	let activeGroupId = $state<string | null>(null);
	let decisionsByGroupId = $state<Record<string, GroupDecision>>({});
	let standalonePathsByGroupId = $state<Record<string, string[]>>({});

	$effect(() => {
		groupedPaths;
		decisionsByGroupId = {};
		standalonePathsByGroupId = {};
	});

	function isGroupResolved(groupId: string) {
		return decisionsByGroupId[groupId] !== undefined;
	}

	function maybeCompleteFlow() {
		if (Object.keys(decisionsByGroupId).length !== groupedPathsCount) return;

		const mergedGroupIds = Object.entries(decisionsByGroupId)
			.filter(([, decision]) => decision === "merged")
			.map(([groupId]) => groupId);

		const skippedGroupIds = Object.entries(decisionsByGroupId)
			.filter(([, decision]) => decision === "standalone")
			.map(([groupId]) => groupId);

		const standaloneGroups: StandaloneGroup[] = skippedGroupIds.map((groupId) => ({
			group_id: groupId,
			paths: standalonePathsByGroupId[groupId] ?? [],
		}));

		onAllGroupsResolved?.({
			mergedGroupIds,
			standaloneGroups,
		});
	}

	async function mergeGroup(groupId: string) {
		if (activeGroupId || isGroupResolved(groupId)) return;
		activeGroupId = groupId;
		try {
			const result = await invoke<DeduplicateGroupResult>("deduplicate_cached_group", { groupId });
			decisionsByGroupId = { ...decisionsByGroupId, [groupId]: "merged" };
			const completedCount = Object.keys(decisionsByGroupId).length;
			toast.success(
				`${result.message} (${result.rows_before} → ${result.rows_after} rows across ${result.source_file_count} files). ${completedCount}/${groupedPathsCount} groups completed.`
			);
			maybeCompleteFlow();
		} catch (error) {
			const message = error instanceof Error ? error.message : String(error);
			toast.error(`Deduplication failed for group '${groupId}': ${message}`);
		} finally {
			activeGroupId = null;
		}
	}

	async function keepGroupStandalone(groupId: string) {
		if (activeGroupId || isGroupResolved(groupId)) return;
		activeGroupId = groupId;
		try {
			const result = await invoke<SkipMergeGroupResult>("skip_merge_cached_group", { groupId });
			decisionsByGroupId = { ...decisionsByGroupId, [groupId]: "standalone" };
			standalonePathsByGroupId = {
				...standalonePathsByGroupId,
				[groupId]: result.standalone_paths,
			};
			const completedCount = Object.keys(decisionsByGroupId).length;
			toast.success(`${result.message}. ${completedCount}/${groupedPathsCount} groups completed.`);
			maybeCompleteFlow();
		} catch (error) {
			const message = error instanceof Error ? error.message : String(error);
			toast.error(`Failed to keep group '${groupId}' standalone: ${message}`);
		} finally {
			activeGroupId = null;
		}
	}

	
</script>

<Card>
	{#if groupedPathsCount > 0}
		<div class="space-y-2 rounded-md border p-3">
			<h2 class="text-xl font-semibold tracking-tight">
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
							<div class="flex items-center gap-2">
								{#if decisionsByGroupId[group.group_id] === "merged"}
									<span class="text-xs font-medium text-muted-foreground">Merged</span>
								{:else if decisionsByGroupId[group.group_id] === "standalone"}
									<span class="text-xs font-medium text-muted-foreground">Kept standalone</span>
								{:else}
									<Button
										size="sm"
										onclick={() => mergeGroup(group.group_id)}
										disabled={activeGroupId !== null}
									>
										{#if activeGroupId === group.group_id}
											Processing...
										{:else}
											Merge
										{/if}
									</Button>
									<Button
										size="sm"
										variant="outline"
										onclick={() => keepGroupStandalone(group.group_id)}
										disabled={activeGroupId !== null}
									>
										{#if activeGroupId === group.group_id}
											Processing...
										{:else}
											Keep standalone
										{/if}
									</Button>
								{/if}
							</div>
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
