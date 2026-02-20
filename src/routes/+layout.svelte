<script lang="ts">
  import "../app.css";
  
  import { ModeWatcher } from "mode-watcher";
  import ModeToggle from '../lib/components/ModeToggle.svelte';
  import Toaster from "$lib/components/ui/sonner/Toaster.svelte";
</script>

<ModeWatcher />

<ModeToggle />

<Toaster />

<slot />
