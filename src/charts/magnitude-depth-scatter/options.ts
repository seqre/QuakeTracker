export const magDepthScatterOption = (data: any) => {
    return {
        title: {
            text: "Magnitude & depth distribution",
            left: "center"
        },
        grid: {
            left: "15%",
        },
        xAxis: {
            name: "Magnitude",
            nameLocation: "center",
            nameGap: 25,
        },
        yAxis: {
            name: "Depth",
            nameLocation: "center",
            inverse: true,
            min: 0,
            nameGap: 35,
        },
        series: [
            {
                data: data,
                type: "scatter",
            },
        ],
    };
};
