import { NumericAxis, SciChartSurface, TWebAssemblyChart } from "scichart";

SciChartSurface.UseCommunityLicense();
SciChartSurface.useWasmFromCDN();

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
  //legend: LegendBox;
  //series: Map<string, WaveformSeries>;

  constructor(chart: TWebAssemblyChart) {
    const { wasmContext, sciChartSurface } = chart;
    const xAxis = new NumericAxis(wasmContext);
    const yAxis = new NumericAxis(wasmContext);
    sciChartSurface.xAxes.add(xAxis);
    sciChartSurface.yAxes.add(yAxis);
    this.chart = chart;
    //this.series = new Map<string, WaveformSeries>();
  }

  static async create(targetElem: HTMLDivElement) {
    const chart = await SciChartSurface.create(targetElem);
    return new Viewer(chart);
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
