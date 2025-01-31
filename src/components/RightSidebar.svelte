<script lang="ts">
    import {X} from "lucide-svelte";
    import * as echarts from "echarts";
    import {pieRegions} from "../charts/pie-regions/options";
    import {magDistributionOption} from "../charts/magnitude-distribution/options";
    import {magDepthScatterOption} from "../charts/magnitude-depth-scatter/options";
    import {onMount} from "svelte";
    import {mapToPieRegions} from "../charts/pie-regions/map";
    import {invoke} from "@tauri-apps/api/core";

    let {rightSidebar, data} = $props()

    let chartDom: HTMLElement;
    let chartDom2: HTMLElement;
    let chartDom3: HTMLElement;

    let magnitude = $state({});
    let count_by_year = $state("");
    let magDepthPairs = $state({});

    const pieRegionsData = mapToPieRegions(data.features);

    onMount(async () => {
        let chart = echarts.init(chartDom);
        let chart2 = echarts.init(chartDom2);
        let chart3 = echarts.init(chartDom3);

        magnitude = await invoke("get_magnitude_distribution");
        const magDistributionOptionObj = magDistributionOption(magnitude);
        magDistributionOptionObj && chart2.setOption(magDistributionOptionObj);

        const pieRegionsObj = pieRegions(pieRegionsData);
        pieRegionsObj && chart.setOption(pieRegionsObj);

        count_by_year = JSON.stringify(await invoke("get_count_by_year"));

        magDepthPairs = await invoke("get_mag_depth_pairs");
        const magDepthPairsOptions = magDepthScatterOption(magDepthPairs);
        magDepthPairsOptions && chart3.setOption(magDepthPairsOptions);
    })

</script>


<aside
        class:hidden={rightSidebar}
        class=" pt-5 h-full fixed bg-white right-0 top-0 z-50 shadow-lg overflow-scroll"
>
    <button onclick={() => (rightSidebar = !rightSidebar)} class="absolute right-3 top-2">
        <X/>
    </button>

    <div class=" w-96 overflow-y-scroll">
        <div class="w-96 pr-2 h-96" bind:this={chartDom2} id="chart2"></div>

        <div  class="chart w-96 h-96" bind:this={chartDom} id="chart"></div>

        <div class="w-96 pr-2 h-96" bind:this={chartDom3} id="chart3"></div>
    </div>
</aside>
