<script lang="ts">
  import type { DbMatchProposal, GroupResolutionSummary, CreateTableFromSourceResult, MergeSourceIntoTableResult } from "$lib/components/csv-types";
  import { Card } from "$lib/components/ui/card/index.js";
  import { Progress } from "$lib/components/ui/progress/index.js";
  import { invoke } from "@tauri-apps/api/core";
  import { toast } from "svelte-sonner";
  import Button from "./ui/button/button.svelte";

  type Props = {
    summary: GroupResolutionSummary;
    onAllResolved?: () => void;
  };

  let { summary, onAllResolved }: Props = $props();

  const mergedGroups = $derived(summary.mergedGroups);
  const standalonePaths = $derived(
    summary.standaloneGroups.flatMap((group) => group.paths),
  );
  const baseItems = $derived.by<DbMatchProposal[]>(() => [
    ...mergedGroups.map((group) => ({
      source_kind: "group" as const,
      source_id: group.group_id,
      source_paths: group.paths,
      columns: [],
      matching_tables: [],
    })),
    ...summary.standaloneGroups.map((group) => ({
      source_kind: "standalone" as const,
      source_id: group.group_id,
      source_paths: group.paths,
      columns: [],
      matching_tables: [],
    })),
  ]);


  

  let dbMatches = $state<DbMatchProposal[]>([]);
  let hasLoadedMatches = $state(false);
  let isFindingMatches = $state(false);
  let lastLoadedSignature = $state("");
  let loadingProgress = $state(0);
  let selectedTables = $state<Record<string, string>>({});
  let tableNames = $state<Record<string, string>>({});
  let creatingBySourceId = $state<Record<string, boolean>>({});
  let mergingBySourceId = $state<Record<string, boolean>>({});
  let resolvedIds = $state<Set<string>>(new Set());
  

  const allItems = $derived.by(() => {
    const matchesById = new Map(dbMatches.map((item) => [item.source_id, item]));
    return baseItems.map((item) => matchesById.get(item.source_id) ?? item);
  });
  const resolutionItems = $derived(allItems.filter((item) => !resolvedIds.has(item.source_id)));

  function markResolved(sourceId: string) {
    resolvedIds = new Set([...resolvedIds, sourceId]);
    if (resolvedIds.size === allItems.length && allItems.length > 0) {
      toast.success("All sources resolved! Redirecting…");
      onAllResolved?.();
    }
  }
  const summarySignature = $derived(
    JSON.stringify({
      mergedGroups: summary.mergedGroups,
      standaloneGroups: summary.standaloneGroups,
    })
  );

  function toFileName(path: string) {
    const normalized = path.replaceAll("\\", "/");
    const lastSlashIndex = normalized.lastIndexOf("/");
    return lastSlashIndex >= 0 ? normalized.slice(lastSlashIndex + 1) : normalized;
  }

  function sourceLabel(item: DbMatchProposal) {
    if (item.source_kind === "group") return item.source_id;
    return toFileName(item.source_paths[0] ?? item.source_id);
  }

  async function findGroupsWithDb(showSuccessToast = true) {
    if (isFindingMatches) return;

    isFindingMatches = true;
    loadingProgress = hasLoadedMatches ? 35 : 12;
    try {
      const response = await invoke<DbMatchProposal[]>("find_groups_between_db_and_files", {
        groups: summary.mergedGroups,
        standalonePaths,
      });

      dbMatches = response;
      hasLoadedMatches = true;
      lastLoadedSignature = summarySignature;
      loadingProgress = 100;

      for (const item of response) {
        if (!selectedTables[item.source_id] && item.matching_tables.length > 0) {
          selectedTables[item.source_id] = item.matching_tables[0];
        }
      }

      if (showSuccessToast) {
        toast.success(`Checked ${response.length} source(s) against the database.`);
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      loadingProgress = 0;
      toast.error(`DB matching failed: ${message}`);
    } finally {
      isFindingMatches = false;
    }
  }

  async function createTableForSource(item: DbMatchProposal) {
    const preferredTableName = (tableNames[item.source_id] ?? "").trim();

    if (!preferredTableName) {
      toast.error("Please enter a table name.");
      return;
    }
    creatingBySourceId[item.source_id] = true;
    try {
      const result = await invoke<CreateTableFromSourceResult>("create_new_table_from_source", {
        sourceKind: item.source_kind,
        sourceId: item.source_id,
        sourcePaths: item.source_paths,
        preferredTableName,
      });
      toast.success(`Table '${preferredTableName}' created — removed ${result.duplicates_removed} duplicate(s).`);
      markResolved(item.source_id);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      toast.error(`Create table failed: ${message}`);
    } finally {
      creatingBySourceId[item.source_id] = false;
    }
  }

  async function mergeSourceIntoTable(item: DbMatchProposal) {
    const targetTable = selectedTables[item.source_id];
    if (!targetTable) {
      toast.error("Please select a target table.");
      return;
    }

    mergingBySourceId[item.source_id] = true;

    try {
      const result = await invoke<MergeSourceIntoTableResult>("merge_source_into_table", {
        sourceKind: item.source_kind,
        sourceId: item.source_id,
        sourcePaths: item.source_paths,
        targetTable,
      });
      toast.success(
        `Merged '${sourceLabel(item)}' into '${targetTable}' — inserted ${result.rows_inserted} row(s), removed ${result.duplicates_removed} duplicate(s).`
      );
      markResolved(item.source_id);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      toast.error(`Merge into table failed: ${message}`);
    } finally {
      mergingBySourceId[item.source_id] = false;
    }
  }

  $effect(() => {
    if (resolutionItems.length === 0) {
      dbMatches = [];
      hasLoadedMatches = false;
      lastLoadedSignature = "";
      loadingProgress = 0;
      return;
    }

    if (summarySignature === lastLoadedSignature) return;

    hasLoadedMatches = false;
    loadingProgress = 8;
    void findGroupsWithDb(false);
  });

  $effect(() => {
    if (!isFindingMatches) return;

    const intervalId = window.setInterval(() => {
      if (loadingProgress >= 90) return;
      loadingProgress = Math.min(90, loadingProgress + 7);
    }, 180);

    return () => {
      window.clearInterval(intervalId);
    };
  });
</script>

<Card
  className="w-full max-w-5xl rounded-lg border bg-card p-6 text-card-foreground shadow-sm"
>
  <div class="space-y-6">
    <div class="flex flex-col gap-3 md:flex-row md:items-start md:justify-between">
      <div class="space-y-2">
        <h2 class="text-xl font-semibold tracking-tight">Resolve tables against the database</h2>
        <p class="text-sm text-muted-foreground">
          Inspect proposed matches for grouped sources and standalone files, then either merge into an existing table or create a new one.
        </p>
      </div>

      <Button class="shrink-0" onclick={() => findGroupsWithDb(true)} disabled={isFindingMatches}>
        {isFindingMatches ? "Checking DB..." : "Refresh DB matches"}
      </Button>
    </div>

    {#if resolutionItems.length === 0}
      <p class="rounded-lg border px-3 py-6 text-sm text-muted-foreground">
        No grouped or standalone sources were passed into this step.
      </p>
    {:else if !hasLoadedMatches}
      <div class="space-y-4 rounded-lg border p-5">
        <div class="space-y-2">
          <h3 class="font-semibold">Loading database matches</h3>
          <p class="text-sm text-muted-foreground">
            Matching your grouped and standalone CSV sources against existing database tables in the background.
          </p>
        </div>

        <Progress value={loadingProgress} class="h-3" />

        <div class="flex items-center justify-between text-sm text-muted-foreground">
          <span>{loadingProgress}%</span>
          <span>{isFindingMatches ? "Checking schemas..." : "Preparing results..."}</span>
        </div>
      </div>
    {:else}
      <div class="grid gap-4 lg:grid-cols-2">
        {#each resolutionItems as item (item.source_id)}
          <section class="space-y-4 rounded-lg border p-4">
            <div class="space-y-1">
              <div class="flex items-center justify-between gap-3">
                <h3 class="font-semibold">{sourceLabel(item)}</h3>
                <span class="rounded-full border px-2 py-1 text-xs text-muted-foreground">
                  {item.source_kind}
                </span>
              </div>
              <p class="text-sm text-muted-foreground">
                {item.source_paths.length} file(s)
              </p>
            </div>

            <ul class="space-y-2 text-sm">
              {#each item.source_paths as path (path)}
                <li class="rounded-md border bg-background px-3 py-2">
                  <div class="truncate font-medium">{toFileName(path)}</div>
                  <div class="truncate text-xs text-muted-foreground">{path}</div>
                </li>
              {/each}
            </ul>

            {#if item.columns.length > 0}
              <div class="space-y-2">
                <div class="text-xs font-medium uppercase tracking-wide text-muted-foreground">Columns</div>
                <div class="rounded-md border bg-background px-3 py-2 text-sm text-muted-foreground">
                  {item.columns.join(", ")}
                </div>
              </div>
            {/if}

            <div class="space-y-2">
              <div class="text-xs font-medium uppercase tracking-wide text-muted-foreground">Matching tables</div>

              {#if item.matching_tables.length > 0}
                <div class="flex gap-2">
                  <select
                    class="h-9 w-full rounded border bg-background px-2 text-sm"
                    bind:value={selectedTables[item.source_id]}
                  >
                    {#each item.matching_tables as tableName}
                      <option value={tableName}>{tableName}</option>
                    {/each}
                  </select>

                  <Button
                    class="shrink-0"
                    onclick={() => mergeSourceIntoTable(item)}
                    disabled={mergingBySourceId[item.source_id]}
                  >
                    {mergingBySourceId[item.source_id] ? "Merging..." : "Merge into table"}
                  </Button>
                </div>
              {:else if hasLoadedMatches}
                <p class="rounded-md border bg-background px-3 py-2 text-sm text-muted-foreground">
                  No matching DB table found for this source.
                </p>
              {:else if isFindingMatches}
                <p class="rounded-md border bg-background px-3 py-2 text-sm text-muted-foreground">
                  Looking for matching DB tables in the background...
                </p>
              {:else}
                <p class="rounded-md border bg-background px-3 py-2 text-sm text-muted-foreground">
                  DB matches will appear here automatically.
                </p>
              {/if}
            </div>

            <div class="space-y-2">
              <div class="text-xs font-medium uppercase tracking-wide text-muted-foreground">Create new table</div>
              <div class="flex gap-2">
                <input
                  class="h-9 w-full rounded border px-2 text-sm"
                  placeholder="Enter table name"
                  bind:value={tableNames[item.source_id]}
                />

                <Button
                  class="shrink-0"
                  variant="outline"
                  onclick={() => createTableForSource(item)}
                  disabled={creatingBySourceId[item.source_id]}
                >
                  {creatingBySourceId[item.source_id] ? "Creating..." : "Create table"}
                </Button>
              </div>
            </div>
          </section>
        {/each}
      </div>
    {/if}
  </div>
</Card>
