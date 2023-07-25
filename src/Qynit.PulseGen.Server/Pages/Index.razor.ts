import {
  ChartXY,
  LegendBox,
  LineSeries,
  LineSeriesOptions,
  lightningChart,
} from "@arction/lcjs";
import { DotNet } from "@microsoft/dotnet-js-interop";

let chart: ChartXY;
let legend: LegendBox;
let dotnetRef: DotNet.DotNetObject;

export function init(targetElem: HTMLDivElement, objRef: DotNet.DotNetObject) {
  chart = lightningChart()
    .ChartXY({
      container: targetElem,
    })
    .setTitle("Waveform Viewer")
    .setAnimationsEnabled(false);
  legend = chart.addLegendBox().setAutoDispose({
    type: "max-width",
    maxWidth: 0.2,
  });
  dotnetRef = objRef;
}

type WaveformSeries = {
  i: LineSeries;
  q?: LineSeries;
};

const series = new Map<string, WaveformSeries>();

enum DataType {
  Float32,
  Float64,
}

type StreamRef = {
  arrayBuffer: () => Promise<ArrayBuffer>;
};

export async function renderWaveform(
  name: string,
  type: DataType,
  isReal: boolean,
  iqBytesStream: StreamRef,
) {
  const iqBytesArray = await iqBytesStream.arrayBuffer();
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

  const { i, q } = series.get(name) ?? initSeries(name, isReal);
  i.clear();
  i.addArrayY(iArray);
  q?.clear();
  q?.addArrayY(qArray!);
}

function initSeries(name: string, isReal: boolean): WaveformSeries {
  const opts: LineSeriesOptions = {
    dataPattern: {
      pattern: "ProgressiveX",
      regularProgressiveStep: true,
    },
  };
  const i = chart.addLineSeries(opts).setName(`${name}_I`);
  const q = isReal ? undefined : chart.addLineSeries(opts).setName(`${name}_Q`);
  series.set(name, { i, q });
  const handler = async (_: LineSeries, state: boolean) => {
    await dotnetRef.invokeMethodAsync("VisibilityChanged", name, state);
  };
  legend.add(i);
  i.onVisibleStateChanged(handler);
  if (q) {
    legend.add(q);
    q.onVisibleStateChanged(handler);
  }
  return { i, q };
}
