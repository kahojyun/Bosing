import {
  CursorModifier,
  EAutoRange,
  ECoordinateMode,
  EHorizontalAnchorPoint,
  EVerticalAnchorPoint,
  FastLineRenderableSeries,
  MouseWheelZoomModifier,
  NumberRange,
  NumericAxis,
  SciChartOverview,
  SciChartSurface,
  TWebAssemblyChart,
  TextAnnotation,
  XAxisDragModifier,
  XyDataSeries,
  YAxisDragModifier,
  ZoomExtentsModifier,
  ZoomPanModifier,
  generateGuid,
} from "scichart";

SciChartSurface.UseCommunityLicense();
SciChartSurface.useWasmLocal();

// import { format } from "d3-format";

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
  chart: TWebAssemblyChart;
  overview: SciChartOverview;
  //legend: LegendBox;
  //series: Map<string, WaveformSeries>;

  constructor(chart: TWebAssemblyChart, overview: SciChartOverview) {
    const { wasmContext, sciChartSurface } = chart;

    // Create 100 dataseries, each with 10k points
    for (let seriesCount = 0; seriesCount < 2; seriesCount++) {
      const xyDataSeries = new XyDataSeries(wasmContext);
      const opacity = (1 - seriesCount / 120).toFixed(2);
      // Populate with some data
      for (let i = 0; i < 10000; i++) {
        xyDataSeries.append(
          i,
          Math.sin(i * 0.01) * Math.exp(i * (0.00001 * (seriesCount + 1))),
        );
      }
      // Add and create a line series with this data to the chart
      // Create a line series
      const lineSeries = new FastLineRenderableSeries(wasmContext, {
        dataSeries: xyDataSeries,
      });
      const lineSeries2 = new FastLineRenderableSeries(wasmContext, {
        dataSeries: xyDataSeries,
      });
      sciChartSurface.renderableSeries.add(lineSeries);
      overview.overviewSciChartSurface.renderableSeries.add(lineSeries2);
    }

    sciChartSurface.annotations.add(
      new TextAnnotation({
        x1: 0,
        y1: 0,
        yCoordShift: 20,
        xCoordShift: 20,
        xCoordinateMode: ECoordinateMode.Relative,
        yCoordinateMode: ECoordinateMode.Relative,
        horizontalAnchorPoint: EHorizontalAnchorPoint.Left,
        verticalAnchorPoint: EVerticalAnchorPoint.Top,
        fontSize: 18,
        opacity: 0.55,
        text: "SciChart.js supports an Overview scrollbar. Zoom the main chart or drag the overview to see it update",
      }),
    );

    this.chart = chart;
    this.overview = overview;
    //this.series = new Map<string, WaveformSeries>();
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
    sciChartSurface.xAxes.add(
      new NumericAxis(wasmContext, { visibleRange: new NumberRange(500, 600) }),
    );
    sciChartSurface.yAxes.add(
      new NumericAxis(wasmContext, {
        autoRange: EAutoRange.Always,
        growBy: new NumberRange(0.1, 0.1),
      }),
    );

    const mouseWheelZoomModifier = new MouseWheelZoomModifier();
    sciChartSurface.chartModifiers.add(mouseWheelZoomModifier);
    const xAxisDragModifier = new XAxisDragModifier();
    sciChartSurface.chartModifiers.add(xAxisDragModifier);
    const yAxisDragModifier = new YAxisDragModifier();
    sciChartSurface.chartModifiers.add(yAxisDragModifier);
    //const rubberBandZoomModifier = new RubberBandXyZoomModifier();
    //sciChartSurface.chartModifiers.add(rubberBandZoomModifier);
    const cursorModifier = new CursorModifier();
    sciChartSurface.chartModifiers.add(cursorModifier);
    const zoomExtentsModifier = new ZoomExtentsModifier();
    sciChartSurface.chartModifiers.add(zoomExtentsModifier);
    const zoomPanModifier = new ZoomPanModifier();
    sciChartSurface.chartModifiers.add(zoomPanModifier);

    const overview = await SciChartOverview.create(sciChartSurface, overviewId);
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

  //private addSingleSeries(name: string) {
  //  this.series.has(name) || this.series.set(name, new WaveformSeries(name));
  //}

  //private removeSingleSeries(name: string) {
  //  this.series.get(name)?.dispose();
  //  this.series.delete(name);
  //}
}
