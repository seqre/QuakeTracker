import type { Feature } from "maplibre-gl";

export const mapToPieRegions = (features: Feature[]) => {
  const grouped = Object.groupBy(
    features,
    (item) => item.properties.flynn_region
  );

  let otherCount = 0;

  const mapped = Object.keys(grouped)
    .filter((key, index) => {
      otherCount++;
      return grouped[key]?.length != 1;
    })
    .map((key, index) => {
      return { name: key, value: grouped[key]?.length };
    });

  mapped.push({
    name: "Others",
    value: otherCount,
  });

  return mapped;
};
