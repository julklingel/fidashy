<script lang="ts">
import { Card } from "$lib/components/ui/card/index.js";
</script>

<section class="flex min-h-screen flex-col items-center justify-center bg-background px-4">
   <h1 class="mb-6 text-center text-3xl font-semibold">Welcome to Fidashy</h1>
   <a href="/csv-import" class="block w-full max-w-xl">
      <Card className="cursor-pointer rounded-lg border p-6 transition hover:shadow-md">
         <div class="space-y-2">
            <h2 class="text-xl font-semibold">CSV - Database Import</h2>
            <p class="text-sm text-muted-foreground">
               Import multiple CSV files into your local Database, group them by schema, and remove dubs.
            </p>
         </div>
      </Card>
   </a>
</section>

