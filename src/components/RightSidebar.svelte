<script lang="ts">
    import {X} from "lucide-svelte";
    import * as echarts from "echarts";
    import {pieRegions} from "../charts/pie-regions/options";
    import {magDistributionOption} from "../charts/magnitude-distribution/options";
    import {magDepthScatterOption} from "../charts/magnitude-depth-scatter/options";
    import {onMount} from "svelte";
    import {mapToPieRegions} from "../charts/pie-regions/map";
    import {invoke} from "@tauri-apps/api/core";
  import { getCountByYearOptions } from "../charts/get_count_by_year/options";
  import { getHourlyFrequencyOptions } from "../charts/get_hourly_frequency/options";
  import { getCoordinateClusters } from "../charts/get_coordinate_clusters/options";
  import { world } from '../world';
  import { getWeeklyFrequencyOptions } from "../charts/get_weekly_frequency/options";
  import { getMonthlyFrequencyOptions } from "../charts/get_monthly_frequency/options";


    let {rightSidebar, data} = $props()

    let chartDom: HTMLElement;
    let chartDom2: HTMLElement;
    let chartDom3: HTMLElement;
    let chartDom4: HTMLElement;
    let chartDom5: HTMLElement;
    let chartDom6: HTMLElement;
    let chartDom7: HTMLElement;
    let chartDom8: HTMLElement;

    let analytics: any = $state(null);


    let magnitude = $state({});
    let count_by_year = $state({});
    let magDepthPairs = $state({});

    const pieRegionsData = mapToPieRegions(data.features);

    onMount(async () => {

        echarts.registerMap('world', world);


        let chart = echarts.init(chartDom);
        let chart2 = echarts.init(chartDom2);
        let chart3 = echarts.init(chartDom3);
        let chart4 = echarts.init(chartDom4);
        let chart5 = echarts.init(chartDom5);
        let chart6 = echarts.init(chartDom6);
        let chart7 = echarts.init(chartDom7);
        let chart8 = echarts.init(chartDom8);
        

        magnitude = await invoke("get_magnitude_distribution");
        const magDistributionOptionObj = magDistributionOption(magnitude);
        magDistributionOptionObj && chart2.setOption(magDistributionOptionObj);

        const pieRegionsObj = pieRegions(pieRegionsData);
        pieRegionsObj && chart.setOption(pieRegionsObj);


        magDepthPairs = await invoke("get_mag_depth_pairs");
        const magDepthPairsOptions = magDepthScatterOption(magDepthPairs);
        magDepthPairsOptions && chart3.setOption(magDepthPairsOptions);

        count_by_year = await invoke("get_count_by_year");
        const countByYearOption = getCountByYearOptions(count_by_year);
        countByYearOption && chart4.setOption(countByYearOption);

        const get_hourly_frequency = await invoke('get_hourly_frequency');
        const hourlyFrequencyOption = getHourlyFrequencyOptions(get_hourly_frequency);
        hourlyFrequencyOption && chart5.setOption(hourlyFrequencyOption);


        const get_coordinate_clusters = await invoke('get_coordinate_clusters');
        const getCordinatesClusters = getCoordinateClusters(get_coordinate_clusters);
        getCordinatesClusters && chart6.setOption(getCordinatesClusters);


        const get_weekly_frequency = await invoke('get_weekly_frequency');
        const getWeeklyFrequency = getWeeklyFrequencyOptions(get_weekly_frequency);
        getWeeklyFrequency && chart7.setOption(getWeeklyFrequency);

        const get_monthly_frequency = await invoke('get_monthly_frequency');

        console.error(get_monthly_frequency);
        const getMonthlyFrequency = getMonthlyFrequencyOptions(get_monthly_frequency);
        getMonthlyFrequency && chart8.setOption(getMonthlyFrequency);



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

        <div class="w-96 pr-2 h-96" bind:this={chartDom4} id="chart4"></div>

        <div class="w-96 pr-2 h-96" bind:this={chartDom5} id="chart5"></div>

        <div class="w-96 pr-2 h-96" bind:this={chartDom6} id="chart6"></div>

        <div class="w-96 pr-2 h-96" bind:this={chartDom7} id="chart7"></div>

        <div class="w-96 pr-2 h-96" bind:this={chartDom8} id="chart8"></div>
    </div>
</aside>
