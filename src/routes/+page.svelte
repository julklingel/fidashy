<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Card } from "$lib/components/ui/card";
  import { toast } from "$lib/components/ui/sonner/sonner";

  let name = $state("");
  let greetedPeople = $state<string[]>([]);

  async function loadGreetedPeople() {
    greetedPeople = await invoke<string[]>("list_greeted_people");
  }

  onMount(async () => {
    await loadGreetedPeople();
  });

  async function greet(event: Event) {
    event.preventDefault();
    const trimmedName = name.trim();
    if (!trimmedName) {
      toast("Please enter a name first.");
      return;
    }

    try {
      const message = await invoke<string>("greet", { name: trimmedName });
      await invoke("save_greeting", { name: trimmedName });
      await loadGreetedPeople();
      toast(message ?? "Hello!");
      name = "";
    } catch (error) {
      toast(`Failed to save greeting: ${String(error)}`);
    }
  }
</script>

<section class="flex min-h-screen flex-col items-center justify-center bg-background px-4">
  <h1 class="mb-8 text-center text-3xl font-semibold">Welcome to Fidashy</h1>
  <Card className="w-full max-w-md space-y-4 rounded-lg border bg-card p-6 text-card-foreground shadow">

    <form class="space-y-4" onsubmit={greet}>
      <div class="space-y-2">
        <label for="greet-input" class="text-sm my-4 font-medium">Enter a name</label>
        <input
          id="greet-input"
          class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring"
          placeholder="Enter a name..."
          bind:value={name}
        />
      </div>

      <Button type="submit" class="w-full">Greet</Button>
    </form>

    <div class="space-y-2 pt-2">
      <h2 class="text-sm font-medium">Greeted people</h2>
      {#if greetedPeople.length === 0}
        <p class="text-sm text-muted-foreground">No greetings saved yet.</p>
      {:else}
        <ul class="space-y-1 text-sm">
          {#each greetedPeople as person}
            <li class="rounded-md border px-2 py-1">{person}</li>
          {/each}
        </ul>
      {/if}
    </div>
  </Card>
</section>

