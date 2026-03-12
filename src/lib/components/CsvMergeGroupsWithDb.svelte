<script lang="ts">
  import FileProcessingResults from "./FileProcessingResults.svelte";
  import type {
    CachedGroupDbMatch,
    CreateTableFromCachedGroupResult,
    FindGroupsBetweenDbAndFilesResult,
    GroupResolutionSummary,
    MergeCachedGroupIntoTableResult,
    MergeFileIntoTableResult,
  } from "$lib/components/csv-types";
  import { Card } from "$lib/components/ui/card/index.js";
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
  } from "$lib/components/ui/dropdown-menu/index.js";
  import { invoke } from "@tauri-apps/api/core";
  import { toast } from "svelte-sonner";
  import Button from "./ui/button/button.svelte";

  type Props = {
    summary: GroupResolutionSummary;
    soloPaths?: string[];
    noGroupsFound?: boolean;
  };

  let { summary, soloPaths = [], noGroupsFound = false }: Props = $props();

  const mergedGroups = $derived(summary.mergedGroups);
  const mergedGroupIds = $derived(summary.mergedGroups.map((group) => group.group_id));
  const mergedPaths = $derived(summary.mergedGroups.flatMap((group) => group.paths));
  const standaloneGroups = $derived(summary.standaloneGroups);
  const standalonePaths = $derived(
    summary.standaloneGroups.flatMap((group) => group.paths),
  );
  const passthroughPaths = $derived(Array.from(new Set(soloPaths)));
  const allFilePaths = $derived.by(() =>
    Array.from(new Set([...passthroughPaths, ...standalonePaths, ...mergedPaths])),
  );
  const mergedGroupIdsText = $derived(mergedGroupIds.join(", "));
  const matchLookupKey = $derived(`${allFilePaths.join("||")}::${mergedGroupIds.join("||")}`);

  let isFindingMatches = $state(false);
  let hasLoadedMatches = $state(false);
  let loadedKey = $state<string | null>(null);
  let matchResult = $state<FindGroupsBetweenDbAndFilesResult>({
    matched_files: [],
    matched_groups: [],
  });
  let mergingByPath = $state<Record<string, boolean>>({});
  let mergingByGroupId = $state<Record<string, boolean>>({});
  let creatingByGroupId = $state<Record<string, boolean>>({});
  let selectedGroupTargetTableById = $state<Record<string, string>>({});
  let selectedFileTargetTableByPath = $state<Record<string, string>>({});

  const matchByFilePath = $derived.by(() => {
    const map = new Map<string, FindGroupsBetweenDbAndFilesResult["matched_files"][number]>();
    for (const match of matchResult.matched_files) {
      map.set(match.file_path, match);
    }
    return map;
  });

  const matchedGroupById = $derived.by(() => {
    const map = new Map<string, CachedGroupDbMatch>();
    for (const match of matchResult.matched_groups) {
      map.set(match.group_id, match);
    }
    return map;
  });

  const mergedPathSet = $derived.by(() => new Set(mergedPaths));
  const standaloneFileRows = $derived.by(() =>
    allFilePaths.filter((path) => !mergedPathSet.has(path)),
  );

  $effect(() => {
    const key = matchLookupKey;
    if (loadedKey === key || isFindingMatches) return;
    void findGroupsWithDb(allFilePaths, mergedGroupIds, key);
  });

  async function findGroupsWithDb(allPaths: string[], cacheGroupIds: string[], key: string) {
    isFindingMatches = true;
    hasLoadedMatches = false;

    try {
      const response = await invoke<FindGroupsBetweenDbAndFilesResult>(
        "find_groups_between_db_and_files",
        {
          paths: allPaths,
          cacheIds: cacheGroupIds,
        },
      );
      matchResult = response;
      loadedKey = key;
    } catch (error) {
      toast.error(`Failed to match DB/cache schemas: ${error}`);
      matchResult = { matched_files: [], matched_groups: [] };
    } finally {
      isFindingMatches = false;
      hasLoadedMatches = true;
    }
  }

  let tableNames = $state<Record<string, string>>({});
  let creatingByPath = $state<Record<string, boolean>>({});

  async function createTableFromGroup(groupId: string) {
    const preferredTableName = (tableNames[groupId] ?? "").trim();

    if (!preferredTableName) {
      toast.error("Please enter a table name.");
      return;
    }

    creatingByGroupId[groupId] = true;
    try {
      const result = await invoke<CreateTableFromCachedGroupResult>(
        "create_new_table_from_cached_group",
        {
          groupId,
          preferredTableName,
        },
      );
      toast.success(result.message);
    } catch (error) {
      toast.error(`Create table failed: ${error}`);
    } finally {
      creatingByGroupId[groupId] = false;
    }
  }

  async function createTableFromPath(path: string) {
    const preferredTableName = (tableNames[path] ?? "").trim();

    if (!preferredTableName) {
      toast.error("Please enter a table name.");
      return;
    }

    creatingByPath[path] = true;
    try {
      await invoke("create_new_table_from_source", {
        sourcePath: path,
        preferredTableName,
      });
      toast.success(`Table '${preferredTableName}' created.`);
    } catch (error) {
      toast.error(`Create table failed: ${error}`);
    } finally {
      creatingByPath[path] = false;
    }
  }

  async function mergeMatchedFile(path: string, selectedTargetTable?: string) {
    const match = matchByFilePath.get(path);
    const targetTable = selectedTargetTable ?? match?.matched_table_names[0];

    if (!targetTable) {
      toast.error("No matched target table found for this file.");
      return;
    }

    mergingByPath[path] = true;
    try {
      const result = await invoke<MergeFileIntoTableResult>("merge_source_file_into_table", {
        sourcePath: path,
        targetTable,
      });
      toast.success(result.message);
    } catch (error) {
      toast.error(`Merge failed: ${error}`);
    } finally {
      mergingByPath[path] = false;
    }
  }

  async function mergeMatchedGroup(groupId: string, selectedTargetTable?: string) {
    const matchedGroup = matchedGroupById.get(groupId);
    const targetTable = selectedTargetTable ?? matchedGroup?.matched_table_names[0];

    if (!targetTable) {
      toast.error("No matched target table found for this group.");
      return;
    }

    mergingByGroupId[groupId] = true;
    try {
      const result = await invoke<MergeCachedGroupIntoTableResult>(
        "merge_cached_group_into_table",
        {
          groupId,
          targetTable,
        },
      );
      toast.success(result.message);
    } catch (error) {
      toast.error(`Group merge failed: ${error}`);
    } finally {
      mergingByGroupId[groupId] = false;
    }
  }
