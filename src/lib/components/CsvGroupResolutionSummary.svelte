<script lang="ts">
	import { Card } from "$lib/components/ui/card/index.js";
	import type { GroupResolutionSummary } from "$lib/components/csv-types";

	type Props = {
		summary: GroupResolutionSummary;
		soloPaths?: string[];
		noGroupsFound?: boolean;
	};

	let { summary, soloPaths = [], noGroupsFound = false }: Props = $props();
</script>

<Card className="w-full max-w-2xl rounded-lg border bg-card p-6 text-card-foreground shadow-sm">
	<div class="space-y-4">
		<div class="space-y-1">
			<h2 class="text-xl font-semibold tracking-tight">Next step</h2>
			{#if noGroupsFound}
				<p class="text-sm text-muted-foreground">
					No matching CSV groups were found, so step 2 was skipped. You can continue straight to database ingestion.
				</p>
			{:else}
				<p class="text-sm text-muted-foreground">
					All grouped files are resolved. Review what was merged and what stays standalone.
				</p>
			{/if}
		</div>

		{#if !noGroupsFound}
			<div>
				<h3 class="text-sm font-semibold">Merged groups (cached DataFrames)</h3>
				<ul class="list-disc pl-5 text-sm text-muted-foreground">
					{#if summary.mergedGroupIds.length === 0}
						<li>None</li>
					{:else}
						{#each summary.mergedGroupIds as groupId}
							<li>{groupId}</li>
						{/each}
					{/if}
				</ul>
			</div>

			<div>
				<h3 class="text-sm font-semibold">Standalone groups (paths per group)</h3>
				{#if summary.standaloneGroups.length === 0}
					<ul class="list-disc pl-5 text-sm text-muted-foreground"><li>None</li></ul>
				{:else}
					<ul class="space-y-2 text-sm text-muted-foreground">
						{#each summary.standaloneGroups as group}
							<li>
								<p class="font-medium text-foreground">{group.group_id}</p>
								<ul class="list-disc pl-5">
									{#each group.paths as path}
										<li>{path}</li>
									{/each}
								</ul>
							</li>
						{/each}
					</ul>
				{/if}
			</div>
		{/if}

		<div>
			<h3 class="text-sm font-semibold">
				{noGroupsFound ? "Files continuing to DB ingestion" : "Solo files from first step (never grouped)"}
			</h3>
			<ul class="list-disc pl-5 text-sm text-muted-foreground">
				{#if soloPaths.length === 0}
					<li>None</li>
				{:else}
					{#each soloPaths as path}
						<li>{path}</li>
					{/each}
				{/if}
			</ul>
		</div>
	</div>
</Card>