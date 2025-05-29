export const getMonthlyFrequencyOptions = (data: any) => {
  const monthNames = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 
                     'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
  
  return {
    title: {
      text: 'Monthly Earthquake Trends',
      left: 'center'
    },
    tooltip: {
      trigger: 'axis',
      formatter: (params: any) => {
        const count = params[0].data[1];
        return `${count} earthquakes`;
      }
    },
    xAxis: {
      type: 'category',
      data: monthNames.filter((_, index) => data.some((item: any) => item[0] === index + 1)),
      axisLabel: {
        interval: 0 // Show all labels
      }
    },
    yAxis: {
      type: 'value',
      name: 'Earthquake Count',
      axisLine: {
        show: true
      }
    },
    series: [{
      name: 'Earthquakes',
      type: 'line',
      data: data.map((item: any) => [monthNames[item[0] - 1], item[1]]),
      areaStyle: {
        color: 'rgba(255, 100, 100, 0.4)'
      },
      lineStyle: {
        color: 'rgba(255, 50, 50, 0.8)'
      },
      itemStyle: {
        color: '#ff3232' // point color
      },
      smooth: true,
      symbol: 'circle',
      symbolSize: 6
    }],
    grid: {
      containLabel: true,
      left: '3%',
      right: '4%',
      bottom: '3%'
    }
  };
};