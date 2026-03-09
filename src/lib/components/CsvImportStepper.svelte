<script lang="ts">
	import { Progress } from "$lib/components/ui/progress/index.js";

	type Step = {
		id: number;
		title: string;
		description: string;
	};

	type Props = {
		steps: Step[];
		currentStep: number;
	};

	let { steps, currentStep }: Props = $props();

	const progressValue = $derived.by(() => {
		if (steps.length <= 1) return 100;
		return ((currentStep - 1) / (steps.length - 1)) * 100;
	});
</script>

<div class="w-full max-w-2xl space-y-4">
	<Progress value={progressValue} class="h-2" />

	<ol class="grid gap-3 sm:grid-cols-3">
		{#each steps as step}
			{@const isComplete = currentStep > step.id}
			{@const isCurrent = currentStep === step.id}

			<li class="rounded-lg border bg-card p-3 text-left shadow-sm transition-colors">
				<div class="flex items-start gap-3">
					<div
						class={`flex h-8 w-8 shrink-0 items-center justify-center rounded-full border text-sm font-semibold ${
							isComplete
								? "border-primary bg-primary text-primary-foreground"
								: isCurrent
									? "border-primary text-primary"
									: "border-muted-foreground/30 text-muted-foreground"
						}`}
					>
						{#if isComplete}
							✓
						{:else}
							{step.id}
						{/if}
					</div>

					<div class="min-w-0 space-y-1">
						<p class={`text-sm font-medium ${isCurrent ? "text-foreground" : "text-muted-foreground"}`}>
							{step.title}
						</p>
						<p class="text-xs text-muted-foreground">{step.description}</p>
					</div>
				</div>
			</li>
		{/each}
	</ol>
</div>