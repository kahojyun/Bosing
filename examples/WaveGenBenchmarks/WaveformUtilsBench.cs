using System.Numerics;

using BenchmarkDotNet.Attributes;

using Bosing;

namespace WaveGenBenchmarks;
public class WaveformUtilsBench
{
    //[Params(16, 64, 256, 1024)]
    [Params(1024)]
    public int Length { get; set; }
    private PooledComplexArray<double>? Source { get; set; }
    private PooledComplexArray<double>? Target { get; set; }
    private IqPair<double> Amplitude { get; } = new IqPair<double>(1, 1);
    private IqPair<double> DragAmplitude { get; } = new IqPair<double>(1, 1);
    private double DPhase { get; } = Math.PI / 11;

    [GlobalSetup]
    public void Init()
    {
        Source = new PooledComplexArray<double>(Length, true);
        Source.DataI.Fill(0.125);
        Source.DataQ.Fill(-0.125);
        Target = new PooledComplexArray<double>(Length, true);
    }

    [Benchmark]
    public void MixAddPlateau()
    {
        WaveformUtils.MixAddPlateau(Target!, Amplitude, 0);
    }

    [Benchmark]
    public void MixAddPlateauFrequency()
    {
        WaveformUtils.MixAddPlateau(Target!, Amplitude, DPhase);
    }

    [Benchmark]
    public void MixAdd()
    {
        WaveformUtils.MixAdd(Target!, Source!, Amplitude, 0, 0);
    }

    [Benchmark]
    public void MixAddFrequency()
    {
        WaveformUtils.MixAdd(Target!, Source!, Amplitude, 0, DPhase);
    }

    [Benchmark]
    public void MixAddWithDrag()
    {
        WaveformUtils.MixAdd(Target!, Source!, Amplitude, DragAmplitude, 0);
    }

    [Benchmark(Baseline = true)]
    public void MixAddFrequencyWithDrag()
    {
        WaveformUtils.MixAdd(Target!, Source!, Amplitude, DragAmplitude, DPhase);
    }

    [Benchmark]
    public void Simple()
    {
        MixAddWithDragSimple(Target!, Source!, Amplitude, DragAmplitude, DPhase);
    }

    private static void MixAddWithDragSimple<T>(ComplexSpan<T> target, ComplexReadOnlySpan<T> source, IqPair<T> amplitude, IqPair<T> dragAmplitude, T dPhase)
       where T : unmanaged, IFloatingPointIeee754<T>
    {
        var length = source.Length;
        var sourceI = source.DataI;
        var sourceQ = source.DataQ;
        var targetI = target.DataI;
        var targetQ = target.DataQ;

        var carrier = amplitude;
        var dragCarrier = dragAmplitude;
        var phaser = IqPair<T>.FromPolarCoordinates(T.One, dPhase);
        for (var i = 0; i < length; i++)
        {
            var diff = i switch
            {
                0 => new IqPair<T>(sourceI[i + 1], sourceQ[i + 1]) - new IqPair<T>(sourceI[i], sourceQ[i]),
                _ when i == length - 1 => new IqPair<T>(sourceI[i], sourceQ[i]) - new IqPair<T>(sourceI[i - 1], sourceQ[i - 1]),
                _ => (new IqPair<T>(sourceI[i + 1], sourceQ[i + 1]) - new IqPair<T>(sourceI[i - 1], sourceQ[i - 1])) * T.CreateChecked(0.5),
            };
            var sourceIq = new IqPair<T>(sourceI[i], sourceQ[i]);
            var totalIq = sourceIq * carrier + diff * dragCarrier;
            targetI[i] += totalIq.I;
            targetQ[i] += totalIq.Q;
            carrier *= phaser;
            dragCarrier *= phaser;
        }
    }
}
