import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params }) => {
	return {
        features: await fetch('https://www.seismicportal.eu/fdsnws/event/1/query?limit=100&format=json')
        .then(res => res.json())
        .then(res => res.features)
    }
};
