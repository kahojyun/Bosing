using System.Buffers;

using CommunityToolkit.Diagnostics;

namespace Qynit.Pulsewave;

/// <summary>
/// Store waveform data with pooled array.
/// </summary>
public sealed class Waveform : IDisposable
{
    public double SampleRate { get; }
    public double Dt => 1 / SampleRate;
    public double TStart { get; private set; }
    public double TEnd => TStart + Length * Dt;
    public int Length { get; }
    public Span<double> DataI
    {
        get
        {
            if (_disposed)
            {
                ThrowHelper.ThrowObjectDisposedException(nameof(Waveform));
            }
            return _dataI.AsSpan(0, Length);
        }
    }
    public Span<double> DataQ
    {
        get
        {
            if (_disposed)
            {
                ThrowHelper.ThrowObjectDisposedException(nameof(Waveform));
            }
            return _dataQ.AsSpan(0, Length);
        }
    }

    private readonly double[] _dataI;
    private readonly double[] _dataQ;
    private bool _disposed;

    public Waveform(int length, double sampleRate, double t0)
    {
        Length = length;
        SampleRate = sampleRate;
        TStart = t0;
        _dataI = ArrayPool<double>.Shared.Rent(length);
        _dataQ = ArrayPool<double>.Shared.Rent(length);
        _dataI.AsSpan(0, length).Clear();
        _dataQ.AsSpan(0, length).Clear();
    }

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;
        _dataI.AsSpan(0, Length).Clear();
        _dataQ.AsSpan(0, Length).Clear();
        ArrayPool<double>.Shared.Return(_dataI, false);
        ArrayPool<double>.Shared.Return(_dataQ, false);
    }

    public void ShiftTime(double deltaT, bool alignToDt)
    {
        if (!alignToDt)
        {
            TStart += deltaT;
            return;
        }
        var samples = deltaT * SampleRate;
        samples = Math.Round(samples);
        TStart += samples / SampleRate;
    }

    public double TimeAt(int index)
    {
        return TStart + index / SampleRate;
    }
}