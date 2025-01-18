<script lang="ts">
  import { onMount } from "svelte";
  import { defaultTheme } from "../theme";
  import { type PageData } from "./$types";
  import "../app.css";
  import { Activity, ChartLine, Settings } from "lucide-svelte";
  import { MapFunc } from "./map-libre-init";
  import LeftSidebar from "../components/LeftSidebar.svelte";
  import RightSidebar from "../components/RightSidebar.svelte";

  let { data }: { data: PageData } = $props();

  let leftSidebar = $state(true);
  let rightSidebar = $state(true);

  let realtime: Array<PointData> = $state([]);

  let settings = $state(false);
  let tmpLimit = $state(100)

  onMount(async () => {
    const PMTILES_URL = "./my_area.pmtiles";
    const map = await MapFunc({ data, PMTILES_URL, defaultTheme, realtime });

    tmpLimit = parseInt(window.localStorage.getItem('limit') || "100")
    console.error(tmpLimit)

  });

  let submit = async () => {
      window.localStorage.setItem('limit', tmpLimit.toString())
      location.reload()  
};


</script>

<div id="map"></div>

<div class="fixed top-0 left-0 m-4">
  <button
    class="p-2 rounded bg-white shadow-lg"
    onclick={() => (leftSidebar = !leftSidebar)}
  >
    <Activity />
  </button>

  <button
  class="p-2 rounded bg-white shadow-lg"
  onclick={() => (rightSidebar = !rightSidebar)}
>
  <ChartLine />
</button>

<button
class="p-2 rounded bg-white shadow-lg"
onclick={() => (settings = !settings)}
>
  <Settings />
</button>

{#if settings}
  <div class="mt-2 w-52 shadow-lg p-4 h-32 rounded bg-white">
            <label for="limit" class="block mb-2 text-sm font-medium text-gray-900 ">Limit</label>
            <input bind:value={tmpLimit} type="text" id="limit" class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 " placeholder="100" required />

          
            <button onclick={submit}>Submit</button>
  </div>
{/if}
</div>





<LeftSidebar {realtime} {leftSidebar} />

<RightSidebar {data} {rightSidebar} />
