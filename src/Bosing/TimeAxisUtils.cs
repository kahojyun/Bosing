namespace Bosing;
public static class TimeAxisUtils
{
    public static int NextIndex(double t, double sampleRate)
    {
        return (int)Math.Ceiling(t * sampleRate);
    }
    public static double NextFracIndex(double t, double sampleRate, int alignLevel)
    {
        var scaledSampleRate = Math.ScaleB(sampleRate, -alignLevel);
        var alignIndex = Math.Ceiling(t * scaledSampleRate);
        return Math.ScaleB(alignIndex, alignLevel);
    }
    public static int PrevIndex(double t, double sampleRate)
    {
        return (int)Math.Floor(t * sampleRate);
    }
    public static double PrevFracIndex(double t, double sampleRate, int alignLevel)
    {
        var scaledSampleRate = Math.ScaleB(sampleRate, -alignLevel);
        var alignIndex = Math.Floor(t * scaledSampleRate);
        return Math.ScaleB(alignIndex, alignLevel);
    }
    public static int ClosestIndex(double t, double sampleRate)
    {
        return (int)Math.Round(t * sampleRate);
    }
    public static double ClosestFracIndex(double t, double sampleRate, int alignLevel)
    {
        var scaledSampleRate = Math.ScaleB(sampleRate, -alignLevel);
        var alignIndex = Math.Round(t * scaledSampleRate);
        return Math.ScaleB(alignIndex, alignLevel);
    }
    public static (int Start, int End) GetIndexRange(double tStart, double tEnd, double sampleRate)
    {
        var iStart = NextIndex(tStart, sampleRate);
        var iEnd = NextIndex(tEnd, sampleRate);
        return (iStart < iEnd) ? (iStart, iEnd) : (iEnd, iStart);
    }
}
