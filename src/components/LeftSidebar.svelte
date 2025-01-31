<script lang="ts">
  import { X } from "lucide-svelte";
  import NewPoint from "./NewPoint.svelte";

  type LeftSidebarProps = {
    leftSidebar: boolean,
    realtime: PointData[]
  }

  let { leftSidebar, realtime }: LeftSidebarProps = $props()



</script>
<aside
  class:hidden={leftSidebar}
  class=" pt-5 h-full fixed bg-white left-0 top-0 z-50 shadow-lg overflow-y-scroll"
>
  <button onclick={() => (leftSidebar = !leftSidebar)} class="absolute right-3 top-2">
    <X />
  </button>

  <div class="flex gap-4 m-4 text-xs items-center w-80">
    <div class="h-1  min-w-1 rounded"></div>
</div>

  {#each realtime as real, index}
    <!-- {JSON.stringify(real) as unknown as PointData} -->
    <NewPoint pointData={real}></NewPoint>
  {/each}
</aside>