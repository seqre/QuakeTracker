import type {PageLoad} from "./$types";
import {invoke, isTauri, Channel} from "@tauri-apps/api/core";
import {warn, debug, trace, info, error} from '@tauri-apps/plugin-log';

export const load: PageLoad = async ({params}) => {
    // @ts-ignore
    console.log(window.__TAURI_INTERNALS__);

    //TODO: replace string with predefined struct
    const onEvent = new Channel<string>();
    onEvent.onmessage = (message) => {
        console.info(message)
    }
    await invoke("listen_to_seismic_events", {onEvent});

    if (isTauri())
        return {
            features: await invoke("get_seismic_events", {
                queryParams: {
                    limit: 100,
                },
            }).then((res) => {
                // @ts-ignore
                return res.features;
            }),
        };
    else
        return {
            features: await fetch(
                "https://www.seismicportal.eu/fdsnws/event/1/query?limit=100&format=json"
            )
                .then((res) => res.json())
                .then((res) => res.features),
        };
};
