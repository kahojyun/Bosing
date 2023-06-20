using System.Diagnostics;

using Qynit.Pulsewave;

using ScottPlot;

var sw = Stopwatch.StartNew();

var ch1 = new Channel("ch1");
var ch2 = new Channel("ch2");

var inputNode1 = new InputNode();
var inputNode2 = new InputNode();

var n = 100000;
var sampleRate = 2e9;
var outputNode1 = new OutputNode(n, sampleRate, 0);
var outputNode2 = new OutputNode(n, sampleRate, 0);

ConnectNode(inputNode1, outputNode1);
ConnectNode(inputNode2, outputNode2);

var generator = new WaveformGenerator();
generator.AddChannel(ch1, inputNode1, 100e6);
generator.AddChannel(ch2, inputNode2, 250e6);

var instructions = new List<Instruction>();
var shape = new HannPulseShape();
instructions.Add(new Play(shape, 0, 30e-9, 100e-9, 0.5, 0, 0, ch1));
instructions.Add(new Play(shape, 0, 30e-9, 100e-9, 0.6, 0, 0, ch2));
instructions.Add(new ShiftPhase(0.25, ch1));
instructions.Add(new ShiftPhase(-0.25, ch2));
instructions.Add(new Play(shape, 200e-9, 30e-9, 100e-9, 0.5, 0, 0, ch1));
instructions.Add(new Play(shape, 200e-9, 30e-9, 100e-9, 0.6, 0, 0, ch2));
instructions.Add(new ShiftFrequency(-100e6, 400e-9, ch1));
instructions.Add(new ShiftFrequency(-250e6, 400e-9, ch2));
instructions.Add(new Play(shape, 400e-9, 30e-9, 100e-9, 0.5, 0, 0, ch1));
instructions.Add(new Play(shape, 400e-9, 30e-9, 100e-9, 0.6, 0, 0, ch2));
instructions.Add(new SetFrequency(0, 600e-9, ch1));
instructions.Add(new SetFrequency(0, 600e-9, ch2));

var tStart = 600e-9;
var count = 0;
while (tStart < 49e-6)
{
    instructions.Add(new Play(shape, tStart, 30e-9, 0, 0.5, 0, 0, ch1));
    instructions.Add(new Play(shape, tStart, 30e-9, 0, 0.6, 0, 0, ch2));
    instructions.Add(new ShiftPhase(0.25, ch1));
    instructions.Add(new ShiftPhase(-0.25, ch2));
    tStart += 0.1e-9;
    count++;
}

generator.Run(instructions);

sw.Stop();
Console.WriteLine($"Elapsed time: {sw.Elapsed.TotalMilliseconds} ms");
Console.WriteLine($"Count = {count}");

using var waveform1 = outputNode1.TakeWaveform();
using var waveform2 = outputNode2.TakeWaveform();

var plot = new Plot();
plot.AddSignal(waveform1.DataI.ToArray(), sampleRate, label: $"wave 1 real");
plot.AddSignal(waveform1.DataQ.ToArray(), sampleRate, label: $"wave 1 imag");
plot.AddSignal(waveform2.DataI.ToArray(), sampleRate, label: $"wave 2 real");
plot.AddSignal(waveform2.DataQ.ToArray(), sampleRate, label: $"wave 2 imag");
plot.Legend();
using var viewer = new FormsPlotViewer(plot);
viewer.ShowDialog();

static void ConnectNode(IFilterNode source, IFilterNode target)
{
    source.Outputs.Add(target);
    target.Inputs.Add(source);
}
