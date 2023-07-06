using System.Diagnostics;
using System.Numerics;

using Qynit.PulseGen;

for (var i = 0; i < 5; i++)
{
    RunDouble();
}

for (var i = 0; i < 5; i++)
{
    RunSingle();
}

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
    phaseTrackingTransform.Play(ch1, new(shape, 30e-9, 100e-9), 0, 0, 0.5, 2e-9, 0);
    phaseTrackingTransform.Play(ch2, new(shape, 30e-9, 100e-9), 0, 0, 0.6, 2e-9, 0);
    phaseTrackingTransform.ShiftPhase(ch1, 0.25);
    phaseTrackingTransform.ShiftPhase(ch2, -0.25);
    phaseTrackingTransform.Play(ch1, new(shape, 30e-9, 100e-9), 0, 0, 0.5, 2e-9, 200e-9);
    phaseTrackingTransform.Play(ch2, new(shape, 30e-9, 100e-9), 0, 0, 0.6, 2e-9, 200e-9);
    phaseTrackingTransform.ShiftFrequency(ch1, -100e6, 400e-9);
    phaseTrackingTransform.ShiftFrequency(ch2, -250e6, 400e-9);
    phaseTrackingTransform.Play(ch1, new(shape, 200e-9, 0), 0, 0, 0.5, 2e-9, 400e-9);
    phaseTrackingTransform.Play(ch2, new(shape, 200e-9, 0), 0, 0, 0.6, 2e-9, 400e-9);
    phaseTrackingTransform.SetFrequency(ch1, 0, 600e-9);
    phaseTrackingTransform.SetFrequency(ch2, 0, 600e-9);
    var tStart = 600e-9;
    var count = 0;
    while (tStart < 49e-6)
    {
        phaseTrackingTransform.Play(ch1, new(shape, 30e-9, 0e-9), 0, 0, 0.5, 2e-9, tStart);
        phaseTrackingTransform.Play(ch2, new(shape, 30e-9, 0e-9), 0, 0, 0.6, 2e-9, tStart);
        phaseTrackingTransform.ShiftPhase(ch1, 0.25);
        phaseTrackingTransform.ShiftPhase(ch2, -0.25);
        tStart += 0.1e-9;
        count++;
    }
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
    Console.WriteLine($"Count = {count}");
    using var waveform1 = waveforms[ch1];
    using var waveform2 = waveforms[ch2];
    //var plot = new Plot(1920, 1080);
    //plot.AddSignal(waveform1.DataI[..2000].ToArray(), sampleRate, label: $"wave 1 real");
    //plot.AddSignal(waveform1.DataQ[..2000].ToArray(), sampleRate, label: $"wave 1 imag");
    //plot.AddSignal(waveform2.DataI[..2000].ToArray(), sampleRate, label: $"wave 2 real");
    //plot.AddSignal(waveform2.DataQ[..2000].ToArray(), sampleRate, label: $"wave 2 imag");
    //plot.Legend();
    //plot.SaveFig("demo2.png");
}
