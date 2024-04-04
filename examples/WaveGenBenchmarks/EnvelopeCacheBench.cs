using BenchmarkDotNet.Attributes;

using Bosing;

namespace WaveGenBenchmarks;

[MemoryDiagnoser]
public class EnvelopeCacheBench
{
    private double[] _tStarts = null!;
    private readonly IPulseShape _pulseShape = new HannPulseShape();

    [GlobalSetup]
    public void Init()
    {
        var rng = new Random(42);
        _tStarts = new double[1000];
        for (var i = 0; i < _tStarts.Length; i++)
        {
            _tStarts[i] = rng.NextDouble() * 1e-6;
        }
    }

    [Benchmark]
    public void GetEnvelopeFromCache()
    {
        var sampleRate = 1e9;
        var alignLevel = -4;
        var envelope = new Envelope(_pulseShape, 30e-9, 10e-9);
        foreach (var tStart in _tStarts)
        {
            var iFracStart = TimeAxisUtils.NextFracIndex(tStart, sampleRate, alignLevel);
            var iStart = (int)Math.Ceiling(iFracStart);
            var envelopeInfo = new EnvelopeInfo(iStart - iFracStart, sampleRate);
            _ = WaveformSampler<double>.GetEnvelopeSample(envelopeInfo, envelope);
        }
    }
}
