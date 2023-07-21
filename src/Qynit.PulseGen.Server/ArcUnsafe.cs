namespace Qynit.PulseGen.Server;

public static class ArcUnsafe
{
    public static ArcUnsafe<T> Wrap<T>(T disposable) where T : IDisposable
    {
        return new ArcUnsafe<T>(disposable);
    }
}

public readonly struct ArcUnsafe<T> : IDisposable where T : IDisposable
{
    private readonly RefCountBox _refCountBox;
    public T Target { get; }

    public ArcUnsafe(T disposable)
    {
        _refCountBox = new RefCountBox();
        Target = disposable;
    }

    public ArcUnsafe<T> Clone()
    {
        _refCountBox.Acquire();
        return this;
    }

    public void Dispose()
    {
        var refCount = _refCountBox.Release();
        if (refCount == 0)
        {
            Target.Dispose();
        }
    }

    private class RefCountBox
    {
        private int _referenceCount;
        public int Acquire()
        {
            return Interlocked.Increment(ref _referenceCount);
        }

        public int Release()
        {
            return Interlocked.Decrement(ref _referenceCount);
        }

        public RefCountBox()
        {
            Acquire();
        }
    }
}
