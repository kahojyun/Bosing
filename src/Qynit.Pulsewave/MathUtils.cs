using System.Numerics;

namespace Qynit.Pulsewave;
public static class MathUtils
{
    public static T MRound<T>(T number, T multiple) where T : IFloatingPoint<T>
    {
        return T.Round(number / multiple) * multiple;
    }

    public static T MFloor<T>(T number, T multiple) where T : IFloatingPoint<T>
    {
        return T.Floor(number / multiple) * multiple;
    }

    public static T MCeiling<T>(T number, T multiple) where T : IFloatingPoint<T>
    {
        return T.Ceiling(number / multiple) * multiple;
    }
}
