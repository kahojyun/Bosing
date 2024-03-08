import {
  AUTO_COLOR,
  EExecuteOn,
  FastLineRenderableSeries,
  MouseWheelZoomModifier,
  NumberRange,
  NumericAxis,
  NumericLabelProvider,
  RolloverModifier,
  RubberBandXyZoomModifier,
  SciChartOverview,
  SciChartSurface,
  TWebAssemblyChart,
  XAxisDragModifier,
  XyDataSeries,
  YAxisDragModifier,
  ZoomExtentsModifier,
  ZoomPanModifier,
  generateGuid,
} from "scichart";

SciChartSurface.UseCommunityLicense();

import { format } from "d3-format";

class ChartSeriesWithData {
  data: XyDataSeries;
  chartSeries: FastLineRenderableSeries;
  overviewSeries: FastLineRenderableSeries;

  constructor(
    chart: TWebAssemblyChart,
    overview: SciChartOverview,
    name: string,
  ) {
    const { sciChartSurface, wasmContext } = chart;
    this.data = new XyDataSeries(wasmContext, {
      dataSeriesName: name,
      dataIsSortedInX: true,
      dataEvenlySpacedInX: true,
      containsNaN: false,
    });
    this.chartSeries = new FastLineRenderableSeries(wasmContext, {
      stroke: AUTO_COLOR,
      dataSeries: this.data,
    });
    this.overviewSeries = new FastLineRenderableSeries(wasmContext, {
      stroke: AUTO_COLOR,
      dataSeries: this.data,
    });
    sciChartSurface.renderableSeries.add(this.chartSeries);
    overview.overviewSciChartSurface.renderableSeries.add(this.overviewSeries);
  }

  setData(data: Float32Array | Float64Array, dt: number) {
    this.data.clear();
    const xValues = Array(data.length)
      .fill(0)
      .map((_, i) => i * dt);
    const yValues = Array.from(data);
    this.data.appendRange(xValues, yValues);
  }

  dispose() {
    this.data.delete();
    this.chartSeries.parentSurface.renderableSeries.remove(this.chartSeries);
    this.chartSeries.delete();
    this.overviewSeries.parentSurface.renderableSeries.remove(
      this.overviewSeries,
    );
    this.overviewSeries.delete();
  }
}

class WaveformSeries {
  readonly name: string;
  chart: TWebAssemblyChart;
  overview: SciChartOverview;
  i: ChartSeriesWithData;
  q?: ChartSeriesWithData;

  constructor(
    name: string,
    chart: TWebAssemblyChart,
    overview: SciChartOverview,
  ) {
    this.name = name;
    this.chart = chart;
    this.overview = overview;
    this.i = new ChartSeriesWithData(chart, overview, `${name}_I`);
  }

  setData(data: WaveformData, dt: number) {
    this.i.setData(data.i, dt);
    if (!data.q) {
      this.q?.dispose();
      delete this.q;
    } else {
      this.q ??= new ChartSeriesWithData(
        this.chart,
        this.overview,
        `${this.name}_Q`,
      );
      this.q.setData(data.q, dt);
    }
  }

  dispose() {
    this.i.dispose();
    this.q?.dispose();
  }
}

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

class CustomNumericLabelProvider extends NumericLabelProvider {
  formatLabelProperty = format("s");
  formatCursorLabelProperty = format("s");
}

export class Viewer {
  chart: TWebAssemblyChart;
  overview: SciChartOverview;
  series: Map<string, WaveformSeries>;

  constructor(chart: TWebAssemblyChart, overview: SciChartOverview) {
    this.chart = chart;
    this.overview = overview;
    this.series = new Map<string, WaveformSeries>();
  }

  static async create(
    chartElement: HTMLDivElement,
    overviewElement: HTMLDivElement,
  ) {
    const chartId = `elem-${generateGuid()}`;
    const overviewId = `elem-${generateGuid()}`;
    chartElement.id = chartId;
    overviewElement.id = overviewId;

    const chart = await SciChartSurface.create(chartId);
    const { wasmContext, sciChartSurface } = chart;
    const xAxis = new NumericAxis(wasmContext, {
      labelProvider: new CustomNumericLabelProvider(),
    });
    const yAxis = new NumericAxis(wasmContext, {
      labelProvider: new CustomNumericLabelProvider(),
    });
    sciChartSurface.xAxes.add(xAxis);
    sciChartSurface.yAxes.add(yAxis);

    const mouseWheelZoomModifier = new MouseWheelZoomModifier();
    sciChartSurface.chartModifiers.add(mouseWheelZoomModifier);
    const xAxisDragModifier = new XAxisDragModifier();
    sciChartSurface.chartModifiers.add(xAxisDragModifier);
    const yAxisDragModifier = new YAxisDragModifier();
    sciChartSurface.chartModifiers.add(yAxisDragModifier);
    const rubberBandZoomModifier = new RubberBandXyZoomModifier();
    sciChartSurface.chartModifiers.add(rubberBandZoomModifier);
    const rolloverModifier = new RolloverModifier({
      tooltipDataTemplate: (seriesInfo) => [
        `${seriesInfo.seriesName}: ${seriesInfo.formattedYValue}`,
      ],
    });
    sciChartSurface.chartModifiers.add(rolloverModifier);
    const zoomExtentsModifier = new ZoomExtentsModifier();
    sciChartSurface.chartModifiers.add(zoomExtentsModifier);
    const zoomPanModifier = new ZoomPanModifier({
      executeOn: EExecuteOn.MouseMiddleButton,
    });
    sciChartSurface.chartModifiers.add(zoomPanModifier);

    const overview = await SciChartOverview.create(
      sciChartSurface,
      overviewId,
      {
        overviewYAxisOptions: {
          growBy: new NumberRange(0.1, 0.1),
        },
      },
    );
    return new Viewer(chart, overview);
  }

  async setSeriesData(
    name: string,
    type: DataType,
    isReal: boolean,
    dt: number,
    iqBytesStream: StreamRef,
  ) {
    const iqBytesArray = await iqBytesStream.arrayBuffer();
    const s = this.series.get(name);
    if (!s) {
      return;
    }
    const data = this.decodeWaveform(type, isReal, iqBytesArray);
    s.setData(data, dt);
  }

  setAllSeries(names: string[]) {
    const oldNames = Array.from(this.series.keys());
    for (const name of oldNames) {
      if (!names.includes(name)) {
        this.removeSingleSeries(name);
      }
    }
    for (const name of names) {
      this.addSingleSeries(name);
    }
  }

  dispose() {
    this.chart.sciChartSurface.delete();
    this.overview.delete();
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
    this.series.has(name) ||
      this.series.set(
        name,
        new WaveformSeries(name, this.chart, this.overview),
      );
  }

  private removeSingleSeries(name: string) {
    this.series.get(name)?.dispose();
    this.series.delete(name);
  }
}
