namespace Qynit.Pulsewave;
internal abstract record EnvelopeSample<T> : IDisposable
    where T : unmanaged
{
    public abstract void Dispose();
}
