<script lang="ts">

    import maplibregl from 'maplibre-gl';
    import {PMTiles, Protocol} from 'pmtiles';
    import {onMount} from 'svelte';

    import {defaultTheme} from '../theme'
    import {type PageData} from './$types';
    import {Channel, invoke} from "@tauri-apps/api/core";
    import {info} from "@tauri-apps/plugin-log";

    import '../app.css'
    import NewPoint from '../components/NewPoint.svelte';
    import {Activity, X} from 'lucide-svelte';

    let {data}: { data: PageData } = $props();
    let sidebar = $state(false)
    let realtime: Array<PointData> = $state([])

    onMount(async () => {
        const protocol = new Protocol();
        maplibregl.addProtocol("pmtiles", (request) => {
            return new Promise((resolve, reject) => {
                const callback = (err: any, data: any) => {
                    if (err) {
                        reject(err);
                    } else {
                        resolve({data});
                    }
                };

                protocol.tile(request, callback);
            });
        });

        // PMTiles setup
        const PMTILES_URL = "./my_area.pmtiles";
        const pmtilesInstance = new PMTiles(PMTILES_URL);

        // this is so we share one instance across the JS code and the map renderer
        protocol.add(pmtilesInstance);

        // we first fetch the header so we can get the center lon, lat of the map.
        // const mapMetaData = await pmtilesInstance.getHeader();

        const map = new maplibregl.Map({
            container: 'map',
            minZoom: 1.3,
            maxZoom: 5,
            center: [0, 0],
            zoom: 0.2,
            style: {
                version: 8,
                sprite: "https://demotiles.maplibre.org/styles/osm-bright-gl-style/sprite",
                sources: {
                    'seismic-events': {
                        type: 'geojson',
                        data: {
                            type: 'FeatureCollection',
                            features: data.features,
                        }
                    },
                    'protomaps': {
                        type: 'vector',
                        url: `pmtiles://${PMTILES_URL}`,
                        attribution: '© <a href="https://openstreetmap.org">OpenStreetMap</a>'
                    }
                },
                glyphs: 'https://protomaps.github.io/basemaps-assets/fonts/{fontstack}/{range}.pbf',
                // simple map layer definitions
                // for more information about structure see
                // https://maplibre.org/maplibre-style-spec/layers/
                layers: [...defaultTheme,
                    {
                        id: "seismic-events-layer",
                        type: "circle",
                        source: "seismic-events",
                        paint: {
                            "circle-radius": [
                                "interpolate",
                                ["linear"],
                                ["get", "mag"],
                                0, 3,
                                10, 7
                            ],
                            "circle-color": [
                                "interpolate-hcl",
                                ["linear"],
                                ["get", "mag"],
                                0, "green",
                                5, "yellow",
                                10, "red"
                            ],
                            "circle-stroke-color": "gray",
                            "circle-stroke-width": 1,
                        },
                    },
                ]
            }
        });

        map.on('load', async () => {
            const image = await map.loadImage('https://upload.wikimedia.org/wikipedia/commons/7/7c/201408_cat.png');
            map.addImage('marker-15', image.data);
            console.log(map)
        })

        map.on('click', 'seismic-events-layer', (e) => {
            // @ts-ignore
            const coordinates = e.features[0]!.geometry.coordinates.slice();
            const description = e.features![0]!.properties;

            // Ensure that if the map is zoomed out such that multiple
            // copies of the feature are visible, the popup appears
            // over the copy being pointed to.
            while (Math.abs(e.lngLat.lng - coordinates[0]) > 180) {
                coordinates[0] += e.lngLat.lng > coordinates[0] ? 360 : -360;
            }

            new maplibregl.Popup()
                .setLngLat(coordinates)
                .setHTML(`
                <p>Region: ${description.flynn_region}<p/>
                <p>Last Update: ${description.lastupdate}</p>
                <p>Mag: ${description.mag}<p>
                `)
                .addTo(map);
        });

        // Change the cursor to a pointer when the mouse is over the places layer.
        map.on('mouseenter', 'seismic-events-layer', () => {
            map.getCanvas().style.cursor = 'pointer';
        });

        // Change it back to a pointer when it leaves.
        map.on('mouseleave', 'seismic-events-layer', () => {
            map.getCanvas().style.cursor = '';
        });

        type WssEvent = {
            action: 'create' | 'update';
            data: PointData;
        };

        const onEvent = new Channel<WssEvent>();
        onEvent.onmessage = (message) => {
            console.log(message)

            const sourceData = map.getSource('seismic-events').serialize();
            const newPoint = {
                type: 'Feature',
                geometry: message.data.geometry,
                properties: message.data,
            };

            realtime.push(message.data)
            sourceData.data.features.push(newPoint);
            map.getSource('seismic-events').setData(sourceData.data);

            map.flyTo({
                center: newPoint.geometry.coordinates,
                speed: 0.5
            })
        }
        await invoke("listen_to_seismic_events", {onEvent});
    })

    let test = '{"geometry":{"coordinates":[-111.3665,45.9583],"type":"Point"},"source_id":"1747322","source_catalog":"EMSC-RTS","lastupdate":"2024-12-23T20:50:05.790919Z","time":"2024-12-23T19:22:40.77Z","lat":45.9583,"lon":-111.3665,"depth":2.4,"evtype":"ke","auth":"MB","mag":2.1,"magtype":"ml","flynn_region":"WESTERN MONTANA","unid":"20241223_0000210","origins":null,"arrivals":null}'
</script>


<div id="map"></div>

<div class="fixed top-0 left-0 m-4">
    <button class="p-2 rounded bg-white shadow-lg" onclick={() => sidebar = !sidebar}>
        <Activity/>
    </button>
</div>

{#if sidebar}
    <aside class="pt-5 h-full fixed bg-white left-0 top-0 z-50 shadow-lg">

        <button onclick={() => sidebar = !sidebar} class="absolute right-3 top-2">
            <X/>
        </button>

        <NewPoint pointData={JSON.parse(test)}></NewPoint>

        {#each realtime as real, index}
            <!-- {JSON.stringify(real) as unknown as PointData} -->
            <NewPoint pointData={real}></NewPoint>
        {/each}
    </aside>
{/if}
