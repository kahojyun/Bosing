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
    .setAnimationsEnabled(false);
  legend = chart.addLegendBox().setAutoDispose({
    type: "max-width",
    maxWidth: 0.2,
  });
  dotnetRef = objRef;
}

type WaveformSeries = {
  i: LineSeries;
  q: LineSeries;
};

const series = new Map<string, WaveformSeries>();

export async function renderWaveform(name: string, iqBytesStream) {
  const iqBytesArray: ArrayBuffer = await iqBytesStream.arrayBuffer();
  const float64Array = new Float64Array(iqBytesArray);
  const length = float64Array.length / 2;
  const iArray = float64Array.subarray(0, length);
  const qArray = float64Array.subarray(length, length * 2);
  if (!series.has(name)) {
    const { i, q } = initSeries(name);
    i.addArrayY(iArray);
    q.addArrayY(qArray);
  } else {
    const { i, q } = series.get(name);
    i.clear();
    i.addArrayY(iArray);
    q.clear();
    q.addArrayY(qArray);
  }
}

function initSeries(name: string) {
  const opts: LineSeriesOptions = {
    dataPattern: {
      pattern: "ProgressiveX",
      regularProgressiveStep: true,
    },
  };
  const i = chart.addLineSeries(opts).setName(`${name}_I`);
  const q = chart.addLineSeries(opts).setName(`${name}_Q`);
  series.set(name, { i, q });
  legend.add(i).add(q);
  const handler = async (_, state: boolean) => {
    await dotnetRef.invokeMethodAsync("VisibilityChanged", name, state);
  };
  i.onVisibleStateChanged(handler);
  q.onVisibleStateChanged(handler);
  return { i, q };
}
