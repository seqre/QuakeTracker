<script lang="ts">
  import { X } from "lucide-svelte";
  import * as echarts from "echarts";
  import { pieRegions } from "../charts/pie-regions/options";
  import { magDistributionOption } from "../charts/magnitude-distribution/options";
  import { magDepthScatterOption } from "../charts/magnitude-depth-scatter/options";
  import { onMount } from "svelte";
  import { mapToPieRegions } from "../charts/pie-regions/map";
  import { invoke } from "@tauri-apps/api/core";
  import { getCountByYearOptions } from "../charts/get_count_by_year/options";
  import { getHourlyFrequencyOptions } from "../charts/get_hourly_frequency/options";
  import { getCoordinateClusters } from "../charts/get_coordinate_clusters/options";
  import { world } from "../world";
  import { getWeeklyFrequencyOptions } from "../charts/get_weekly_frequency/options";
  import { getMonthlyFrequencyOptions } from "../charts/get_monthly_frequency/options";
  import { magFreqDataFunc } from "../charts/frequneces/options";
  import AnalyticsTables from "./AnalyticsTables.svelte";

  let { rightSidebar, data } = $props();

  let chartDom: HTMLElement;
  let chartDom2: HTMLElement;
  let chartDom3: HTMLElement;
  let chartDom4: HTMLElement;
  let chartDom5: HTMLElement;
  let chartDom6: HTMLElement;
  let chartDom7: HTMLElement;
  let chartDom8: HTMLElement;
  let chartDom9: HTMLElement;

  let bValue: number = $state(0);
  let riskLevel: string = $state("Low");
  let interpretation: string = $state(
    "Normal stress environment - typical earthquake distribution"
  );

  let analytics: any = $state(null);

  let magnitude = $state({});
  let count_by_year = $state({});
  let magDepthPairs = $state({});

  const pieRegionsData = mapToPieRegions(data.features);

  onMount(async () => {
    echarts.registerMap("world", world);

    let chart = echarts.init(chartDom);
    let chart2 = echarts.init(chartDom2);
    let chart3 = echarts.init(chartDom3);
    let chart4 = echarts.init(chartDom4);
    let chart5 = echarts.init(chartDom5);
    let chart6 = echarts.init(chartDom6);
    let chart7 = echarts.init(chartDom7);
    let chart8 = echarts.init(chartDom8);
    let chart9 = echarts.init(chartDom9);

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

    const get_hourly_frequency = await invoke("get_hourly_frequency");
    const hourlyFrequencyOption =
      getHourlyFrequencyOptions(get_hourly_frequency);
    hourlyFrequencyOption && chart5.setOption(hourlyFrequencyOption);

    const get_coordinate_clusters = await invoke("get_coordinate_clusters");
    const getCordinatesClusters = getCoordinateClusters(
      get_coordinate_clusters
    );
    getCordinatesClusters && chart6.setOption(getCordinatesClusters);

    const get_weekly_frequency = await invoke("get_weekly_frequency");
    const getWeeklyFrequency = getWeeklyFrequencyOptions(get_weekly_frequency);
    getWeeklyFrequency && chart7.setOption(getWeeklyFrequency);

    const get_monthly_frequency = await invoke("get_monthly_frequency");

    const getMonthlyFrequency = getMonthlyFrequencyOptions(
      get_monthly_frequency
    );
    getMonthlyFrequency && chart8.setOption(getMonthlyFrequency);

    const magFreqData: any = await invoke("get_magnitude_frequency_data");
    const option = magFreqDataFunc(magFreqData);
    option && chart9.setOption(option);




    const totalEnergy = await invoke("get_total_energy");
    console.log("Total energy (Joules):", totalEnergy);

    const [prob5_30, prob6_365, prob7_365, totalEnergy2]: any =
      await invoke("get_risk_metrics");
    console.log("Risk metrics:", {
      probabilityMag5In30Days: prob5_30,
      probabilityMag6In365Days: prob6_365,
      probabilityMag7In365Days: prob7_365,
      totalEnergyJoules: totalEnergy,
    });

    bValue = await invoke("get_b_value");
    console.log("B-value:", bValue);

    // Interpret b-value for user

    if (bValue < 0.8) {
      interpretation =
        "High stress environment - more large earthquakes expected";
    } else if (bValue > 1.2) {
      interpretation = "Low stress environment - small earthquakes dominate";
    } else {
      interpretation =
        "Normal stress environment - typical earthquake distribution";
    }

    // Display risk levels
    riskLevel =
      prob6_365 > 0.1 ? "High" : prob6_365 > 0.05 ? "Moderate" : "Low";

    const stats = await invoke("get_data_stats");
    console.log("Data statistics:", stats);

    analytics = await invoke("get_advanced_analytics");
    console.log("Advanced analytics:", analytics);


    // // Use for Gutenberg-Richter plot
    // const grData = magFreqData.map(([magnitude, count, cumulative]) => ({
    //   magnitude,
    //   count,
    //   cumulative,
    //   logCumulative: Math.log10(cumulative),
    // }));
  });
</script>

<aside
  class:hidden={rightSidebar}
  class=" pt-5 h-full fixed bg-white right-0 top-0 z-50 shadow-lg overflow-scroll"
>
  <button
    onclick={() => (rightSidebar = !rightSidebar)}
    class="absolute right-3 top-2"
  >
    <X />
  </button>

  <div class=" w-96 overflow-y-scroll">
    <div class="w-96 pr-2 h-96" bind:this={chartDom2} id="chart2"></div>

    <div class="chart w-96 h-96" bind:this={chartDom} id="chart"></div>

    <div class="w-96 pr-2 h-96" bind:this={chartDom3} id="chart3"></div>

    <div class="w-96 pr-2 h-96" bind:this={chartDom4} id="chart4"></div>

    <div class="w-96 pr-2 h-96" bind:this={chartDom5} id="chart5"></div>

    <div class="w-96 pr-2 h-96" bind:this={chartDom6} id="chart6"></div>

    <div class="w-96 pr-2 h-96" bind:this={chartDom7} id="chart7"></div>

    <div class="w-96 pr-2 h-96" bind:this={chartDom8} id="chart8"></div>

    <div class="w-96 pr-2 h-96" bind:this={chartDom9} id="chart9"></div>

    <div class="relative overflow-x-auto mt-6 mx-4">
      <table class="w-full text-sm text-left text-gray-500">
        <thead class="text-xs text-gray-700 uppercase bg-gray-50">
          <tr>
            <th scope="col" class="px-6 py-3"> Metric </th>
            <th scope="col" class="px-6 py-3"> Value </th>
            <th scope="col" class="px-6 py-3"> Interpretation </th>
          </tr>
        </thead>
        <tbody>
          <tr class="bg-white border-b border-gray-200">
            <th
              scope="row"
              class="px-6 py-4 font-medium text-gray-900 whitespace-nowrap"
            >
              B-value
            </th>
            <td class="px-6 py-4">
              {Math.round(bValue * 100) / 100}
            </td>
            <td class="px-6 py-4">
              {interpretation}
            </td>
          </tr>
          <tr class="bg-white">
            <th
              scope="row"
              class="px-6 py-4 font-medium text-gray-900 whitespace-nowrap"
            >
              Risk Level
            </th>
            <td class="px-6 py-4">
              {riskLevel}
            </td>
            <td class="px-6 py-4">
              {riskLevel === "High"
                ? "Increased likelihood of significant earthquakes"
                : riskLevel === "Moderate"
                  ? "Potential for moderate earthquakes"
                  : "Low likelihood of significant earthquakes"}
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <AnalyticsTables {analytics} />
  </div>
</aside>
