export const magDistributionOption = (magnitude: any) => {
    return {
        title: {
            text: "Magnitude distribution",
            left: "center"
        },
        xAxis: {
            name: "Mag",
            minorTick: {
                show: true,
                splitNumber: 5,
            },
        },
        yAxis: {
            name: "Count",
        },
        tooltip: {
            trigger: "axis",
            axisPointer: {
                type: "none",
            },
        },
        grid: {
            right: '11%',
        },
        series: [
            {
                name: "Count",
                data: magnitude,
                type: "bar",
            },
        ],
    };
};
