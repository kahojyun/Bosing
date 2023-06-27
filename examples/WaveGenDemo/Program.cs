using System.Diagnostics;
using System.Numerics;

using Qynit.Pulsewave;

for (var i = 0; i < 5; i++)
{
    RunDouble();
}

for (var i = 0; i < 5; i++)
{
    RunSingle();
}

static void ConnectNode<T>(IFilterNode<T> source, IFilterNode<T> target) where T : unmanaged, IFloatingPointIeee754<T>
{
    source.Outputs.Add(target);
    target.Inputs.Add(source);
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

    var ch1 = new Channel("ch1");
    var ch2 = new Channel("ch2");

    var inputNode1 = new InputNode<T>();
    var inputNode2 = new InputNode<T>();

    var n = 100000;
    var sampleRate = 2e9;
    var outputNode1 = new OutputNode<T>(n, sampleRate, 0, -4);
    var outputNode2 = new OutputNode<T>(n, sampleRate, 0, -4);

    ConnectNode(inputNode1, outputNode1);
    ConnectNode(inputNode2, outputNode2);

    var generator = new WaveformGenerator<T>();
    generator.AddChannel(ch1, inputNode1, 100e6);
    generator.AddChannel(ch2, inputNode2, 250e6);

    var instructions = new List<Instruction>();
    var shape = new HannPulseShape();
    instructions.Add(new Play(shape, 0, 30e-9, 100e-9, 0.5, 0, 0, ch1));
    instructions.Add(new Play(shape, 0, 30e-9, 100e-9, 0.6, 0, 0, ch2));
    instructions.Add(new ShiftPhase(0.25 * Math.Tau, ch1));
    instructions.Add(new ShiftPhase(-0.25 * Math.Tau, ch2));
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
        //instructions.Add(new Play(shape, tStart, 30e-9, 0, 0.5, 0, 0, ch1));
        //instructions.Add(new Play(shape, tStart, 30e-9, 0, 0.6, 0, 0, ch2));
        instructions.Add(new Play(shape, tStart, 30e-9, 500e-9, 0.5, 0, 0, ch1));
        instructions.Add(new Play(shape, tStart, 30e-9, 500e-9, 0.6, 0, 0, ch2));
        instructions.Add(new ShiftPhase(0.25 * Math.Tau, ch1));
        instructions.Add(new ShiftPhase(-0.25 * Math.Tau, ch2));
        tStart += 0.1e-9;
        count++;
    }

    var t1 = sw.Elapsed;

    generator.Run(instructions);

    sw.Stop();
    var t2 = sw.Elapsed;
    Console.WriteLine($"Instructions time: {t1.TotalMilliseconds} ms");
    Console.WriteLine($"Run time: {(t2 - t1).TotalMilliseconds} ms");
    Console.WriteLine($"Total elapsed time: {sw.Elapsed.TotalMilliseconds} ms");
    Console.WriteLine($"Count = {count}");
    using var waveform1 = outputNode1.TakeWaveform();
    using var waveform2 = outputNode2.TakeWaveform();
    //var plot = new Plot(1920, 1080);
    //plot.AddSignal(waveform1.DataI[..300].ToArray(), sampleRate, label: $"wave 1 real");
    //plot.AddSignal(waveform1.DataQ[..300].ToArray(), sampleRate, label: $"wave 1 imag");
    //plot.AddSignal(waveform2.DataI[..300].ToArray(), sampleRate, label: $"wave 2 real");
    //plot.AddSignal(waveform2.DataQ[..300].ToArray(), sampleRate, label: $"wave 2 imag");
    //plot.Legend();
    //plot.SaveFig("demo2.png");
}
