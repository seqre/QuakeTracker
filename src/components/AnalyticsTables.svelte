<script lang="ts">
  let { analytics } = $props();
</script>

<!-- Magnitude Statistics Table -->
{#if analytics?.stats}
  {#each analytics.stats as stat}
    {#if stat.title === "Magnitude Statistics"}
      <div class="relative overflow-x-auto mt-6 mx-4">
        <h3 class="text-lg font-semibold text-gray-900 mb-3">{stat.title}</h3>
        <table class="w-full text-sm text-left text-gray-500">
          <thead class="text-xs text-gray-700 uppercase bg-gray-50">
            <tr>
              <th scope="col" class="px-6 py-3">Statistic</th>
              <th scope="col" class="px-6 py-3">Value</th>
            </tr>
          </thead>
          <tbody>
            <tr class="bg-white border-b border-gray-200">
              <td class="px-6 py-4 font-medium text-gray-900">Mean Magnitude</td>
              <td class="px-6 py-4">{stat.data.mean_magnitude?.toFixed(2)}</td>
            </tr>
            <tr class="bg-white border-b border-gray-200">
              <td class="px-6 py-4 font-medium text-gray-900">Median Magnitude</td>
              <td class="px-6 py-4">{stat.data.median_magnitude?.toFixed(2)}</td>
            </tr>
            <tr class="bg-white border-b border-gray-200">
              <td class="px-6 py-4 font-medium text-gray-900">Standard Deviation</td>
              <td class="px-6 py-4">{stat.data.std_magnitude?.toFixed(2)}</td>
            </tr>
            <tr class="bg-white border-b border-gray-200">
              <td class="px-6 py-4 font-medium text-gray-900">Min Magnitude</td>
              <td class="px-6 py-4">{stat.data.min_magnitude?.toFixed(1)}</td>
            </tr>
            <tr class="bg-white">
              <td class="px-6 py-4 font-medium text-gray-900">Max Magnitude</td>
              <td class="px-6 py-4">{stat.data.max_magnitude?.toFixed(1)}</td>
            </tr>
          </tbody>
        </table>
      </div>
    {/if}

    <!-- Depth Statistics Table -->
    {#if stat.title === "Depth Statistics"}
      <div class="relative overflow-x-auto mt-6 mx-4">
        <h3 class="text-lg font-semibold text-gray-900 mb-3">{stat.title}</h3>
        <table class="w-full text-sm text-left text-gray-500">
          <thead class="text-xs text-gray-700 uppercase bg-gray-50">
            <tr>
              <th scope="col" class="px-6 py-3">Statistic</th>
              <th scope="col" class="px-6 py-3">Value (km)</th>
            </tr>
          </thead>
          <tbody>
            <tr class="bg-white border-b border-gray-200">
              <td class="px-6 py-4 font-medium text-gray-900">Mean Depth</td>
              <td class="px-6 py-4">{stat.data.mean_depth?.toFixed(1)}</td>
            </tr>
            <tr class="bg-white border-b border-gray-200">
              <td class="px-6 py-4 font-medium text-gray-900">Median Depth</td>
              <td class="px-6 py-4">{stat.data.median_depth?.toFixed(1)}</td>
            </tr>
            <tr class="bg-white border-b border-gray-200">
              <td class="px-6 py-4 font-medium text-gray-900">Standard Deviation</td>
              <td class="px-6 py-4">{stat.data.std_depth?.toFixed(1)}</td>
            </tr>
            <tr class="bg-white border-b border-gray-200">
              <td class="px-6 py-4 font-medium text-gray-900">Min Depth</td>
              <td class="px-6 py-4">{stat.data.min_depth?.toFixed(1)}</td>
            </tr>
            <tr class="bg-white">
              <td class="px-6 py-4 font-medium text-gray-900">Max Depth</td>
              <td class="px-6 py-4">{stat.data.max_depth?.toFixed(1)}</td>
            </tr>
          </tbody>
        </table>
      </div>
    {/if}

    <!-- Regional Analysis Table -->
    {#if stat.title === "Regional Analysis"}
      <div class="relative overflow-x-auto mt-6 mx-4">
        <h3 class="text-lg font-semibold text-gray-900 mb-3">{stat.title}</h3>
        <table class="w-full text-sm text-left text-gray-500">
          <thead class="text-xs text-gray-700 uppercase bg-gray-50">
            <tr>
              <th scope="col" class="px-6 py-3">Region</th>
              <th scope="col" class="px-6 py-3">Event Count</th>
              <th scope="col" class="px-6 py-3">Avg Magnitude</th>
            </tr>
          </thead>
          <tbody>
            {#each stat.data as region, index}
              <tr class="bg-white {index < stat.data.length - 1 ? 'border-b border-gray-200' : ''}">
                <td class="px-6 py-4 font-medium text-gray-900 whitespace-nowrap">
                  {region.flynn_region}
                </td>
                <td class="px-6 py-4">{region.event_count}</td>
                <td class="px-6 py-4">{region.avg_magnitude?.toFixed(1)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}

    <!-- Temporal Patterns Analysis Table -->
    {#if stat.title === "Temporal Patterns Analysis"}
      <div class="relative overflow-x-auto mt-6 mx-4">
        <h3 class="text-lg font-semibold text-gray-900 mb-3">{stat.title}</h3>
        <div class="max-h-96 overflow-y-auto">
          <table class="w-full text-sm text-left text-gray-500">
            <thead class="text-xs text-gray-700 uppercase bg-gray-50 sticky top-0">
              <tr>
                <th scope="col" class="px-6 py-3">Date</th>
                <th scope="col" class="px-6 py-3">Daily Count</th>
              </tr>
            </thead>
            <tbody>
              {#each stat.data as dayData, index}
                <tr class="bg-white {index < stat.data.length - 1 ? 'border-b border-gray-200' : ''}">
                  <td class="px-6 py-4 font-medium text-gray-900">
                    {new Date(dayData.date).toLocaleDateString('pl-PL')}
                  </td>
                  <td class="px-6 py-4">{dayData.daily_count}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </div>
    {/if}
  {/each}
{/if}
