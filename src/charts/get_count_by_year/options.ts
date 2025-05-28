

export const getCountByYearOptions = (data: any) => {
    return {
    title: {
      text: 'Daily Earthquake Counts - 2025',
      left: 'center'
    },
    tooltip: {
      trigger: 'axis',
      formatter: (params:any) => {
        const { name, value } = params[0];
        return `${name}<br/>Count: ${value}`;
      }
    },
    xAxis: {
      type: 'category',
      data: data.map((item) => item[0]),
      boundaryGap: false,
      axisLabel: {
        rotate: 45,
        formatter: (value:any) => value.substring(5) // Show MM-DD for clarity
      }
    },
    yAxis: {
      type: 'value',
      name: 'Earthquakes',
      minInterval: 1,
    },
    series: [{
      name: 'Daily Count',
      type: 'line',
      data: data.map((item) => item[1]),
      smooth: true,
      lineStyle: {
        color: '#73C0DE'
      },
      itemStyle: {
        color: '#5470C6'
      },
      areaStyle: {
        color: 'rgba(115, 192, 222, 0.3)'
      }
    }]
  };
}