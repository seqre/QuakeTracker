# QuakeTracker Tauri Commands Documentation

This document provides a comprehensive guide to all available Tauri commands in the QuakeTracker application. These commands can be invoked from the frontend using Tauri's `invoke` function.

## Table of Contents

1. [Data Retrieval Commands](#data-retrieval-commands)
2. [Analytics Commands](#analytics-commands)
3. [Temporal Analysis Commands](#temporal-analysis-commands)
4. [Geographic Analysis Commands](#geographic-analysis-commands)
5. [Seismic Risk Assessment Commands](#seismic-risk-assessment-commands)



## Data Retrieval Commands

### `get_seismic_events`

Fetches seismic events from the EMSC (European-Mediterranean Seismological Centre) API with optional filtering parameters.

**Parameters:**
- `query_params`: Object containing query parameters
- `clear`: Boolean to clear existing data before adding new events

**Query Parameters Object:**
```typescript
interface QueryParams {
  // Time constraints
  start_time?: string;  // ISO 8601 format
  end_time?: string;    // ISO 8601 format
  
  // Geographic constraints (bounding box)
  min_latitude?: number;
  max_latitude?: number;
  min_longitude?: number;
  max_longitude?: number;
  
  // Geographic constraints (circular)
  latitude?: number;
  longitude?: number;
  min_radius?: number;  // meters
  max_radius?: number;  // meters
  
  // Event filters
  min_depth?: number;      // kilometers
  max_depth?: number;      // kilometers
  min_magnitude?: number;
  max_magnitude?: number;
  magnitude_type?: string; // "Mw", "ML", "mb", etc.
  
  // Query options
  limit?: number;          // default: 10
  offset?: number;
  order_by?: string;       // "time", "time-asc", "magnitude", "magnitude-asc"
  contributor?: string;
  catalog?: string;
  event_id?: string;
  updated_after?: string;  // ISO 8601 format
  include_all_origins?: boolean;
  include_arrivals?: boolean;
}
```

**Frontend Usage:**
```javascript
import { invoke } from '@tauri-apps/api/tauri';

// Fetch recent earthquakes with magnitude >= 4.0
const queryParams = {
  min_magnitude: 4.0,
  limit: 50,
  order_by: "time"
};

try {
  const events = await invoke('get_seismic_events', {
    queryParams,
    clear: true
  });
  console.log('Seismic events:', events);
} catch (error) {
  console.error('Error fetching events:', error);
}
```

**Example Output:**
```json
{
  "type": "FeatureCollection",
  "features": [
    {
      "type": "Feature",
      "geometry": {
        "type": "Point",
        "coordinates": [-155.4875, 18.8232, -16.1]
      },
      "id": "20241210_0000315",
      "properties": {
        "source_id": "1741830",
        "source_catalog": "EMSC-RTS",
        "lastupdate": "2024-12-10T22:30:25.164009Z",
        "time": "2024-12-10T22:28:31.49Z",
        "flynn_region": "HAWAII REGION, HAWAII",
        "lat": 18.8232,
        "lon": -155.4875,
        "depth": 16.1,
        "evtype": "ke",
        "auth": "HV",
        "mag": 4.2,
        "magtype": "md",
        "unid": "20241210_0000315"
      }
    }
  ]
}
```


### `listen_to_seismic_events`

Establishes a WebSocket connection to receive real-time seismic events from EMSC.

**Parameters:**
- `on_event`: Channel callback function to handle incoming events

**Frontend Usage:**
```javascript
import { invoke } from '@tauri-apps/api/tauri';
import { Channel } from '@tauri-apps/api/core';

// Create a channel to receive real-time events
const eventChannel = new Channel();

// Set up event listener
eventChannel.onmessage = (event) => {
  console.log('Real-time earthquake event:', event);
  // Handle the event (update UI, show notification, etc.)
  updateEarthquakeMap(event.data);
};

// Start listening to real-time events
try {
  await invoke('listen_to_seismic_events', {
    onEvent: eventChannel
  });
} catch (error) {
  console.error('Error starting real-time listener:', error);
}
```

**Example Event Output:**
```json
{
  "action": "create",
  "data": {
    "geometry": {
      "type": "Point",
      "coordinates": [7.8865, 46.0554, -8.0]
    },
    "id": "20241214_0000249",
    "properties": {
      "source_id": "1744000",
      "source_catalog": "EMSC-RTS",
      "lastupdate": "2024-12-15T18:26:38.787209Z",
      "time": "2024-12-14T09:39:47.2Z",
      "flynn_region": "SWITZERLAND",
      "lat": 46.0554,
      "lon": 7.8865,
      "depth": 8.0,
      "evtype": "ke",
      "auth": "ETHZ",
      "mag": 3.1,
      "magtype": "ml",
      "unid": "20241214_0000249"
    }
  }
}
```



## Analytics Commands

### `get_magnitude_distribution`

Returns the distribution of earthquake magnitudes grouped into buckets.

**Parameters:** None

**Frontend Usage:**
```javascript
const distribution = await invoke('get_magnitude_distribution');
console.log('Magnitude distribution:', distribution);
```

**Example Output:**
```json
[
  ["2.0", 45],
  ["2.2", 38],
  ["2.4", 32],
  ["2.6", 28],
  ["2.8", 24],
  ["3.0", 20],
  ["3.2", 16],
  ["3.4", 12],
  ["3.6", 8],
  ["3.8", 6],
  ["4.0", 4],
  ["4.2", 2],
  ["4.4", 1]
]
```

### `get_advanced_analytics`

Returns comprehensive analytics computed using Polars for advanced statistical analysis.

**Parameters:** None

**Frontend Usage:**
```javascript
const analytics = await invoke('get_advanced_analytics');
console.log('Advanced analytics:', analytics);
```

**Example Output:**
```json
{
  "stats": [
    {
      "title": "Magnitude Statistics",
      "data": {
        "mean_magnitude": 3.2,
        "median_magnitude": 3.1,
        "std_magnitude": 0.8,
        "min_magnitude": 1.5,
        "max_magnitude": 6.2
      }
    },
    {
      "title": "Temporal Patterns Analysis",
      "data": [
        {"date": "2024-12-01", "daily_count": 12},
        {"date": "2024-12-02", "daily_count": 8},
        {"date": "2024-12-03", "daily_count": 15}
      ]
    },
    {
      "title": "Depth Statistics",
      "data": {
        "mean_depth": 15.6,
        "median_depth": 12.3,
        "std_depth": 8.9,
        "min_depth": 0.1,
        "max_depth": 45.2
      }
    },
    {
      "title": "Regional Analysis",
      "data": [
        {"flynn_region": "SOUTHERN CALIFORNIA", "event_count": 45, "avg_magnitude": 3.1},
        {"flynn_region": "CENTRAL ITALY", "event_count": 32, "avg_magnitude": 2.8},
        {"flynn_region": "GREECE", "event_count": 28, "avg_magnitude": 3.3}
      ]
    }
  ]
}
```

### `get_data_stats`

Returns current data statistics including total events, last update time, and memory usage.

**Parameters:** None

**Frontend Usage:**
```javascript
const stats = await invoke('get_data_stats');
console.log('Data statistics:', stats);
```

**Example Output:**
```json
{
  "total_events": 1247,
  "last_updated": "2024-12-15T10:30:45.123Z",
  "memory_usage_estimate": 623500
}
```

### `recompute_analytics`

Forces a full recomputation of all analytics. Useful after data cleanup or when analytics seem inconsistent.

**Parameters:** None

**Frontend Usage:**
```javascript
try {
  await invoke('recompute_analytics');
  console.log('Analytics recomputed successfully');
} catch (error) {
  console.error('Error recomputing analytics:', error);
}
```



## Temporal Analysis Commands

### `get_count_by_year`

Returns daily earthquake counts for time series analysis.

**Parameters:** None

**Frontend Usage:**
```javascript
const dailyCounts = await invoke('get_count_by_year');
console.log('Daily earthquake counts:', dailyCounts);

// Use for time series charts
const chartData = dailyCounts.map(([date, count]) => ({
  x: new Date(date),
  y: count
}));
```

**Example Output:**
```json
[
  ["2024-12-01", 12],
  ["2024-12-02", 8],
  ["2024-12-03", 15],
  ["2024-12-04", 6],
  ["2024-12-05", 11]
]
```

### `get_hourly_frequency`

Returns the distribution of earthquakes by hour of day (0-23).

**Parameters:** None

**Frontend Usage:**
```javascript
const hourlyFreq = await invoke('get_hourly_frequency');
console.log('Hourly frequency:', hourlyFreq);

// Create hourly distribution chart
const hourlyChart = hourlyFreq.map(([hour, count]) => ({
  hour: `${hour}:00`,
  count
}));
```

**Example Output:**
```json
[
  [0, 8],
  [1, 6],
  [2, 4],
  [3, 7],
  [4, 9],
  [5, 12],
  [6, 15],
  [7, 18],
  [8, 22],
  [9, 25],
  [10, 28],
  [11, 24],
  [12, 26],
  [13, 23],
  [14, 21],
  [15, 19],
  [16, 17],
  [17, 16],
  [18, 14],
  [19, 13],
  [20, 11],
  [21, 10],
  [22, 9],
  [23, 7]
]
```

### `get_monthly_frequency`

Returns the distribution of earthquakes by month (1-12).

**Parameters:** None

**Frontend Usage:**
```javascript
const monthlyFreq = await invoke('get_monthly_frequency');
console.log('Monthly frequency:', monthlyFreq);
```

**Example Output:**
```json
[
  [1, 145],
  [2, 132],
  [3, 156],
  [4, 148],
  [5, 162],
  [6, 158],
  [7, 171],
  [8, 169],
  [9, 154],
  [10, 147],
  [11, 139],
  [12, 143]
]
```

### `get_weekly_frequency`

Returns the distribution of earthquakes by day of week.

**Parameters:** None

**Frontend Usage:**
```javascript
const weeklyFreq = await invoke('get_weekly_frequency');
console.log('Weekly frequency:', weeklyFreq);
```

**Example Output:**
```json
[
  ["Mon", 156],
  ["Tue", 148],
  ["Wed", 162],
  ["Thu", 159],
  ["Fri", 154],
  ["Sat", 147],
  ["Sun", 151]
]
```



## Geographic Analysis Commands

### `get_region_hotspots`

Returns the most seismically active regions ranked by earthquake count.

**Parameters:** None

**Frontend Usage:**
```javascript
const hotspots = await invoke('get_region_hotspots');
console.log('Regional hotspots:', hotspots);

// Create a ranked list for display
const topRegions = hotspots.slice(0, 10).map(([region, count], index) => ({
  rank: index + 1,
  region,
  count
}));
```

**Example Output:**
```json
[
  ["SOUTHERN CALIFORNIA", 245],
  ["CENTRAL ITALY", 198],
  ["GREECE", 187],
  ["TURKEY", 156],
  ["JAPAN REGION", 134],
  ["CHILE", 112],
  ["ALASKA", 98],
  ["IRAN", 87],
  ["INDONESIA", 76],
  ["NEW ZEALAND", 65]
]
```

### `get_coordinate_clusters`

Returns geographic coordinate clusters for mapping earthquake hotspots.

**Parameters:** None

**Frontend Usage:**
```javascript
const clusters = await invoke('get_coordinate_clusters');
console.log('Coordinate clusters:', clusters);

// Use for heatmap or cluster markers on map
const mapClusters = clusters.map(([lat, lon, count]) => ({
  position: [lat, lon],
  weight: count,
  radius: Math.min(count * 2, 50) // Scale radius by count
}));
```

**Example Output:**
```json
[
  [35.0, -120.0, 45],
  [38.5, 22.5, 32],
  [40.0, 14.5, 28],
  [35.5, 139.5, 25],
  [41.0, 29.0, 22],
  [36.0, -118.0, 18],
  [37.5, 15.0, 15]
]
```



## Seismic Risk Assessment Commands

### `get_mag_depth_pairs`

Returns magnitude-depth pairs for correlation analysis and scatter plots.

**Parameters:** None

**Frontend Usage:**
```javascript
const magDepthPairs = await invoke('get_mag_depth_pairs');
console.log('Magnitude-depth pairs:', magDepthPairs);

// Use for scatter plot
const scatterData = magDepthPairs.map(([magnitude, depth]) => ({
  x: magnitude,
  y: depth
}));
```

**Example Output:**
```json
[
  [2.1, 5.2],
  [3.4, 12.8],
  [2.8, 8.1],
  [4.2, 15.6],
  [3.1, 9.3],
  [5.1, 22.4],
  [2.5, 6.7],
  [3.8, 14.2]
]
```

### `get_b_value`

Returns the Gutenberg-Richter b-value, which indicates the stress state of the region.

**Parameters:** None

**Frontend Usage:**
```javascript
const bValue = await invoke('get_b_value');
console.log('B-value:', bValue);

// Interpret b-value for user
let interpretation;
if (bValue < 0.8) {
  interpretation = "High stress environment - more large earthquakes expected";
} else if (bValue > 1.2) {
  interpretation = "Low stress environment - small earthquakes dominate";
} else {
  interpretation = "Normal stress environment - typical earthquake distribution";
}
```

**Example Output:**
```json
0.95
```

### `get_magnitude_frequency_data`

Returns magnitude-frequency relationship data for Gutenberg-Richter analysis.

**Parameters:** None

**Frontend Usage:**
```javascript
const magFreqData = await invoke('get_magnitude_frequency_data');
console.log('Magnitude-frequency data:', magFreqData);

// Use for Gutenberg-Richter plot
const grData = magFreqData.map(([magnitude, count, cumulative]) => ({
  magnitude,
  count,
  cumulative,
  logCumulative: Math.log10(cumulative)
}));
```

**Example Output:**
```json
[
  [2.0, 45, 156],
  [2.2, 38, 111],
  [2.4, 32, 73],
  [2.6, 28, 41],
  [2.8, 24, 13],
  [3.0, 20, 8],
  [3.2, 16, 4],
  [3.4, 12, 2],
  [3.6, 8, 1],
  [3.8, 6, 1]
]
```

### `get_risk_metrics`

Returns comprehensive risk assessment metrics including probabilities and energy.

**Parameters:** None

**Frontend Usage:**
```javascript
const [prob5_30, prob6_365, prob7_365, totalEnergy] = await invoke('get_risk_metrics');

console.log('Risk metrics:', {
  probabilityMag5In30Days: prob5_30,
  probabilityMag6In365Days: prob6_365,
  probabilityMag7In365Days: prob7_365,
  totalEnergyJoules: totalEnergy
});

// Display risk levels
const riskLevel = prob6_365 > 0.1 ? 'High' : prob6_365 > 0.05 ? 'Moderate' : 'Low';
```

**Example Output:**
```json
[0.15, 0.08, 0.02, 1.23e15]
```

### `get_total_energy`

Returns the total seismic energy released by all earthquakes in Joules.

**Parameters:** None

**Frontend Usage:**
```javascript
const totalEnergy = await invoke('get_total_energy');
console.log('Total energy (Joules):', totalEnergy);

// Convert to more readable units
const energyInTNT = totalEnergy / 4.184e9; // Convert to TNT equivalent
const energyInMegatons = energyInTNT / 1e12; // Convert to megatons
```

**Example Output:**
```json
1.234567e15
```