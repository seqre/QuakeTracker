import { Channel, invoke } from "@tauri-apps/api/core";
import maplibregl from "maplibre-gl";
import {PMTiles, Protocol} from 'pmtiles';

export const MapFunc = async ({data, PMTILES_URL, defaultTheme, realtime}) => {


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

        const pmtilesInstance = new PMTiles(PMTILES_URL);

        // this is so we share one instance across the JS code and the map renderer
        protocol.add(pmtilesInstance);


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

        // @ts-ignore
        const sourceData = map.getSource('seismic-events').serialize();
        const newPoint = {
            type: 'Feature',
            geometry: message.data.geometry,
            properties: message.data,
        };

        realtime.push(message.data)
        sourceData.data.features.push(newPoint);
        // @ts-ignore
        map.getSource('seismic-events').setData(sourceData.data);

        map.flyTo({
            // @ts-ignore
            center: newPoint.geometry.coordinates,
            speed: 0.5
        })
    }

    await invoke("listen_to_seismic_events", { onEvent });

    return map
}
