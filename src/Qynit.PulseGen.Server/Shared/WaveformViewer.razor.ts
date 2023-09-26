//import { format } from "d3-format";
import * as echarts from "echarts/core";
import { GridComponent, GridComponentOption, DataZoomComponent, DataZoomComponentOption } from "echarts/components";
import { LineChart, LineSeriesOption } from "echarts/charts";
import { CanvasRenderer } from "echarts/renderers";

echarts.use([GridComponent, DataZoomComponent, LineChart, CanvasRenderer]);

type EChartsOption = echarts.ComposeOption<
  GridComponentOption | LineSeriesOption | DataZoomComponentOption
>;

interface WaveformData {
  i: Float32Array | Float64Array;
  q?: Float32Array | Float64Array;
}

interface SeriesId {
  i: string;
  q?: string;
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
  seriesIds: Map<string, SeriesId>;
  resizeObserver: ResizeObserver;

  constructor(targetElem: HTMLDivElement) {
    const isDarkMode = window.matchMedia(
      "(prefers-color-scheme: dark)",
    ).matches;

    const chart = echarts.init(targetElem, isDarkMode ? "dark" : null);

    const option: EChartsOption = {
      xAxis: {
        type: "value",
      },
      yAxis: {
        type: "value",
      },
      animation: false,
      dataZoom: [
        {
          type: "inside",
        },
        {
          type: "slider",
        }
      ],
      series: [],
    };

    chart.setOption(option);

    const resizeObserver = new ResizeObserver(() => {
      chart.resize();
    });
    resizeObserver.observe(targetElem);

    this.chart = chart;
    this.seriesIds = new Map();
    this.resizeObserver = resizeObserver;
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
    const existingSeries = this.seriesIds.get(name);
    if (!existingSeries) {
      return;
    }

    const data = this.decodeWaveform(type, isReal, iqBytesArray);
    const seriesData = new Array(data.i.length);
    for (let i = 0; i < data.i.length; i++) {
      seriesData[i] = [i * dt, data.i[i]];
    }
    const newOptions: EChartsOption = {
      series: [
        {
          id: name,
          type: "line",
          data: seriesData,
          sampling: "lttb",
          showSymbol: false,
        },
      ],
    };
    this.chart.setOption(newOptions);
  }

  setAllSeries(names: string[]) {
    const newSeriesIds = new Map<string, SeriesId>();
    for (const name of names) {
      const oldSeries = this.seriesIds.get(name);
      if (oldSeries) {
        newSeriesIds.set(name, oldSeries);
      } else {
        newSeriesIds.set(name, { i: name });
      }
    }
    this.seriesIds = newSeriesIds;
    const series = Array.from(newSeriesIds.values()).flatMap((seriesId) => {
      if (seriesId.q) {
        return [{ id: seriesId.i }, { id: seriesId.q }];
      } else {
        return [{ id: seriesId.i }];
      }
    });
    const newOptions: EChartsOption = { series };
    this.chart.setOption(newOptions, {
      replaceMerge: "series",
    });
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
}