</script>

<FileProcessingResults
  {mergedGroups}
  {mergedGroupIds}
  {standaloneGroups}
  {passthroughPaths}
  {allFilePaths}
  {noGroupsFound}
/>

<Card
  className="w-full max-w-2xl rounded-lg border bg-card p-6 text-card-foreground shadow-sm"
>
  <h2 class="text-xl pb-4 font-semibold tracking-tight">
    Resolve tables against the database
  </h2>

  {#if mergedGroupIds.length > 0}
    <p class="pb-4 text-sm text-muted-foreground">
      Merged group IDs: {mergedGroupIdsText}
    </p>
  {/if}

  {#if !hasLoadedMatches || isFindingMatches}
    <p class="rounded border p-3 text-sm text-muted-foreground">
      Checking database and cache matches for current files...
    </p>
  {:else}
    <ul class="space-y-3">
      {#each mergedGroups as group}
        {@const groupMatch = matchedGroupById.get(group.group_id)}
        {@const matchedTables = groupMatch?.matched_table_names ?? []}
        {@const selectedTargetTable = selectedGroupTargetTableById[group.group_id] ?? matchedTables[0]}
        <li class="rounded border p-3">
          <div class="mb-2 text-sm font-medium">Merged group: {group.group_id}</div>

          <ul class="mb-2 list-disc space-y-1 pl-5 text-xs text-muted-foreground">
            {#each group.paths as path}
              <li>{path}</li>
            {/each}
          </ul>

          {#if matchedTables.length > 0}
            <div
              class="mb-3 flex w-full flex-col gap-2 text-xs text-muted-foreground sm:flex-row sm:items-center"
            >
              <p>- Merge Data with DB Table</p>
              <div class="w-full sm:flex-1">
                <DropdownMenu>
                  <DropdownMenuTrigger
                    class="inline-flex h-8 w-full items-center justify-between rounded border px-2 text-xs"
                  >
                    {selectedTargetTable}
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="start">
                    {#each matchedTables as tableName}
                      <DropdownMenuItem
                        onclick={() => {
                          selectedGroupTargetTableById[group.group_id] = tableName;
                        }}
                      >
                        {tableName}
                      </DropdownMenuItem>
                    {/each}
                  </DropdownMenuContent>
                </DropdownMenu>
              </div>

              <Button
                class="w-28 shrink-0"
                variant="outline"
                onclick={() => mergeMatchedGroup(group.group_id, selectedTargetTable)}
                disabled={creatingByGroupId[group.group_id] || mergingByGroupId[group.group_id]}
              >
                {mergingByGroupId[group.group_id] ? "Merging..." : "Merge"}
              </Button>
            </div>
          {/if}

          <div class="flex items-center gap-2">
            <input
              class="h-9 w-full rounded border px-2 text-sm"
              placeholder="Enter table name"
              bind:value={tableNames[group.group_id]}
            />

            <Button
              class="w-28 shrink-0"
              onclick={() => createTableFromGroup(group.group_id)}
              disabled={creatingByGroupId[group.group_id] || mergingByGroupId[group.group_id]}
            >
              {creatingByGroupId[group.group_id] ? "Creating..." : "Create table"}
            </Button>
          </div>
        </li>
      {/each}

      {#each standaloneFileRows as path}
        {@const pathMatch = matchByFilePath.get(path)}
        {@const matchedTables = pathMatch?.matched_table_names ?? []}
        {@const matchedCacheIds = pathMatch?.matched_cache_group_ids ?? []}
        {@const selectedTargetTable = selectedFileTargetTableByPath[path] ?? matchedTables[0]}
        <li class="rounded border p-3">
          <div class="mb-2 text-sm">{path}</div>

          {#if matchedTables.length > 0 || matchedCacheIds.length > 0}
            <div class="mb-2 space-y-1 text-xs text-muted-foreground">
              {#if matchedTables.length > 0}
                <div class="mb-3 flex w-full flex-col gap-2 sm:flex-row sm:items-center">
                  <p>- Merge Data with DB Table</p>
                  <div class="w-full sm:flex-1">
                    <DropdownMenu>
                      <DropdownMenuTrigger
                        class="inline-flex h-8 w-full items-center justify-between rounded border px-2 text-xs"
                      >
                        {selectedTargetTable}
                      </DropdownMenuTrigger>
                      <DropdownMenuContent align="start">
                        {#each matchedTables as tableName}
                          <DropdownMenuItem
                            onclick={() => {
                              selectedFileTargetTableByPath[path] = tableName;
                            }}
                          >
                            {tableName}
                          </DropdownMenuItem>
                        {/each}
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </div>

                  <Button
                    class="w-28 shrink-0"
                    variant="outline"
                    onclick={() => mergeMatchedFile(path, selectedTargetTable)}
                    disabled={creatingByPath[path] || mergingByPath[path]}
                  >
                    {mergingByPath[path] ? "Merging..." : "Merge"}
                  </Button>
                </div>
              {/if}
              {#if matchedCacheIds.length > 0}
                <p>Matched cache group ID(s): {matchedCacheIds.join(", ")}</p>
              {/if}
            </div>
          {/if}

          <div class="flex items-center gap-2">
            <input
              class="h-9 w-full rounded border px-2 text-sm"
              placeholder="Enter table name"
              bind:value={tableNames[path]}
            />

            <Button
              class="w-28 shrink-0"
              onclick={() => createTableFromPath(path)}
              disabled={creatingByPath[path] || mergingByPath[path]}
            >
              {creatingByPath[path] ? "Creating..." : "Create table"}
            </Button>
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</Card>
