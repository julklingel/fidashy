<script lang="ts">
  	

	import { invoke } from "@tauri-apps/api/core";
	import { open } from "@tauri-apps/plugin-dialog";
	import { Card } from "$lib/components/ui/card/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import { toast } from "$lib/components/ui/sonner/sonner.js";

	import CsvInfoHover from "$lib/components/CsvInfoHover.svelte";
	import type { GroupWithDuplicates, SelectedCsvFile } from "$lib/components/csv-types";

	type Props = {
		groupedPaths?: GroupWithDuplicates[];
		soloPaths?: string[];
		noGroupsFound?: boolean;
		
	};

	let {
		groupedPaths = $bindable<GroupWithDuplicates[]>([]),
		soloPaths = $bindable<string[]>([]),
		noGroupsFound = $bindable(false)
	}: Props = $props();
	
	let selectedFiles = $state<SelectedCsvFile[]>([]);
	let isProcessing = $state(false);
	
	const selectedFilesLabel = $derived(
		selectedFiles.length === 0 ? "No files selected" : `${selectedFiles.length} file(s) selected`
	);

	function toFileName(path: string) {
		const normalized = path.replaceAll("\\", "/");
		const lastSlashIndex = normalized.lastIndexOf("/");
		return lastSlashIndex >= 0 ? normalized.slice(lastSlashIndex + 1) : normalized;
	}

	async function chooseCsvFiles() {
		const response = await open({
			multiple: true,
			filters: [{ name: "CSV", extensions: ["csv", "CSV"] }],
		});

		if (!response) return;

		const paths = Array.isArray(response) ? response : [response];
		const discoveredFiles: SelectedCsvFile[] = paths.map((path) => ({
			path,
			name: toFileName(path),
		}));

		const byPath = new Map<string, SelectedCsvFile>();
		for (const file of selectedFiles) byPath.set(file.path, file);
		for (const file of discoveredFiles) byPath.set(file.path, file);

		selectedFiles = Array.from(byPath.values());
		groupedPaths = [];
		soloPaths = [];
		noGroupsFound = false;
	}

	function deleteFile(path: string) {
		selectedFiles = selectedFiles.filter((file) => file.path !== path);
		groupedPaths = [];
		soloPaths = [];
		noGroupsFound = false;
	}

	async function processFiles() {
		if (selectedFiles.length === 0 || isProcessing) return;

		isProcessing = true;
		try {
			const paths = selectedFiles.map((file) => file.path);
			const response = await invoke<GroupWithDuplicates[]>("lazy_grouping_csv_many", { paths });
			groupedPaths = response;
			noGroupsFound = response.length === 0;

			const groupedSet = new Set(response.flatMap((group) => group.paths));
			soloPaths = paths.filter((path) => !groupedSet.has(path));
			toast.success(`Processed ${paths.length} file(s) into ${response.length} group(s).`);
			
		} catch (error) {
			groupedPaths = [];
			soloPaths = [];
			noGroupsFound = false;
			const message = error instanceof Error ? error.message : String(error);
			toast.error(`CSV processing failed: ${message}`);
		} finally {

			isProcessing = false;
		}
	}
</script>

<Card className="w-full max-w-2xl rounded-lg border bg-card p-6 text-card-foreground shadow-sm">
	<div class="space-y-5">
		<div class="flex items-center gap-2">
			<h2 class="text-xl font-semibold tracking-tight">Upload CSV files</h2>
			<CsvInfoHover />
		</div>

		<div class="space-y-3">
			<label for="selected-csv-files" class="text-sm font-medium">Select CSV files</label>
			<div class="flex gap-2">
				<input
					id="selected-csv-files"
					type="text"
					readonly
					value={selectedFilesLabel}
					class="flex h-9 w-full rounded-md border border-input bg-background px-3 py-2 text-sm text-muted-foreground"
				/>
				<Button type="button" variant="outline" onclick={chooseCsvFiles} disabled={isProcessing}>
					Choose files
				</Button>
			</div>

			{#if selectedFiles.length > 0}
				<div class="rounded-md border">
					<div class="border-b px-3 py-2 text-sm font-medium">Selected files ({selectedFiles.length})</div>
					<ul class="max-h-64 divide-y overflow-y-auto">
						{#each selectedFiles as file (file.path)}
							<li class="flex items-start gap-3 px-3 py-2 text-sm">
								<div class="min-w-0 flex-1">
									<div class="truncate font-medium">{file.name}</div>
									<div class="truncate text-xs text-muted-foreground">{file.path}</div>
								</div>
								<Button
									type="button"
									variant="ghost"
									size="sm"
									onclick={() => deleteFile(file.path)}
									disabled={isProcessing}
								>
									<p class="text-red-500">Delete</p>
								</Button>
							</li>
						{/each}
					</ul>
				</div>
			{/if}
		</div>

		<Button class="w-full" onclick={processFiles} disabled={selectedFiles.length === 0 || isProcessing}>
			{isProcessing ? "Processing..." : "Process"}
		</Button>

	</div>
</Card>
