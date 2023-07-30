using System.Diagnostics;
using System.Numerics;

using Qynit.PulseGen;

using ScottPlot;

for (var i = 0; i < 5; i++)
{
    RunDouble();
}

//for (var i = 0; i < 5; i++)
//{
//    RunSingle();
//}
//RunDouble();

static void RunDouble()
{
    Console.WriteLine("RunDouble:");
    Run<double>();
    Console.WriteLine("------------------------");
}

static void RunSingle()
{
    Console.WriteLine("RunSingle:");
    Run<float>();
    Console.WriteLine("------------------------");
}

static void Run<T>() where T : unmanaged, IFloatingPointIeee754<T>
{
    var sw = Stopwatch.StartNew();

    var phaseTrackingTransform = new PhaseTrackingTransform();
    var ch1 = phaseTrackingTransform.AddChannel(100e6);
    var ch2 = phaseTrackingTransform.AddChannel(250e6);
    var shape = new HannPulseShape();
    var stack = new StackSchedule();
    stack.Add(new PlayElement(ch1, new(shape, 30e-9, 100e-9), 0, 0, 0.5, 2e-9));
    stack.Add(new PlayElement(ch2, new(shape, 30e-9, 50e-9), 0, 0, 0.6, 2e-9));
    stack.Add(new ShiftPhaseElement(ch1, 0.25));
    stack.Add(new ShiftPhaseElement(ch2, -0.25));
    stack.Add(new BarrierElement(ch1, ch2) { Margin = new(15e-9) });

    var stack2 = new StackSchedule();
    stack2.Add(new PlayElement(ch1, new(shape, 30e-9, 0), 0, 0, 0.5, 2e-9) { Margin = new(15e-9) });
    stack2.Add(new PlayElement(ch1, new(shape, 30e-9, 0), 0, 0, 0.5, 2e-9) { Margin = new(15e-9) });
    stack2.Add(new PlayElement(ch1, new(shape, 30e-9, 0), 0, 0, 0.5, 2e-9) { Margin = new(15e-9) });
    stack.Add(stack2);

    stack.Add(new BarrierElement(ch1, ch2) { Margin = new(15e-9) });

    var grid = new GridSchedule() { Duration = 500e-9 };
    grid.AddColumn(GridLength.Absolute(90e-9));
    grid.AddColumn(GridLength.Star(1));
    grid.AddColumn(GridLength.Absolute(90e-9));
    grid.Add(new PlayElement(ch1, new(shape, 30e-9, 200e-9), 0, 0, 0.5, 2e-9) { Alignment = Qynit.PulseGen.Alignment.Center }, 1, 1);
    grid.Add(new PlayElement(ch2, new(shape, 30e-9, 50e-9), -250e6, 0, 0.6, 2e-9) { Alignment = Qynit.PulseGen.Alignment.Stretch, FlexiblePlateau = true }, 0, 3);
    stack.Add(grid);

    stack.Add(new ShiftFrequencyElement(ch1, -100e6));
    stack.Add(new ShiftFrequencyElement(ch2, -250e6));
    stack.Add(new BarrierElement(ch1, ch2) { Margin = new(15e-9) });

    var abs = new AbsoluteSchedule();
    abs.Add(new PlayElement(ch1, new(null, 200e-9, 0), 0, 0, 0.5, 2e-9), 10e-9);
    abs.Add(new RepeatElement(new PlayElement(ch2, new(null, 100e-9, 0), 0, 0, 0.6, 2e-9), 2) { Spacing = 10e-9 }, 210e-9);
    stack.Add(abs);

    stack.Add(new SetFrequencyElement(ch1, 0));
    stack.Add(new SetFrequencyElement(ch2, 0));
    stack.Add(new BarrierElement(ch1, ch2) { Margin = new(15e-9) });
    stack.Add(new PlayElement(ch1, new(shape, 200e-9, 0), 0, 0, 0.5, 2e-9));
    stack.Add(new PlayElement(ch2, new(shape, 100e-9, 0), 0, 0, 0.6, 2e-9));
    var main = new GridSchedule();
    main.Add(stack);
    main.Measure(49.9e-6);
    main.Arrange(0, 49.9e-6);
    main.Render(0, phaseTrackingTransform);

    var t1 = sw.Elapsed;

    var pulseLists = phaseTrackingTransform.Finish();
    var t2 = sw.Elapsed;

    var sampleRate = 2e9;
    var n = 100000;
    var alignLevel = -4;
    var waveforms = pulseLists.Select(x => WaveformUtils.SampleWaveform<double>(x, sampleRate, 0, n, alignLevel)).ToList();
    var t3 = sw.Elapsed;
    sw.Stop();
    Console.WriteLine($"Instructions time: {t1.TotalMilliseconds} ms");
    Console.WriteLine($"Build time: {(t2 - t1).TotalMilliseconds} ms");
    Console.WriteLine($"Sampling time: {(t3 - t2).TotalMilliseconds} ms");
    Console.WriteLine($"Total elapsed time: {sw.Elapsed.TotalMilliseconds} ms");
    using var waveform1 = waveforms[ch1];
    using var waveform2 = waveforms[ch2];
    var plot = new Plot(1920, 1080);
    plot.AddSignal(waveform1.DataI[^4000..].ToArray(), sampleRate, label: $"wave 1 real");
    plot.AddSignal(waveform1.DataQ[^4000..].ToArray(), sampleRate, label: $"wave 1 imag");
    plot.AddSignal(waveform2.DataI[^4000..].ToArray(), sampleRate, label: $"wave 2 real");
    plot.AddSignal(waveform2.DataQ[^4000..].ToArray(), sampleRate, label: $"wave 2 imag");
    plot.Legend();
    plot.SaveFig("demo2.png");
}
