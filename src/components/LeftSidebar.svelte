<script lang="ts">
  import { X } from "lucide-svelte";
  import NewPoint from "./NewPoint.svelte";

  type LeftSidebarProps = {
    leftSidebar: boolean,
    realtime: PointData[]
  }

  let { leftSidebar, realtime }: LeftSidebarProps = $props()

  let test =
    '{"geometry":{"coordinates":[-111.3665,45.9583],"type":"Point"},"source_id":"1747322","source_catalog":"EMSC-RTS","lastupdate":"2024-12-23T20:50:05.790919Z","time":"2024-12-23T19:22:40.77Z","lat":45.9583,"lon":-111.3665,"depth":2.4,"evtype":"ke","auth":"MB","mag":2.1,"magtype":"ml","flynn_region":"WESTERN MONTANA","unid":"20241223_0000210","origins":null,"arrivals":null}';


</script>
<aside
  class:hidden={leftSidebar}
  class=" pt-5 h-full fixed bg-white left-0 top-0 z-50 shadow-lg"
>
  <button onclick={() => (leftSidebar = !leftSidebar)} class="absolute right-3 top-2">
    <X />
  </button>

  <NewPoint pointData={JSON.parse(test)}></NewPoint>

  {#each realtime as real, index}
    <!-- {JSON.stringify(real) as unknown as PointData} -->
    <NewPoint pointData={real}></NewPoint>
  {/each}
</aside>