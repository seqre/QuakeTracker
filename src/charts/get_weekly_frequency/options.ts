
export const getWeeklyFrequencyOptions = (data: any) => {
  const max = Math.max(...data.map((item: any) => item[1])) * 1.2; // Add 20% padding

  return {
    title: {
      text: "Weekly Frequency Chart",
      left: "center",
      padding: 20,
    },
    radar: {
      // Shape: 'circle', // Optional
      indicator: data.map((item: any) => ({
        name: item[0],
        max: max,
      })),
    },
    series: [  // Moved outside radar property
      {
        name: 'Frequency',
        type: 'radar',
        data: [
          {
            value: data.map((item: any) => item[1]),
            name: 'Frequency',
          }
        ],
        areaStyle: {  // Optional styling
          opacity: 0.2
        }
      }
    ]
  };
};