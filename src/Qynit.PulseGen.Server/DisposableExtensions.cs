namespace Qynit.PulseGen.Server;

internal static class DisposableExtensions
{
    public static ArcUnsafe<T> ToArc<T>(this T disposable) where T : IDisposable
    {
        return new ArcUnsafe<T>(disposable);
    }
}
