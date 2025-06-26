import type { PageData, PageLoad } from "./$types";
import {invoke, isTauri} from "@tauri-apps/api/core";



export const load: PageLoad = async ({params}) => {
    // @ts-ignore
    console.log(window.__TAURI_INTERNALS__);
    
    // Get query parameters from localStorage
    let queryParams: any = { limit: 100 }; // default
    
    try {
        const savedParams = window.localStorage.getItem('queryParams');
        if (savedParams) {
            queryParams = JSON.parse(savedParams);
        }
        console.error(JSON.stringify(queryParams))
    } catch (e) {
        console.error('Error parsing query params from localStorage:', e);
        // Fallback to legacy limit if new format fails
        const limit = parseInt(window.localStorage.getItem('limit') || "100");
        queryParams = { limit };
    }

    if (isTauri()) {
        return {
            features: await invoke("get_seismic_events", {
                queryParams,
                clear: true
            }).then((res) => {
                // @ts-ignore
                return res.features;
            }),
        };
    } else {
        // For web version, we'll build a query string from the parameters
        const searchParams = new URLSearchParams();
        
        // Map our parameter names to the web API parameter names if they differ
        const paramMapping: { [key: string]: string } = {
            'limit': 'limit',
            'start_time': 'starttime',
            'end_time': 'endtime',
            'min_latitude': 'minlatitude',
            'max_latitude': 'maxlatitude',
            'min_longitude': 'minlongitude',
            'max_longitude': 'maxlongitude',
            'min_depth': 'mindepth',
            'max_depth': 'maxdepth',
            'min_magnitude': 'minmagnitude',
            'max_magnitude': 'maxmagnitude',
            'magnitude_type': 'magnitudetype',
            'order_by': 'orderby'
        };
        
        // Add enabled parameters to the search params
        Object.keys(queryParams).forEach(key => {
            const value = queryParams[key];
            if (value !== undefined && value !== null && value !== '') {
                const apiKey = paramMapping[key] || key;
                searchParams.append(apiKey, String(value));
            }
        });
        
        // Always add format=json for the web API
        searchParams.append('format', 'json');
        
        const queryString = searchParams.toString();
        const url = `https://www.seismicportal.eu/fdsnws/event/1/query?${queryString}`;
        
        console.log('Fetching from URL:', url);
        
        return {
            features: await fetch(url)
                .then((res) => res.json())
                .then((res) => res.features),
        };
    }
};
