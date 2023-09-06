import {
  ChartXY,
  LegendBox,
  LineSeries,
  LineSeriesOptions,
  lightningChart,
} from "@arction/lcjs";

class WaveformSeries {
  readonly name: string;
  i?: LineSeries;
  q?: LineSeries;

  constructor(name: string) {
    this.name = name;
  }

  setData(chart: ChartXY, legend: LegendBox, data: WaveformData) {
    this.setReal(chart, legend, !data.q);
    this.i!.clear();
    this.i!.addArrayY(data.i);
    if (data.q) {
      this.q!.clear();
      this.q!.addArrayY(data.q);
    }
  }

  private setReal(chart: ChartXY, legend: LegendBox, isReal: boolean) {
    const opts: LineSeriesOptions = {
      dataPattern: {
        pattern: "ProgressiveX",
        regularProgressiveStep: true,
      },
    };
    if (this.i === undefined) {
      const label = isReal ? this.name : `${this.name}_I`;
      this.i = chart.addLineSeries(opts).setName(label);
      legend.add(this.i);
    }
    if (isReal) {
      this.q?.dispose();
      delete this.q;
    } else {
      if (this.q === undefined) {
        this.q = chart.addLineSeries(opts).setName(`${this.name}_Q`);
        legend.add(this.q);
      }
    }
  }

  dispose() {
    this.i?.dispose();
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

export class Viewer {
  chart: ChartXY;
  legend: LegendBox;
  series: Map<string, WaveformSeries>;

  constructor(targetElem: HTMLDivElement) {
    this.chart = lightningChart()
      .ChartXY({
        container: targetElem,
      })
      .setTitle("Waveform Viewer")
      .setAnimationsEnabled(false);
    this.legend = this.chart.addLegendBox().setAutoDispose({
      type: "max-width",
      maxWidth: 0.2,
    });
    this.series = new Map<string, WaveformSeries>();
  }

  static create(targetElem: HTMLDivElement) {
    return new Viewer(targetElem);
  }

  async setSeriesData(
    name: string,
    type: DataType,
    isReal: boolean,
    iqBytesStream: StreamRef,
  ) {
    const iqBytesArray = await iqBytesStream.arrayBuffer();
    const s = this.series.get(name);
    if (s === undefined) {
      return;
    }
    const data = this.decodeWaveform(type, isReal, iqBytesArray);
    s.setData(this.chart, this.legend, data);
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
    this.chart.dispose();
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
    this.series.has(name) || this.series.set(name, new WaveformSeries(name));
  }

  private removeSingleSeries(name: string) {
    this.series.get(name)?.dispose();
    this.series.delete(name);
  }
}
