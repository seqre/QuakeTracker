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
  
  // Query parameters state
  let queryParams = $state({
    // Time constraints
    start_time: '',
    end_time: '',
    
    // Geographic constraints (bounding box)
    min_latitude: '',
    max_latitude: '',
    min_longitude: '',
    max_longitude: '',
    
    // Geographic constraints (circular)
    latitude: '',
    longitude: '',
    min_radius: '',
    max_radius: '',
    
    // Event filters
    min_depth: '',
    max_depth: '',
    min_magnitude: '',
    max_magnitude: '',
    magnitude_type: '',
    
    // Query options
    limit: '100',
    offset: '',
    order_by: '',
    contributor: '',
    catalog: '',
    event_id: '',
    updated_after: '',
    include_all_origins: false,
    include_arrivals: false
  });

  // Checkboxes to enable/disable parameters
  let enabledParams = $state({
    start_time: false,
    end_time: false,
    min_latitude: false,
    max_latitude: false,
    min_longitude: false,
    max_longitude: false,
    latitude: false,
    longitude: false,
    min_radius: false,
    max_radius: false,
    min_depth: false,
    max_depth: false,
    min_magnitude: false,
    max_magnitude: false,
    magnitude_type: false,
    limit: true,
    offset: false,
    order_by: false,
    contributor: false,
    catalog: false,
    event_id: false,
    updated_after: false,
    include_all_origins: false,
    include_arrivals: false
  });

  onMount(async () => {
    const PMTILES_URL = "./my_area.pmtiles";
    const map = await MapFunc({ data, PMTILES_URL, defaultTheme, realtime });

    // Load saved query params
    const savedParams = localStorage.getItem('queryParams');
    if (savedParams) {
      try {
        const parsed = JSON.parse(savedParams);
        queryParams = { ...queryParams, ...parsed };
      } catch (e) {
        console.error('Error parsing saved query params:', e);
      }
    }

    // Load enabled params
    const savedEnabledParams = localStorage.getItem('enabledParams');
    if (savedEnabledParams) {
      try {
        const parsed = JSON.parse(savedEnabledParams);
        enabledParams = { ...enabledParams, ...parsed };
      } catch (e) {
        console.error('Error parsing saved enabled params:', e);
      }
    }
  });

  let submit = async () => {
    // Create clean query params object with only enabled parameters
    const cleanParams: any = {};
    
    Object.keys(enabledParams).forEach(key => {
      if (enabledParams[key as keyof typeof enabledParams]) {
        const value = queryParams[key as keyof typeof queryParams];
        if (value !== '' && value !== undefined) {
          // Convert string numbers to actual numbers where appropriate
          if (['limit', 'offset', 'min_latitude', 'max_latitude', 'min_longitude', 'max_longitude',
               'latitude', 'longitude', 'min_radius', 'max_radius', 'min_depth', 'max_depth',
               'min_magnitude', 'max_magnitude'].includes(key) && typeof value === 'string') {
            const numValue = parseFloat(value);
            if (!isNaN(numValue)) {
              cleanParams[key] = numValue;
            }
          } else {
            cleanParams[key] = value;
          }
        }
      }
    });

    localStorage.setItem('queryParams', JSON.stringify(cleanParams));
    localStorage.setItem('enabledParams', JSON.stringify(enabledParams));
    location.reload();
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
  <div class="mt-2 w-96 max-h-[calc(100vh-6rem)] overflow-y-auto shadow-lg p-4 rounded bg-white">
    <h3 class="text-lg font-semibold mb-4">Query Parameters</h3>
    
    <!-- Time Constraints -->
    <div class="mb-4">
      <h4 class="font-medium text-gray-800 mb-2">Time Constraints</h4>
      
      <div class="flex items-center mb-2">
        <input type="checkbox" bind:checked={enabledParams.start_time} id="enable_start_time" class="mr-2">
        <label for="start_time" class="block text-sm font-medium text-gray-700">Start Time (ISO 8601)</label>
      </div>
      <input 
        bind:value={queryParams.start_time} 
        type="datetime-local" 
        id="start_time" 
        class="w-full p-2 border border-gray-300 rounded-md text-sm mb-2"
        disabled={!enabledParams.start_time}
      />
      
      <div class="flex items-center mb-2">
        <input type="checkbox" bind:checked={enabledParams.end_time} id="enable_end_time" class="mr-2">
        <label for="end_time" class="block text-sm font-medium text-gray-700">End Time (ISO 8601)</label>
      </div>
      <input 
        bind:value={queryParams.end_time} 
        type="datetime-local" 
        id="end_time" 
        class="w-full p-2 border border-gray-300 rounded-md text-sm mb-2"
        disabled={!enabledParams.end_time}
      />
    </div>

    <!-- Geographic Constraints (Bounding Box) -->
    <div class="mb-4">
      <h4 class="font-medium text-gray-800 mb-2">Geographic Constraints (Bounding Box)</h4>
      
      <div class="grid grid-cols-2 gap-2">
        <div>
          <div class="flex items-center mb-1">
            <input type="checkbox" bind:checked={enabledParams.min_latitude} id="enable_min_lat" class="mr-2">
            <label for="min_latitude" class="text-xs text-gray-700">Min Latitude</label>
          </div>
          <input 
            bind:value={queryParams.min_latitude} 
            type="number" 
            step="any"
            id="min_latitude" 
            class="w-full p-1 border border-gray-300 rounded text-xs"
            disabled={!enabledParams.min_latitude}
          />
        </div>
        
        <div>
          <div class="flex items-center mb-1">
            <input type="checkbox" bind:checked={enabledParams.max_latitude} id="enable_max_lat" class="mr-2">
            <label for="max_latitude" class="text-xs text-gray-700">Max Latitude</label>
          </div>
          <input 
            bind:value={queryParams.max_latitude} 
            type="number" 
            step="any"
            id="max_latitude" 
            class="w-full p-1 border border-gray-300 rounded text-xs"
            disabled={!enabledParams.max_latitude}
          />
        </div>
        
        <div>
          <div class="flex items-center mb-1">
            <input type="checkbox" bind:checked={enabledParams.min_longitude} id="enable_min_lng" class="mr-2">
            <label for="min_longitude" class="text-xs text-gray-700">Min Longitude</label>
          </div>
          <input 
            bind:value={queryParams.min_longitude} 
            type="number" 
            step="any"
            id="min_longitude" 
            class="w-full p-1 border border-gray-300 rounded text-xs"
            disabled={!enabledParams.min_longitude}
          />
        </div>
        
        <div>
          <div class="flex items-center mb-1">
            <input type="checkbox" bind:checked={enabledParams.max_longitude} id="enable_max_lng" class="mr-2">
            <label for="max_longitude" class="text-xs text-gray-700">Max Longitude</label>
          </div>
          <input 
            bind:value={queryParams.max_longitude} 
            type="number" 
            step="any"
            id="max_longitude" 
            class="w-full p-1 border border-gray-300 rounded text-xs"
            disabled={!enabledParams.max_longitude}
          />
        </div>
      </div>
    </div>

    <!-- Geographic Constraints (Circular) -->
    <div class="mb-4">
      <h4 class="font-medium text-gray-800 mb-2">Geographic Constraints (Circular)</h4>
      
      <div class="grid grid-cols-2 gap-2">
        <div>
          <div class="flex items-center mb-1">
            <input type="checkbox" bind:checked={enabledParams.latitude} id="enable_lat" class="mr-2">
            <label for="latitude" class="text-xs text-gray-700">Center Latitude</label>
          </div>
          <input 
            bind:value={queryParams.latitude} 
            type="number" 
            step="any"
            id="latitude" 
            class="w-full p-1 border border-gray-300 rounded text-xs"
            disabled={!enabledParams.latitude}
          />
        </div>
        
        <div>
          <div class="flex items-center mb-1">
            <input type="checkbox" bind:checked={enabledParams.longitude} id="enable_lng" class="mr-2">
            <label for="longitude" class="text-xs text-gray-700">Center Longitude</label>
          </div>
          <input 
            bind:value={queryParams.longitude} 
            type="number" 
            step="any"
            id="longitude" 
            class="w-full p-1 border border-gray-300 rounded text-xs"
            disabled={!enabledParams.longitude}
          />
        </div>
        
        <div>
          <div class="flex items-center mb-1">
            <input type="checkbox" bind:checked={enabledParams.min_radius} id="enable_min_radius" class="mr-2">
            <label for="min_radius" class="text-xs text-gray-700">Min Radius (m)</label>
          </div>
          <input 
            bind:value={queryParams.min_radius} 
            type="number" 
            id="min_radius" 
            class="w-full p-1 border border-gray-300 rounded text-xs"
            disabled={!enabledParams.min_radius}
          />
        </div>
        
        <div>
          <div class="flex items-center mb-1">
            <input type="checkbox" bind:checked={enabledParams.max_radius} id="enable_max_radius" class="mr-2">
            <label for="max_radius" class="text-xs text-gray-700">Max Radius (m)</label>
          </div>
          <input 
            bind:value={queryParams.max_radius} 
            type="number" 
            id="max_radius" 
            class="w-full p-1 border border-gray-300 rounded text-xs"
            disabled={!enabledParams.max_radius}
          />
        </div>
      </div>
    </div>

    <!-- Event Filters -->
    <div class="mb-4">
      <h4 class="font-medium text-gray-800 mb-2">Event Filters</h4>
      
      <div class="grid grid-cols-2 gap-2">
        <div>
          <div class="flex items-center mb-1">
            <input type="checkbox" bind:checked={enabledParams.min_depth} id="enable_min_depth" class="mr-2">
            <label for="min_depth" class="text-xs text-gray-700">Min Depth (km)</label>
          </div>
          <input 
            bind:value={queryParams.min_depth} 
            type="number" 
            step="any"
            id="min_depth" 
            class="w-full p-1 border border-gray-300 rounded text-xs"
            disabled={!enabledParams.min_depth}
          />
        </div>
        
        <div>
          <div class="flex items-center mb-1">
            <input type="checkbox" bind:checked={enabledParams.max_depth} id="enable_max_depth" class="mr-2">
            <label for="max_depth" class="text-xs text-gray-700">Max Depth (km)</label>
          </div>
          <input 
            bind:value={queryParams.max_depth} 
            type="number" 
            step="any"
            id="max_depth" 
            class="w-full p-1 border border-gray-300 rounded text-xs"
            disabled={!enabledParams.max_depth}
          />
        </div>
        
        <div>
          <div class="flex items-center mb-1">
            <input type="checkbox" bind:checked={enabledParams.min_magnitude} id="enable_min_mag" class="mr-2">
            <label for="min_magnitude" class="text-xs text-gray-700">Min Magnitude</label>
          </div>
          <input 
            bind:value={queryParams.min_magnitude} 
            type="number" 
            step="any"
            id="min_magnitude" 
            class="w-full p-1 border border-gray-300 rounded text-xs"
            disabled={!enabledParams.min_magnitude}
          />
        </div>
        
        <div>
          <div class="flex items-center mb-1">
            <input type="checkbox" bind:checked={enabledParams.max_magnitude} id="enable_max_mag" class="mr-2">
            <label for="max_magnitude" class="text-xs text-gray-700">Max Magnitude</label>
          </div>
          <input 
            bind:value={queryParams.max_magnitude} 
            type="number" 
            step="any"
            id="max_magnitude" 
            class="w-full p-1 border border-gray-300 rounded text-xs"
            disabled={!enabledParams.max_magnitude}
          />
        </div>
      </div>
      
      <div class="mt-2">
        <div class="flex items-center mb-1">
          <input type="checkbox" bind:checked={enabledParams.magnitude_type} id="enable_mag_type" class="mr-2">
          <label for="magnitude_type" class="text-xs text-gray-700">Magnitude Type</label>
        </div>
        <select 
          bind:value={queryParams.magnitude_type} 
          id="magnitude_type"
          class="w-full p-1 border border-gray-300 rounded text-xs"
          disabled={!enabledParams.magnitude_type}
        >
          <option value="">Select type</option>
          <option value="Mw">Mw (Moment magnitude)</option>
          <option value="ML">ML (Local magnitude)</option>
          <option value="mb">mb (Body wave magnitude)</option>
          <option value="Ms">Ms (Surface wave magnitude)</option>
          <option value="Md">Md (Duration magnitude)</option>
        </select>
      </div>
    </div>

    <!-- Query Options -->
    <div class="mb-4">
      <h4 class="font-medium text-gray-800 mb-2">Query Options</h4>
      
      <div class="grid grid-cols-2 gap-2">
        <div>
          <div class="flex items-center mb-1">
            <input type="checkbox" bind:checked={enabledParams.limit} id="enable_limit" class="mr-2">
            <label for="limit" class="text-xs text-gray-700">Limit</label>
          </div>
          <input 
            bind:value={queryParams.limit} 
            type="number" 
            id="limit" 
            class="w-full p-1 border border-gray-300 rounded text-xs"
            disabled={!enabledParams.limit}
          />
        </div>
        
        <div>
          <div class="flex items-center mb-1">
            <input type="checkbox" bind:checked={enabledParams.offset} id="enable_offset" class="mr-2">
            <label for="offset" class="text-xs text-gray-700">Offset</label>
          </div>
          <input 
            bind:value={queryParams.offset} 
            type="number" 
            id="offset" 
            class="w-full p-1 border border-gray-300 rounded text-xs"
            disabled={!enabledParams.offset}
          />
        </div>
      </div>
      
      <div class="mt-2">
        <div class="flex items-center mb-1">
          <input type="checkbox" bind:checked={enabledParams.order_by} id="enable_order_by" class="mr-2">
          <label for="order_by" class="text-xs text-gray-700">Order By</label>
        </div>
        <select 
          bind:value={queryParams.order_by} 
          id="order_by"
          class="w-full p-1 border border-gray-300 rounded text-xs"
          disabled={!enabledParams.order_by}
        >
          <option value="">Select order</option>
          <option value="time">Time (desc)</option>
          <option value="time-asc">Time (asc)</option>
          <option value="magnitude">Magnitude (desc)</option>
          <option value="magnitude-asc">Magnitude (asc)</option>
        </select>
      </div>
      
      <div class="mt-2">
        <div class="flex items-center mb-1">
          <input type="checkbox" bind:checked={enabledParams.contributor} id="enable_contributor" class="mr-2">
          <label for="contributor" class="text-xs text-gray-700">Contributor</label>
        </div>
        <input 
          bind:value={queryParams.contributor} 
          type="text" 
          id="contributor" 
          class="w-full p-1 border border-gray-300 rounded text-xs"
          disabled={!enabledParams.contributor}
        />
      </div>
      
      <div class="mt-2">
        <div class="flex items-center mb-1">
          <input type="checkbox" bind:checked={enabledParams.catalog} id="enable_catalog" class="mr-2">
          <label for="catalog" class="text-xs text-gray-700">Catalog</label>
        </div>
        <input 
          bind:value={queryParams.catalog} 
          type="text" 
          id="catalog" 
          class="w-full p-1 border border-gray-300 rounded text-xs"
          disabled={!enabledParams.catalog}
        />
      </div>
      
      <div class="mt-2">
        <div class="flex items-center mb-1">
          <input type="checkbox" bind:checked={enabledParams.event_id} id="enable_event_id" class="mr-2">
          <label for="event_id" class="text-xs text-gray-700">Event ID</label>
        </div>
        <input 
          bind:value={queryParams.event_id} 
          type="text" 
          id="event_id" 
          class="w-full p-1 border border-gray-300 rounded text-xs"
          disabled={!enabledParams.event_id}
        />
      </div>
      
      <div class="mt-2">
        <div class="flex items-center mb-1">
          <input type="checkbox" bind:checked={enabledParams.updated_after} id="enable_updated_after" class="mr-2">
          <label for="updated_after" class="text-xs text-gray-700">Updated After</label>
        </div>
        <input 
          bind:value={queryParams.updated_after} 
          type="datetime-local" 
          id="updated_after" 
          class="w-full p-1 border border-gray-300 rounded text-xs"
          disabled={!enabledParams.updated_after}
        />
      </div>
      
      <div class="mt-2 space-y-2">
        <div class="flex items-center">
          <input type="checkbox" bind:checked={enabledParams.include_all_origins} id="enable_include_all_origins" class="mr-2">
          <input type="checkbox" bind:checked={queryParams.include_all_origins} id="include_all_origins" class="mr-2" disabled={!enabledParams.include_all_origins}>
          <label for="include_all_origins" class="text-xs text-gray-700">Include All Origins</label>
        </div>
        
        <div class="flex items-center">
          <input type="checkbox" bind:checked={enabledParams.include_arrivals} id="enable_include_arrivals" class="mr-2">
          <input type="checkbox" bind:checked={queryParams.include_arrivals} id="include_arrivals" class="mr-2" disabled={!enabledParams.include_arrivals}>
          <label for="include_arrivals" class="text-xs text-gray-700">Include Arrivals</label>
        </div>
      </div>
    </div>

    <button 
      onclick={submit}
      class="w-full bg-blue-500 hover:bg-blue-600 text-white font-medium py-2 px-4 rounded"
    >
      Apply Settings
    </button>
  </div>
{/if}
</div>





<LeftSidebar {realtime} {leftSidebar} />

<RightSidebar {data} {rightSidebar} />
