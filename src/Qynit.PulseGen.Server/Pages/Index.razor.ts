import { ChartXY, LegendBox, LineSeries, lightningChart } from "@arction/lcjs"
import { DotNet } from "@microsoft/dotnet-js-interop"

let chart: ChartXY;
let legend: LegendBox;
let dotnetRef: DotNet.DotNetObject;

export function init(targetElem: HTMLDivElement, objRef: DotNet.DotNetObject) {
  chart = lightningChart()
    .ChartXY({
      container: targetElem,
    })
    .setAnimationsEnabled(false);
  legend = chart.addLegendBox();
  dotnetRef = objRef;
}

type WaveformSeries = {
  i: LineSeries,
  q: LineSeries,
};

let series = new Map<string, WaveformSeries>();

export async function addWaveform(name: string, iqBytesStream) {
  const iqBytesArray: ArrayBuffer = await iqBytesStream.arrayBuffer();
  const float64Array = new Float64Array(iqBytesArray);
  const length = float64Array.length / 2;
  const iArray = float64Array.subarray(0, length);
  const qArray = float64Array.subarray(length, length * 2);
  if (series.has(name)) {
    const { i, q } = series.get(name);
    if (i.getVisible()) {
      i.clear();
      i.addArrayY(iArray);
    }
    if (q.getVisible()) {
      q.clear();
      q.addArrayY(qArray);
    }
  }
  else {
    const i = chart.addLineSeries({
      dataPattern: {
        pattern: 'ProgressiveX',
        regularProgressiveStep: true,
      }
    }).setName(`${name}_I`).addArrayY(iArray);
    legend.add(i);
    i.onVisibleStateChanged(async (_, state) => {
      if (state) {
        await dotnetRef.invokeMethodAsync('UpdateWaveform', name);
      }
    });
    const q = chart.addLineSeries({
      dataPattern: {
        pattern: 'ProgressiveX',
        regularProgressiveStep: true,
      }
    }).setName(`${name}_Q`).addArrayY(qArray);
    series.set(name, { i, q });
    legend.add(q);
    q.onVisibleStateChanged(async (_, state) => {
      if (state) {
        await dotnetRef.invokeMethodAsync('UpdateWaveform', name);
      }
    });
  }
}