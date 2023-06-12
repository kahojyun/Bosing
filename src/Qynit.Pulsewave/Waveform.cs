using System.Buffers;
using System.Diagnostics;

using CommunityToolkit.Diagnostics;

namespace Qynit.Pulsewave;

/// <summary>
/// Store waveform data with pooled array.
/// </summary>
/// <remarks>
/// <c>TStart</c> will be aligned to <c>Dt</c>.
/// </remarks>
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

    public Waveform(int length, double sampleRate, double tStart) : this(length, sampleRate, tStart, true) { }

    public Waveform(Waveform waveform) : this(waveform.Length, waveform.SampleRate, waveform.TStart, false)
    {
        waveform.DataI.CopyTo(DataI);
        waveform.DataQ.CopyTo(DataQ);
    }

    private Waveform(int length, double sampleRate, double tStart, bool clear)
    {
        Debug.Assert(length > 0);
        Debug.Assert(sampleRate > 0);
        Length = length;
        SampleRate = sampleRate;
        TStart = MathUtils.MRound(tStart, 1 / sampleRate);
        _dataI = ArrayPool<double>.Shared.Rent(length);
        _dataQ = ArrayPool<double>.Shared.Rent(length);
        if (clear)
        {
            ClearData();
        }
    }

    public static Waveform CreateFromRange(double sampleRate, double tStart, double tEnd)
    {
        var dt = 1 / sampleRate;
        tStart = MathUtils.MFloor(tStart, dt);
        tEnd = MathUtils.MCeiling(tEnd, dt);
        var length = (int)Math.Round((tEnd - tStart) * sampleRate) + 1;
        return new Waveform(length, sampleRate, tStart);
    }

    private void ClearData()
    {
        _dataI.AsSpan(0, Length).Clear();
        _dataQ.AsSpan(0, Length).Clear();
    }

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;
        ClearData();
        ArrayPool<double>.Shared.Return(_dataI, false);
        ArrayPool<double>.Shared.Return(_dataQ, false);
    }

    public void ShiftTime(double deltaT)
    {
        TStart = MathUtils.MRound(TStart + deltaT, Dt);
    }

    public double TimeAt(int index)
    {
        return TStart + index * Dt;
    }

    public Waveform Copy()
    {
        return new Waveform(this);
    }
}