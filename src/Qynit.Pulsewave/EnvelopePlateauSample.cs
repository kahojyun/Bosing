namespace Qynit.Pulsewave;
internal record EnvelopePlateauSample<T>(PooledComplexArray<T> LeftEdge, PooledComplexArray<T> RightEdge, int Plateau) : EnvelopeSample<T>
    where T : unmanaged
{
    public override void Dispose()
    {
        LeftEdge.Dispose();
        RightEdge.Dispose();
    }
}
