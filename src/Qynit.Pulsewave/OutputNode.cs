using System.Diagnostics.CodeAnalysis;
using System.Numerics;

using CommunityToolkit.Diagnostics;

namespace Qynit.Pulsewave;
public class OutputNode<T> : IFilterNode<T>
    where T : unmanaged, IFloatingPointIeee754<T>
{
    public OutputNode(int length, double sampleRate, double tStart, int alignLevel)
    {
        var iStart = TimeAxisUtils.ClosestIndex(tStart, sampleRate);
        _waveformInfo = new WaveformInfo(iStart, sampleRate);
        Length = length;
        AlignLevel = alignLevel;
    }

    public double SampleRate => _waveformInfo.SampleRate;
    public double TStart => _waveformInfo.IndexStart / SampleRate;
    public double TEnd => (_waveformInfo.IndexStart + Length) / SampleRate;
    public int IndexStart => _waveformInfo.IndexStart;
    public int Length { get; }
    public int AlignLevel { get; }
    public string? Name { get; set; }
    public IList<IFilterNode<T>> Outputs { get; } = Array.Empty<IFilterNode<T>>();
    public IList<IFilterNode<T>> Inputs { get; } = new List<IFilterNode<T>>();

    private PooledComplexArray<T>? _array;
    private readonly WaveformInfo _waveformInfo;


    public void Initialize()
    {
        _array?.Dispose();
        _array = new PooledComplexArray<T>(Length, true);
    }

    public void Complete()
    {
    }

    public void AddPulse(IPulseShape shape, double tStart, double width, double plateau, double amplitude, double frequency, double phase, double referenceTime)
    {
        EnsureInitialized();
        var iFracStart = TimeAxisUtils.ClosestFracIndex(tStart, SampleRate, AlignLevel);
        var iStart = (int)Math.Ceiling(iFracStart);
        var envelopeInfo = new EnvelopeInfo(iStart - iFracStart, SampleRate);
        using var envelope = WaveformUtils.SampleWaveform<T>(envelopeInfo, shape, width, plateau);
        var dt = 1 / SampleRate;
        var phaseStart = (iStart * dt - referenceTime) * frequency * Math.Tau + phase;
        var cPhase = IqPair<T>.FromPolarCoordinates(T.CreateChecked(amplitude), T.CreateChecked(phaseStart));
        var dPhase = T.CreateChecked(Math.Tau * frequency * dt);
        var arrayIStart = iStart - IndexStart;
        WaveformUtils.MixAddFrequency(_array[arrayIStart..], envelope, cPhase, dPhase);
    }

    public void AddWaveform(ComplexArrayReadOnlySpan<T> waveform, WaveformInfo waveformInfo, double amplitude, double frequency, double phase, double referenceTime)
    {
        EnsureInitialized();
        if (waveformInfo.SampleRate != SampleRate)
        {
            ThrowHelper.ThrowArgumentException("Sample rate of waveform does not match.");
        }
        var arrayIStart = waveformInfo.IndexStart - IndexStart;
        var arrayIEnd = arrayIStart + waveform.Length;
        if (arrayIStart < 0 || arrayIEnd > Length)
        {
            ThrowHelper.ThrowArgumentOutOfRangeException("Index out of range");
        }
        var dt = 1 / SampleRate;
        var phaseStart = (waveformInfo.IndexStart * dt - referenceTime) * frequency * Math.Tau + phase;
        var cPhase = IqPair<T>.FromPolarCoordinates(T.CreateChecked(amplitude), T.CreateChecked(phaseStart));
        var dPhase = T.CreateChecked(Math.Tau * frequency * dt);
        WaveformUtils.MixAddFrequency(_array[arrayIStart..arrayIEnd], waveform, cPhase, dPhase);
    }

    public PooledComplexArray<T> TakeWaveform()
    {
        EnsureInitialized();
        var waveform = _array;
        _array = null;
        return waveform;
    }

    [MemberNotNull(nameof(_array))]
    private void EnsureInitialized()
    {
        if (_array is null)
        {
            ThrowHelper.ThrowInvalidOperationException($"{nameof(OutputNode<T>)} {Name} is not initialized.");
        }
    }
}
