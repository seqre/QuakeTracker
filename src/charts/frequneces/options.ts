export const magFreqDataFunc = (data: any[]) => {
    const transformedData = data.map(([magnitude, count, cumulative]) => ({
        magnitude,
        logCumulative: Math.log10(cumulative),
    }));

    return {
        title: {
            text: "Gutenberg-Richter Magnitude Distribution",
            left: "center"
        },
        tooltip: {
            trigger: "axis",
            formatter: (params: any) => {
                const p = params[0];
                return `Mag: ${p.data[0]}<br>log10(Cumulative): ${p.data[1].toFixed(2)}`;
            }
        },
        xAxis: {
            name: "Magnitude",
            type: "value",
            minorTick: {
                show: true,
                splitNumber: 5,
            },
        },
        yAxis: {
            name: "log₁₀(Cumulative)",
            type: "value",
        },
        series: [
            {
                name: "log₁₀(Cumulative)",
                type: "line",
                smooth: true,
                showSymbol: true,
                symbolSize: 8,
                data: transformedData.map(d => [d.magnitude, d.logCumulative]),
                lineStyle: {
                    width: 2
                },
                itemStyle: {
                    color: '#5470C6'
                }
            }
        ]
    };
};
