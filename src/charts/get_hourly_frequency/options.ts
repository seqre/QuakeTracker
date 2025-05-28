

export const getHourlyFrequencyOptions = (data: any) => {
    return {
    title: {
      text: 'Hourly Earthquake Frequency',
      left: 'center'
    },
    angleAxis: {
      type: 'category',
      data: data.map(item => item[0]).map((hour:any) => `${hour}:00`),
      startAngle: 90,
      axisLabel: {
        interval: 1
      }
    },
    radiusAxis: {
      type: 'value'
    },
    polar: {},
    tooltip: {
      trigger: 'item',
      formatter: (params:any) => `${params.name}<br/>Count: ${params.value}`
    },
    series: [{
      type: 'bar',
      data: data.map((item) => item[1]),
      coordinateSystem: 'polar',
      name: 'Earthquakes',
      itemStyle: {
        color: '#EE6666'
      }
    }]
  }
}