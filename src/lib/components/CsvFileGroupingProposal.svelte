<script lang="ts">
	import { Card } from "$lib/components/ui/card/index.js";
	import type {
		GroupProposal,
		GroupResolutionSummary,
		MergedGroup,
		StandaloneGroup,
	} from "$lib/components/csv-types";
	import { toast } from "$lib/components/ui/sonner/sonner.js";
	import Button from "./ui/button/button.svelte";

	type DraggedFile = {
		path: string;
		sourceGroupId: string | null;
	};

	type Props = {
		groupedPaths: GroupProposal[];
		soloPaths?: string[];
		onAllGroupsResolved?: (summary: GroupResolutionSummary) => void;
	};

	let { groupedPaths, soloPaths = [], onAllGroupsResolved }: Props = $props();
	let editableGroups = $state<GroupProposal[]>([]);
	let standalonePaths = $state<string[]>([]);
	let mergedGroups = $state<MergedGroup[]>([]);
	let mergedGroupIds = $state<string[]>([]);
	let activeGroupId = $state<string | null>(null);
	let draggedFile = $state<DraggedFile | null>(null);
	let nextCustomGroupNumber = $state(1);

	const groupedPathsCount = $derived(editableGroups.length);
	const mergedGroupsCount = $derived(mergedGroups.length);
	const standaloneCount = $derived(standalonePaths.length);
	const pendingGroupedFilesCount = $derived(
		editableGroups.reduce((total, group) => total + group.paths.length, 0)
	);

	$effect(() => {
		const assignedPaths = new Set<string>();
		const normalizedGroups = groupedPaths
			.map((group, index) => {
				const uniqueGroupPaths = Array.from(new Set(group.paths)).filter((path) => {
					if (!path || assignedPaths.has(path)) return false;
					assignedPaths.add(path);
					return true;
				});

				return {
					group_id: group.group_id || `group-${index + 1}`,
					paths: uniqueGroupPaths,
				};
			})
			.filter((group) => group.paths.length > 0);

		editableGroups = normalizedGroups;
		standalonePaths = Array.from(new Set(soloPaths)).filter(
			(path) => path && !assignedPaths.has(path)
		);
		mergedGroups = [];
		mergedGroupIds = [];
		activeGroupId = null;
		draggedFile = null;
		nextCustomGroupNumber = normalizedGroups.length + 1;
	});

	function uniquePaths(paths: string[]) {
		return Array.from(new Set(paths));
	}

	function createNewGroup() {
		const groupId = `custom-group-${nextCustomGroupNumber}`;
		nextCustomGroupNumber += 1;
		editableGroups = [...editableGroups, { group_id: groupId, paths: [] }];
	}

	function deleteGroup(groupId: string) {
		const groupToDelete = editableGroups.find((group) => group.group_id === groupId);
		if (!groupToDelete) return;

		standalonePaths = uniquePaths([...standalonePaths, ...groupToDelete.paths]);
		editableGroups = editableGroups.filter((group) => group.group_id !== groupId);
	}

	function movePath(path: string, targetGroupId: string | null) {
		editableGroups = editableGroups.map((group) => ({
			...group,
			paths: group.paths.filter((entry) => entry !== path),
		}));
		standalonePaths = standalonePaths.filter((entry) => entry !== path);

		if (targetGroupId === null) {
			standalonePaths = uniquePaths([...standalonePaths, path]);
			return;
		}

		editableGroups = editableGroups.map((group) =>
			group.group_id === targetGroupId
				? { ...group, paths: uniquePaths([...group.paths, path]) }
				: group
		);
	}

	function startDragging(path: string, sourceGroupId: string | null, event: DragEvent) {
		draggedFile = { path, sourceGroupId };
		event.dataTransfer?.setData("text/plain", JSON.stringify({ path, sourceGroupId }));
		event.dataTransfer?.setData("application/fidashy-path", path);
		event.dataTransfer?.setData("application/fidashy-source-group", sourceGroupId ?? "");
		event.dataTransfer?.setDragImage?.(event.currentTarget as Element, 12, 12);
	}

	function readDraggedFile(event: DragEvent) {
		const rawPayload = event.dataTransfer?.getData("text/plain");
		if (rawPayload) {
			try {
				const payload = JSON.parse(rawPayload) as DraggedFile;
				if (payload.path) return payload;
			} catch {
				// Ignore malformed payloads and fall back to in-memory state.
			}
		}

		return draggedFile;
	}

	function dropIntoGroup(event: DragEvent, targetGroupId: string) {
		event.preventDefault();
		const payload = readDraggedFile(event);
		if (!payload) return;

		movePath(payload.path, targetGroupId);
		draggedFile = null;
	}

	function dropIntoStandalone(event: DragEvent) {
		event.preventDefault();
		const payload = readDraggedFile(event);
		if (!payload) return;

		movePath(payload.path, null);
		draggedFile = null;
	}

	function handleDragEnd() {
		draggedFile = null;
	}

	function groupLabel(index: number) {
		return `Group ${index + 1}`;
	}

	function toFileName(path: string) {
		const normalized = path.replaceAll("\\", "/");
		const lastSlashIndex = normalized.lastIndexOf("/");
		return lastSlashIndex >= 0 ? normalized.slice(lastSlashIndex + 1) : normalized;
	}

	function finishGrouping() {
		const groupsWithFiles = editableGroups.filter((group) => group.paths.length > 0);
		if (groupsWithFiles.length > 0) {
			toast.error("Merge or delete the remaining groups before continuing.");
			return;
		}

		const standaloneGroups: StandaloneGroup[] = standalonePaths.map((path, index) => ({
			group_id: `standalone-${index + 1}`,
			paths: [path],
		}));

		onAllGroupsResolved?.({
			mergedGroups,
			mergedGroupIds,
			standaloneGroups,
		});
	}

	async function mergeGroup(groupId: string) {
		if (activeGroupId) return;

		const group = editableGroups.find((entry) => entry.group_id === groupId);
		if (!group) return;
		if (group.paths.length === 0) {
			toast.error("Add at least one file to the group before merging.");
			return;
		}

		activeGroupId = groupId;
		try {
			mergedGroupIds = [...mergedGroupIds, groupId];
			mergedGroups = [...mergedGroups, { group_id: groupId, paths: [...group.paths] }];
			editableGroups = editableGroups.filter((entry) => entry.group_id !== groupId);
			toast.success(`Queued ${group.paths.length} file(s) from ${groupId} for DB resolution.`);
		} finally {
			activeGroupId = null;
		}
	}
