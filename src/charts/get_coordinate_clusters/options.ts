export const getCoordinateClusters = (data: any) => {
  return {
    title: {
      text: "Earthquake Hotspot Clusters",
      subtext: "Using Apache ECharts",
      left: "center",
    },
    tooltip: {
      trigger: "item",
      formatter: function (params) {
        if (params.componentSubType === "scatter") {
          return (
            params.name +
            "<br/>Lng: " +
            params.value[0].toFixed(4) +
            "<br/>Lat: " +
            params.value[1].toFixed(4)
          );
        }
        return params.name;
      },
    },

    geo: {
      map: "world",
      roam: true,
      itemStyle: {
        areaColor: "#f0f2f5", // background color for map
        borderColor: "#aaa", // border color (countries)
      },
      emphasis: {
        // Style for map areas on hover
        disabled: false,
        itemStyle: {
          areaColor: "#d4d8dd",
          shadowBlur: 10,
          shadowColor: "rgba(0, 0, 0, 0.3)",
        },
        label: {
          show: false, // no country labels
        },
      },
      // Initial map center 
      center: [10, 20],
      zoom: 1.2, // Initial zoom
      scaleLimit: {
        min: 1, // Min zoom
        max: 10, // Max zoom
      },
      boundingCoords: [
        // limit area
        [-180, 90], // Top-left
        [180, -90], // Bottom-right
      ],
    },
    series: [
      {
        type: "scatter", // type of chart
        coordinateSystem: "geo",
        // show points on the map
        data: data
          .filter((item) => Math.abs(item[1]) < 70)
          .map((item) => {
            return {
              name: item[2],
              // longitude, latitude, and magnitude
              value: [item[0], item[1], item[2]],
            };
          }),
        // size of points based on magnitude
        symbolSize: function (val) {
          return Math.min(2 + val[2] / 3, 30);
        },
        itemStyle: {
          // point style
          color: "#dd6b66",
        },
        emphasis: {
          // points on hover
          focus: "series", 
          label: {
            show: true,
            formatter: "{b}\nLng: {@[1]:.2f}, Lat: {@[2]:.2f}", // data format on hover
            backgroundColor: "rgba(0,0,0,0.7)",
            borderColor: "#333",
            borderWidth: 1,
            padding: [5, 7],
            color: "#fff",
            fontSize: 12,
            borderRadius: 4,
          },
          itemStyle: {
            shadowBlur: 10,
            shadowColor: "rgba(0, 0, 0, 0.3)",
            borderColor: "rgba(0,0,0,0.4)",
            borderWidth: 1,
          },
        },
      },
    ],
  };
};
