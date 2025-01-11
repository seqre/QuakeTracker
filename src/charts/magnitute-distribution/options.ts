export const magDistributionOption = (magnitude: any) => {
  return {
    title: {
      text: "Magnitute distribution",
    },
    xAxis: {
      name: "Mag",
    },
    yAxis: {
      name: "Count",
    },
    tooltip: {
      trigger: "axis", // Aligns tooltip with axis values
      axisPointer: {
        type: "cross", // Creates a crosshair for the axis pointer
      },
    },
    series: [
      {
        name: "Count",
        data: magnitude,
        type: "line",
        smooth: true,
      },
    ],
  };
};