</script>

<Card className="w-full max-w-5xl rounded-lg border bg-card p-6 text-card-foreground shadow-sm">
	<div class="space-y-6">
		<div class="space-y-2">
			<h2 class="text-xl font-semibold tracking-tight">Adjust grouping before import</h2>
			<p class="text-sm text-muted-foreground">
				Drag files between groups, create new groups, or leave files in the standalone list.
				Merged groups stay in the frontend state and move to the DB resolution step.
			</p>
		</div>

		<div class="grid gap-3 text-sm md:grid-cols-3">
			<div class="rounded-md border px-3 py-2">
				<div class="text-muted-foreground">Editable groups</div>
				<div class="text-lg font-semibold">{groupedPathsCount}</div>
			</div>
			<div class="rounded-md border px-3 py-2">
				<div class="text-muted-foreground">Standalone files</div>
				<div class="text-lg font-semibold">{standaloneCount}</div>
			</div>
			<div class="rounded-md border px-3 py-2">
				<div class="text-muted-foreground">Already merged</div>
				<div class="text-lg font-semibold">{mergedGroupsCount}</div>
			</div>
		</div>

		<div
			class="space-y-3 rounded-lg border border-dashed p-4"
			role="region"
			aria-label="Standalone files drop zone"
			ondragover={(event) => event.preventDefault()}
			ondrop={dropIntoStandalone}
		>
			<div class="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
				<div>
					<h3 class="font-semibold">Standalone files</h3>
					<p class="text-sm text-muted-foreground">
						Files here will continue without being part of a merge group.
					</p>
				</div>
				<Button onclick={finishGrouping} disabled={activeGroupId !== null}>
					{standaloneCount > 0 ? "Submit standalone files" : "Continue"}
				</Button>
			</div>

			{#if standalonePaths.length > 0}
				<ul class="grid gap-2 md:grid-cols-2">
					{#each standalonePaths as path (path)}
						<li
							class="flex items-center justify-between gap-3 rounded-md border bg-background px-3 py-2 text-sm"
							draggable="true"
							ondragstart={(event) => startDragging(path, null, event)}
							ondragend={handleDragEnd}
						>
							<div class="min-w-0 flex-1">
								<div class="truncate font-medium">{toFileName(path)}</div>
								<div class="truncate text-xs text-muted-foreground">{path}</div>
							</div>
							<span class="shrink-0 text-xs text-muted-foreground">Drag to group</span>
						</li>
					{/each}
				</ul>
			{:else}
				<p class="rounded-md border bg-background px-3 py-6 text-sm text-muted-foreground">
					Drop files here to keep them standalone.
				</p>
			{/if}
		</div>

		<div class="space-y-3">
			<div class="flex items-center justify-between gap-3">
				<div>
					<h3 class="font-semibold">Groups</h3>
					<p class="text-sm text-muted-foreground">
						Create groups manually or refine the proposed ones before merging.
					</p>
				</div>
				<Button variant="outline" onclick={createNewGroup} disabled={activeGroupId !== null}>
					Add group
				</Button>
			</div>

			{#if editableGroups.length > 0}
				<div class="grid gap-4 lg:grid-cols-2">
					{#each editableGroups as group, index (group.group_id)}
						<div
							class="space-y-3 rounded-lg border p-4"
							role="region"
							aria-label={`Drop zone for ${groupLabel(index)}`}
							ondragover={(event) => event.preventDefault()}
							ondrop={(event) => dropIntoGroup(event, group.group_id)}
						>
							<div class="flex items-start justify-between gap-3">
								<div>
									<h4 class="font-semibold">{groupLabel(index)}</h4>
									<p class="text-xs text-muted-foreground">{group.paths.length} file(s)</p>
								</div>
								<div class="flex gap-2">
									<Button
										size="sm"
										onclick={() => mergeGroup(group.group_id)}
										disabled={activeGroupId !== null || group.paths.length === 0}
									>
										{#if activeGroupId === group.group_id}
											Merging...
										{:else}
											Merge
										{/if}
									</Button>
									<Button
										size="sm"
										variant="outline"
										onclick={() => deleteGroup(group.group_id)}
										disabled={activeGroupId !== null}
									>
										Delete group
									</Button>
								</div>
							</div>

							{#if group.paths.length > 0}
								<ul class="space-y-2">
									{#each group.paths as path (path)}
										<li
											class="flex items-center justify-between gap-3 rounded-md border bg-background px-3 py-2 text-sm"
											draggable="true"
											ondragstart={(event) => startDragging(path, group.group_id, event)}
											ondragend={handleDragEnd}
										>
											<div class="min-w-0 flex-1">
												<div class="truncate font-medium">{toFileName(path)}</div>
												<div class="truncate text-xs text-muted-foreground">{path}</div>
											</div>
											<Button
												size="sm"
												variant="ghost"
												onclick={() => movePath(path, null)}
												disabled={activeGroupId !== null}
											>
												Make standalone
											</Button>
										</li>
									{/each}
								</ul>
							{:else}
								<p class="rounded-md border border-dashed px-3 py-6 text-sm text-muted-foreground">
									Drop files here.
								</p>
							{/if}
						</div>
					{/each}
				</div>
			{:else}
				<p class="rounded-lg border px-3 py-6 text-sm text-muted-foreground">
					No editable groups left. Submit the standalone files to continue.
				</p>
			{/if}
		</div>

		{#if mergedGroups.length > 0}
			<div class="space-y-3 rounded-lg border p-4">
				<h3 class="font-semibold">Ready for DB resolution</h3>
				<ul class="space-y-2 text-sm text-muted-foreground">
					{#each mergedGroups as group}
						<li class="rounded-md border bg-background px-3 py-2">
							<div class="font-medium text-foreground">{group.group_id}</div>
							<div>{group.paths.length} file(s) queued for DB resolution</div>
						</li>
					{/each}
				</ul>
			</div>
		{/if}

		{#if pendingGroupedFilesCount > 0}
			<p class="text-sm text-muted-foreground">
				There are still {pendingGroupedFilesCount} file(s) inside editable groups. Merge those groups or delete the groups before continuing.
			</p>
		{/if}
	</div>
</Card>
