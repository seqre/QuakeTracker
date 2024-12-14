import type {PageLoad} from './$types';
import {invoke} from '@tauri-apps/api/core';

export const load: PageLoad = async ({params}) => {
    return {
        features: await invoke('get_seismic_events', {
            queryParams: {
                'limit': 100,
            }
        }).then((res) => {
            // @ts-ignore
            return res.features
        })
    }
};
