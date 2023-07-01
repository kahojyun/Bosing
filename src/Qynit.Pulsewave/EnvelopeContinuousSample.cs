namespace Qynit.Pulsewave;
internal record EnvelopeContinuousSample<T>(PooledComplexArray<T> Envelope) : EnvelopeSample<T>
    where T : unmanaged
{
    public override void Dispose()
    {
        Envelope.Dispose();
    }
}
