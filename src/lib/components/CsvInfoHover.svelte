<script lang="ts">

  import Info from "@lucide/svelte/icons/info";

  
</script>

        <div class="group relative inline-flex">
          <button
            type="button"
            class="inline-flex h-5 w-5 items-center justify-center rounded-full border border-input text-muted-foreground"
            aria-label="How CSV processing works"
          >
            <Info class="h-3.5 w-3.5" />
          </button>
          <div class="pointer-events-none absolute left-0 top-6 z-10 hidden w-64 rounded-md border bg-popover p-3 text-xs text-popover-foreground shadow-md group-hover:block">
            Pick one or more CSV files. When you press "process", the app reads each provided file path and loads the data into a optimized database file on your computer.
          </div>
        </div>

