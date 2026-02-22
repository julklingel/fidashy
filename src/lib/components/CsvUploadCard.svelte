<script lang="ts">
  import { onDestroy } from "svelte";
  import CsvNextStepCard from "./CsvNextStepCard.svelte";
  import CsvProcessResultsCard from "./CsvProcessResultsCard.svelte";
  import CsvUploadActionCard from "./CsvUploadActionCard.svelte";
  import type { GroupMergeStatus, MergeState, ProcessedPayload, SchemaGroup } from "./csv-types";

  const MERGE_PROGRESS_INTERVAL_MS = 120;
  const MERGE_PROGRESS_STEP = 20;

  const defaultGroupMergeStatus = (): GroupMergeStatus => ({
    isMerging: false,
    status: "idle",
    progress: 0,
  });

  let flowState = $state<{
    step: "upload" | "results" | "next";
    groups: SchemaGroup[];
    mergeState: MergeState;
  }>({
    step: "upload",
    groups: [],
    mergeState: { groups: {} },
  });

  const mergeTimers = new Map<string, ReturnType<typeof setInterval>>();

  onDestroy(() => {
    for (const timer of mergeTimers.values()) {
      clearInterval(timer);
    }
    mergeTimers.clear();
  });

  function buildInitialMergeState(groups: SchemaGroup[]): MergeState {
    return {
      groups: Object.fromEntries(
        groups.map((group) => [group.id, defaultGroupMergeStatus()])
      ),
    };
  }

  function getGroupStatus(groupId: string): GroupMergeStatus {
    return flowState.mergeState.groups[groupId] ?? defaultGroupMergeStatus();
  }

  function moveToNextStepIfFinished() {
    if (flowState.groups.length === 0) {
      flowState.step = "next";
    }
  }

  function clearMergeTimer(groupId: string) {
    const timer = mergeTimers.get(groupId);
    if (!timer) {
      return;
    }

    clearInterval(timer);
    mergeTimers.delete(groupId);
  }

  function resetToUploadStep() {
    for (const timerId of mergeTimers.keys()) {
      clearMergeTimer(timerId);
    }

    flowState.groups = [];
    flowState.mergeState = { groups: {} };
    flowState.step = "upload";
  }

  function handleProcessed(payload: ProcessedPayload) {
    flowState.groups = payload.groups;
    flowState.mergeState = buildInitialMergeState(payload.groups);
    flowState.step = payload.groups.length === 0 ? "next" : "results";
  }

  function finalizeGroupDecision(groupId: string, status: "merged" | "canceled") {
    clearMergeTimer(groupId);

    const current = getGroupStatus(groupId);
    flowState.mergeState.groups[groupId] = {
      ...current,
      isMerging: false,
      status,
      progress: status === "merged" ? 100 : 0,
    };

    flowState.groups = flowState.groups.filter((group) => group.id !== groupId);
    moveToNextStepIfFinished();
  }

  function handleMerge(groupId: string) {
    const current = getGroupStatus(groupId);
    if (current.isMerging) {
      return;
    }

    flowState.mergeState.groups[groupId] = {
      ...current,
      isMerging: true,
      progress: 0,
    };

    const timer = setInterval(() => {
      const state = flowState.mergeState.groups[groupId];
      if (!state) {
        clearInterval(timer);
        mergeTimers.delete(groupId);
        return;
      }

      const nextProgress = Math.min(100, state.progress + MERGE_PROGRESS_STEP);
      flowState.mergeState.groups[groupId] = {
        ...state,
        progress: nextProgress,
      };

      if (nextProgress >= 100) {
        finalizeGroupDecision(groupId, "merged");
      }
    }, MERGE_PROGRESS_INTERVAL_MS);

    mergeTimers.set(groupId, timer);
  }

  function handleCancel(groupId: string) {
    finalizeGroupDecision(groupId, "canceled");
  }

  function handleCancelResultsStep() {
    resetToUploadStep();
  }
</script>

<div class="w-full max-w-md space-y-4">
  {#if flowState.step === "upload"}
    <h1 class="mb-4 text-center text-3xl font-semibold">Welcome to Fidashy</h1>
    <CsvUploadActionCard onProcessed={handleProcessed} />
  {:else if flowState.step === "results"}
    <CsvProcessResultsCard
      groups={flowState.groups}
      mergeState={flowState.mergeState}
      onMerge={handleMerge}
      onCancel={handleCancel}
      onCancelStep={handleCancelResultsStep}
    />
  {:else}
    <CsvNextStepCard />
  {/if}
</div>
