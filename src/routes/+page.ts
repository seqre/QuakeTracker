import type { PageLoad } from "./$types";
import { invoke, isTauri } from "@tauri-apps/api/core";

export const load: PageLoad = async ({ params }) => {
  // @ts-ignore
  console.log(window.__TAURI_INTERNALS__);

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
