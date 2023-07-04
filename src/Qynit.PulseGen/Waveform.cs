namespace Qynit.PulseGen;

/// <summary>
/// Store waveform data with pooled array.
/// </summary>
/// <remarks>
/// <c>TStart</c> will be aligned to <c>Dt</c>.
/// </remarks>
public sealed class Waveform<T> : IDisposable
    where T : unmanaged
{
    public double SampleRate { get; }
    public int IndexStart { get; set; }
    public int Length => _array.Length;
    public double Dt => 1 / SampleRate;
    public double TStart => IndexStart * Dt;
    public ComplexSpan<T> Array => _array;

    private readonly PooledComplexArray<T> _array;
    private bool _shouldDispose;

    public Waveform(int indexStart, int length, double sampleRate) : this(indexStart, length, sampleRate, true) { }

    public Waveform(Waveform<T> source) : this(source.IndexStart, source.Length, source.SampleRate, false)
    {
        source._array.CopyTo(_array);
    }

    public Waveform(ComplexReadOnlySpan<T> source, int indexStart, double sampleRate) : this(indexStart, source.Length, sampleRate, false)
    {
        source.CopyTo(_array);
    }

    public Waveform(PooledComplexArray<T> array, int indexStart, double sampleRate)
    {
        SampleRate = sampleRate;
        IndexStart = indexStart;
        _array = array;
        _shouldDispose = true;
    }

    private Waveform(int indexStart, int length, double sampleRate, bool clear)
    {
        SampleRate = sampleRate;
        IndexStart = indexStart;
        _array = new PooledComplexArray<T>(length, clear);
        _shouldDispose = true;
    }

    public static Waveform<T> CreateFromRange(double sampleRate, double tStart, double tEnd)
    {
        var (start, end) = TimeAxisUtils.GetIndexRange(tStart, tEnd, sampleRate);
        return new Waveform<T>(start, end - start, sampleRate);
    }
    public PooledComplexArray<T> TakeArray()
    {
        _shouldDispose = false;
        return _array;
    }
    public void Dispose()
    {
        if (_shouldDispose)
        {
            _array.Dispose();
        }
    }
    public Waveform<T> Copy()
    {
        return new Waveform<T>(this);
    }
}
