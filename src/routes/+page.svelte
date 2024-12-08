<script lang="ts">

    import maplibregl from 'maplibre-gl';
    import {PMTiles, Protocol} from 'pmtiles';
    import {onMount} from 'svelte';

    import {defaultTheme} from '../theme'

    let {data}: { data: PageData } = $props();


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

        // the location of our pmtiles file
        const PMTILES_URL = "./my_area.pmtiles";

        // create a new PmTiles instance
        const pmtilesInstance = new PMTiles(PMTILES_URL);

        // this is so we share one instance across the JS code and the map renderer
        protocol.add(pmtilesInstance);

        // we first fetch the header so we can get the center lon, lat of the map.
        const mapMetaData = await pmtilesInstance.getHeader();


        const all = await fetch('https://www.seismicportal.eu/fdsnws/event/1/query?limit=100&format=json')
            .then(res => res.json())
            .then(res => res.features)

        console.log(all)


        const map = new maplibregl.Map({
            container: 'map',
            minZoom: 1.3,
            maxZoom: 5,
            center: [0, 0],
            zoom: 0.2,
            style: {
                version: 8,
                sprite: "https://demotiles.maplibre.org/styles/osm-bright-gl-style/sprite",
                // ading protomaps as the data source for our map
                sources: {
                    'example-points': {
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
                        id: "example-points-layer",
                        type: "circle",
                        source: "example-points",
                        paint: {
                            "circle-radius": 5,
                            "circle-color": "#ff5722",
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


        map.on('click', 'example-points-layer', (e) => {
            const coordinates = e.features[0]!.geometry.coordinates.slice();
            const description = e.features[0]!.properties.description;

            // Ensure that if the map is zoomed out such that multiple
            // copies of the feature are visible, the popup appears
            // over the copy being pointed to.
            while (Math.abs(e.lngLat.lng - coordinates[0]) > 180) {
                coordinates[0] += e.lngLat.lng > coordinates[0] ? 360 : -360;
            }

            new maplibregl.Popup()
                .setLngLat(coordinates)
                .setHTML(description)
                .addTo(map);
        });

        // Change the cursor to a pointer when the mouse is over the places layer.
        map.on('mouseenter', 'example-points-layer', () => {
            map.getCanvas().style.cursor = 'pointer';
        });

        // Change it back to a pointer when it leaves.
        map.on('mouseleave', 'example-points-layer', () => {
            map.getCanvas().style.cursor = '';
        });


        // console.debug(map.styles.features)

        // map.on('')  
    })


</script>


<div id="map"></div>


