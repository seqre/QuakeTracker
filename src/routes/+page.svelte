<script lang="ts">
  import { onMount } from "svelte";
  import { defaultTheme } from "../theme";
  import { type PageData } from "./$types";
  import "../app.css";
  import { Activity, ChartArea, ChartBar, ChartLine, X } from "lucide-svelte";
  import { MapFunc } from "./map-libre-init";
  import LeftSidebar from "../components/LeftSidebar.svelte";
  import RightSidebar from "../components/RightSidebar.svelte";

  let { data }: { data: PageData } = $props();

  let leftSidebar = $state(true);
  let rightSidebar = $state(true);

  let realtime: Array<PointData> = $state([]);

  onMount(async () => {
    const PMTILES_URL = "./my_area.pmtiles";
    const map = await MapFunc({ data, PMTILES_URL, defaultTheme, realtime });
  });

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
</div>

<LeftSidebar {realtime} {leftSidebar} />

<RightSidebar {data} {rightSidebar} />
