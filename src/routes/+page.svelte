<script lang="ts">
import { Card } from "$lib/components/ui/card/index.js";
</script>

<section class="min-h-screen bg-background px-4 py-10">
   <div class="mx-auto w-full max-w-5xl space-y-8">
      <div class="space-y-2 text-center">
         <h1 class=" text-4xl text-primary font-semibold">Welcome to Fidashy</h1>
         
      </div>

      <div class="space-y-3">
         <h2 class="text-sm font-semibold uppercase tracking-wide text-muted-foreground">Data and Database</h2>
         <div class="grid gap-4 md:grid-cols-2">
            <a href="/analyse-dashboard" class="block">
               <Card className="h-full cursor-pointer rounded-lg border p-6 transition hover:shadow-md">
                  <div class="space-y-2">
                     <h3 class="text-xl font-semibold">Analytics Dashboard</h3>
                     <p class="text-sm text-muted-foreground">
                        Analyze your data from multiple sources in one place.
                     </p>
                  </div>
               </Card>
            </a>

            <a href="/db-manager" class="block">
               <Card className="h-full cursor-pointer rounded-lg border p-6 transition hover:shadow-md">
                  <div class="space-y-2">
                     <h3 class="text-xl font-semibold">Database Manager</h3>
                     <p class="text-sm text-muted-foreground">
                        Manage and inspect tables in your local DuckDB database.
                     </p>
                  </div>
               </Card>
            </a>
         </div>
      </div>

      <div class="space-y-3">
         <h2 class="text-sm font-semibold uppercase tracking-wide text-muted-foreground">CSV Utilities</h2>
         <div class="grid gap-4 md:grid-cols-2">
            <a href="/csv-import" class="block">
               <Card className="h-full cursor-pointer rounded-lg border p-6 transition hover:shadow-md">
                  <div class="space-y-2">
                     <h3 class="text-xl font-semibold">CSV to Database Import</h3>
                     <p class="text-sm text-muted-foreground">
                        Import multiple CSV files, group by schema, and handle duplicates before ingestion.
                     </p>
                  </div>
               </Card>
            </a>

            <a href="/csv-delete-dubs" class="block">
               <Card className="h-full cursor-pointer rounded-lg border p-6 transition hover:shadow-md">
                  <div class="space-y-2">
                     <h3 class="text-xl font-semibold">Remove CSV Duplicates</h3>
                     <p class="text-sm text-muted-foreground">
                        Remove duplicate rows from one or multiple CSV files.
                     </p>
                  </div>
               </Card>
            </a>

            <a href="/csv-merge-files" class="block md:col-span-2">
               <Card className="h-full cursor-pointer rounded-lg border p-6 transition hover:shadow-md">
                  <div class="space-y-2">
                     <h3 class="text-xl font-semibold">Merge CSV Files</h3>
                     <p class="text-sm text-muted-foreground">
                        Combine multiple CSV files into a single output file.
                     </p>
                  </div>
               </Card>
            </a>
         </div>
      </div>
   </div>
</section>

