<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Card } from "$lib/components/ui/card";
  import { toast } from "$lib/components/ui/sonner/sonner";

  let name = $state("");

  async function greet(event: Event) {
    event.preventDefault();
    const message = await invoke<string>("greet", { name });
    toast(message ?? "Hello!");
  }
</script>

<section class="flex min-h-screen flex-col items-center justify-center bg-background px-4">
  <h1 class="mb-8 text-center text-3xl font-semibold">Welcome to Fidashy</h1>
  <Card className="w-full max-w-md space-y-4 rounded-lg border bg-card p-6 text-card-foreground shadow">
    

    <form class="space-y-4" onsubmit={greet}>
      <div class="space-y-2">
        <label for="greet-input" class="text-sm font-medium">Enter a name</label>
        <input
          id="greet-input"
          class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring"
          placeholder="Enter a name..."
          bind:value={name}
        />
      </div>

      <Button type="submit" class="w-full">Greet</Button>
    </form>
  </Card>
</section>

