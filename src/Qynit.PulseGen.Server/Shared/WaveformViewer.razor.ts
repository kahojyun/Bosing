//import { format } from "d3-format";
import * as echarts from "echarts/core";
import {
  TitleComponent,
  TitleComponentOption,
  ToolboxComponent,
  ToolboxComponentOption,
  TooltipComponent,
  TooltipComponentOption,
  GridComponent,
  GridComponentOption,
  DataZoomComponent,
  DataZoomComponentOption,
} from "echarts/components";
import { LineChart, LineSeriesOption } from "echarts/charts";
import { CanvasRenderer } from "echarts/renderers";

echarts.use([
  TitleComponent,
  ToolboxComponent,
  TooltipComponent,
  GridComponent,
  DataZoomComponent,
  LineChart,
  CanvasRenderer,
]);

type EChartsOption = echarts.ComposeOption<
  | TitleComponentOption
  | ToolboxComponentOption
  | TooltipComponentOption
  | GridComponentOption
  | DataZoomComponentOption
  | LineSeriesOption
>;

//class WaveformSeries {
//  readonly name: string;
//  i?: LineSeries;
//  q?: LineSeries;

//  constructor(name: string) {
//    this.name = name;
//  }

//  setData(chart: ChartXY, legend: LegendBox, data: WaveformData, dt: number) {
//    this.setReal(chart, legend, !data.q);
//    this.i!.clear();
//    this.i!.addArrayY(data.i, dt);
//    if (data.q) {
//      this.q!.clear();
//      this.q!.addArrayY(data.q, dt);
//    }
//  }

//  private setReal(chart: ChartXY, legend: LegendBox, isReal: boolean) {
//    const opts: LineSeriesOptions = {
//      dataPattern: {
//        pattern: "ProgressiveX",
//        regularProgressiveStep: true,
//      },
//    };
//    if (this.i === undefined) {
//      const label = isReal ? this.name : `${this.name}_I`;
//      this.i = chart.addLineSeries(opts).setName(label);
//      legend.add(this.i);
//    }
//    if (isReal) {
//      this.q?.dispose();
//      delete this.q;
//    } else {
//      if (this.q === undefined) {
//        this.q = chart.addLineSeries(opts).setName(`${this.name}_Q`);
//        legend.add(this.q);
//      }
//    }
//  }

//  dispose() {
//    this.i?.dispose();
//    this.q?.dispose();
//  }
//}

interface WaveformData {
  i: Float32Array | Float64Array;
  q?: Float32Array | Float64Array;
}

enum DataType {
  Float32,
  Float64,
}

interface StreamRef {
  arrayBuffer(): Promise<ArrayBuffer>;
}

export class Viewer {
  chart: echarts.ECharts;
  resizeObserver: ResizeObserver;
  //series: Map<string, WaveformSeries>;

  constructor(targetElem: HTMLDivElement) {
    const isDarkMode = window.matchMedia(
      "(prefers-color-scheme: dark)",
    ).matches;

    const chart = echarts.init(targetElem, isDarkMode ? "dark" : null);

    let base = +new Date(1968, 9, 3);
    const oneDay = 24 * 3600 * 1000;
    const date = [];

    const data = [Math.random() * 300];

    for (let i = 1; i < 100000; i++) {
      const now = new Date((base += oneDay));
      date.push(
        [now.getFullYear(), now.getMonth() + 1, now.getDate()].join("/"),
      );
      data.push(Math.round((Math.random() - 0.5) * 20 + data[i - 1]));
    }

    const option: EChartsOption = {
      tooltip: {
        trigger: "axis",
        position: function (pt) {
          return [pt[0], "10%"];
        },
      },
      title: {
        left: "center",
        text: "Large Area Chart",
      },
      toolbox: {
        feature: {
          dataZoom: {
            yAxisIndex: "none",
          },
          restore: {},
          saveAsImage: {},
        },
      },
      xAxis: {
        type: "category",
        boundaryGap: false,
        data: date,
        animation: false,
      },
      yAxis: {
        type: "value",
        boundaryGap: [0, "100%"],
        animation: false,
      },
      dataZoom: [
        {
          type: "inside",
          start: 0,
          end: 10,
        },
        {
          start: 0,
          end: 10,
        },
      ],
      series: [
        {
          name: "Fake Data",
          type: "line",
          symbol: "none",
          sampling: "lttb",
          itemStyle: {
            color: "rgb(255, 70, 131)",
          },
          areaStyle: {
            color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
              {
                offset: 0,
                color: "rgb(255, 158, 68)",
              },
              {
                offset: 1,
                color: "rgb(255, 70, 131)",
              },
            ]),
          },
          data: data,
        },
      ],
    };

    chart.setOption(option);

    const resizeObserver = new ResizeObserver(() => {
      chart.resize();
    });
    resizeObserver.observe(targetElem);

    this.chart = chart;
    this.resizeObserver = resizeObserver;
    //this.series = new Map<string, WaveformSeries>();
  }

  static create(targetElem: HTMLDivElement) {
    return new Viewer(targetElem);
  }

  async setSeriesData(
    name: string,
    type: DataType,
    isReal: boolean,
    dt: number,
    iqBytesStream: StreamRef,
  ) {
    const iqBytesArray = await iqBytesStream.arrayBuffer();
    //const s = this.series.get(name);
    //if (s === undefined) {
    //  return;
    //}
    //const data = this.decodeWaveform(type, isReal, iqBytesArray);
    //s.setData(this.chart, this.legend, data, dt);
  }

  setAllSeries(names: string[]) {
    //const oldNames = Array.from(this.series.keys());
    //for (const name of oldNames) {
    //  if (!names.includes(name)) {
    //    this.removeSingleSeries(name);
    //  }
    //}
    //for (const name of names) {
    //  this.addSingleSeries(name);
    //}
  }

  dispose() {
    this.chart.dispose();
    this.resizeObserver.disconnect();
  }

  private decodeWaveform(
    type: DataType,
    isReal: boolean,
    iqBytesArray: ArrayBuffer,
  ): WaveformData {
    const iqArray = (() => {
      if (type === DataType.Float32) {
        return new Float32Array(iqBytesArray);
      } else if (type === DataType.Float64) {
        return new Float64Array(iqBytesArray);
      } else {
        throw new Error("Unsupported data type");
      }
    })();

    const length = isReal ? iqArray.length : iqArray.length / 2;
    const iArray = iqArray.subarray(0, length);
    const qArray = isReal ? undefined : iqArray.subarray(length, length * 2);
    return { i: iArray, q: qArray };
  }

  private addSingleSeries(name: string) {
    //this.series.has(name) || this.series.set(name, new WaveformSeries(name));
  }

  private removeSingleSeries(name: string) {
    //this.series.get(name)?.dispose();
    //this.series.delete(name);
  }
}
