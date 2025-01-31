import type { Feature } from "maplibre-gl";

export const mapToPieRegions = (features: Feature[]) => {
  const grouped = Object.groupBy(
    features,
    (item) => item.properties.flynn_region
  );

  // Convert to array and sort by count in descending order
  const sortedRegions = Object.entries(grouped)
    .map(([key, value]) => ({ name: key, value: value!.length }))
    .sort((a, b) => b.value - a.value);

  // Get the top 10 regions
  const topRegions = sortedRegions.slice(0, 10);

  // Sum up the rest into "Others"
  const othersCount = sortedRegions.slice(10).reduce((sum, region) => sum + region.value, 0);

  if (othersCount > 0) {
    topRegions.push({ name: "Others", value: othersCount });
  }

  return topRegions;
};
