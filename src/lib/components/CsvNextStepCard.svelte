<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { NextStepDecision } from "./csv-types";
  import { Button } from "$lib/components/ui/button";
  import { Card } from "$lib/components/ui/card";
  import { toast } from "$lib/components/ui/sonner/sonner";
  import type { CsvIngestionWriteResult } from "./csv-types";

  let {
    decisions = [],
    onCancel,
    onDecisionCompleted,
  }: {
    decisions?: NextStepDecision[];
    onCancel?: () => void;
    onDecisionCompleted?: (groupId: string) => void;
  } = $props();

  let busyGroups = $state<Record<string, boolean>>({});
  let suggestedTableNames = $state<Record<string, string>>({});

  function isBusy(groupId: string): boolean {
    return busyGroups[groupId] ?? false;
  }

  function getSuggestedTableName(groupId: string): string {
    return suggestedTableNames[groupId] ?? "";
  }

  async function createNewTable(decision: NextStepDecision) {
    busyGroups[decision.groupId] = true;
    try {
      const suggestedTableName = getSuggestedTableName(decision.groupId).trim();
      const result = await invoke<CsvIngestionWriteResult>("create_table_from_csv_group", {
        paths: decision.filePaths,
        suggestedTableName: suggestedTableName.length > 0 ? suggestedTableName : null,
      });
      toast(
        `Created table ${result.table_name}. Inserted ${result.rows_inserted} row(s), skipped ${result.rows_skipped_duplicates} duplicate row(s).`
      );
      onDecisionCompleted?.(decision.groupId);
    } catch (error) {
      toast(`Failed to create table: ${String(error)}`);
    } finally {
      busyGroups[decision.groupId] = false;
    }
  }

  async function mergeIntoExistingTable(decision: NextStepDecision) {
    const tableName = decision.mergeResult.matching_table_name;
    if (!tableName) {
      toast("No matching table available for merge.");
      return;
    }

    busyGroups[decision.groupId] = true;
    try {
      const result = await invoke<CsvIngestionWriteResult>("merge_csv_group_into_existing_table", {
        paths: decision.filePaths,
        tableName,
      });
      toast(
        `Merged into ${result.table_name}. Inserted ${result.rows_inserted} row(s), skipped ${result.rows_skipped_duplicates} duplicate row(s).`
      );
      onDecisionCompleted?.(decision.groupId);
    } catch (error) {
      toast(`Failed to merge with existing table: ${String(error)}`);
    } finally {
      busyGroups[decision.groupId] = false;
    }
  }
</script>

<Card className="w-full max-w-md space-y-4 rounded-lg border bg-card p-6 text-card-foreground shadow">
  <div class="space-y-2">
    <p class="text-sm font-medium">Choose database ingestion strategy</p>
    <p class="text-xs text-muted-foreground">Review each merged group and decide how to continue.</p>
  </div>

  {#if decisions.length === 0}
    <div class="space-y-3">
      <p class="text-sm font-medium">You are all set.</p>
      <p class="text-xs text-muted-foreground">All groups have been ingested successfully.</p>
      <Button type="button" class="h-8 px-3 text-xs" href="/dashboard">
        Go to dashboard
      </Button>
    </div>
  {:else}
    <div class="space-y-3">
      <ul class="space-y-3 text-sm">
        {#each decisions as decision}
          <li class="space-y-2 rounded-md border px-3 py-3">
            <p class="font-medium">Files: {decision.fileNames.join(", ")}</p>
            <p class="text-muted-foreground">
              New DataFrame shape: {decision.mergeResult.merged_rows} rows × {decision.mergeResult.merged_columns} columns
            </p>
            <div class="space-y-1">
              <label for={`table-name-${decision.groupId}`} class="text-xs text-muted-foreground">
                New table name (optional)
              </label>
              <input
                id={`table-name-${decision.groupId}`}
                type="text"
                class="h-8 w-full rounded-md border bg-background px-2 text-xs"
                placeholder="Auto-generate if empty"
                value={getSuggestedTableName(decision.groupId)}
                oninput={(event) => {
                  const target = event.currentTarget as HTMLInputElement;
                  suggestedTableNames[decision.groupId] = target.value;
                }}
                disabled={isBusy(decision.groupId)}
              />
            </div>

            {#if decision.mergeResult.matching_table_name}
              <p class="text-muted-foreground">
                Matching table in DuckDB: {decision.mergeResult.matching_table_name}
              </p>
              <p class="text-muted-foreground">
                Potential duplicates between DB and merged CSV: {decision.mergeResult.duplicate_rows_with_db}
              </p>
              <div class="flex gap-2">
                <Button
                  type="button"
                  class="h-8 px-3 text-xs"
                  onclick={() => mergeIntoExistingTable(decision)}
                  disabled={isBusy(decision.groupId)}
                >
                  Merge with existing table
                </Button>
                <Button
                  type="button"
                  variant="outline"
                  class="h-8 px-3 text-xs"
                  onclick={() => createNewTable(decision)}
                  disabled={isBusy(decision.groupId)}
                >
                  Create new table
                </Button>
              </div>
            {:else}
              <p class="text-muted-foreground">No matching table shape found in DuckDB.</p>
              <Button
                type="button"
                class="h-8 px-3 text-xs"
                onclick={() => createNewTable(decision)}
                disabled={isBusy(decision.groupId)}
              >
                Create new table
              </Button>
            {/if}
          </li>
        {/each}
      </ul>
    <div class="flex justify-end">
      <Button type="button" variant="outline" class="h-8 px-3 text-xs" onclick={() => onCancel?.()}>
        Cancel
      </Button>
    </div>
    </div>
  {/if}
</Card>
