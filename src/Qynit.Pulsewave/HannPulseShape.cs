﻿using System.Numerics;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Qynit.Pulsewave;
public sealed class HannPulseShape : IPulseShape
{
    public IqPair<T> SampleAt<T>(T x)
        where T : unmanaged, IFloatingPointIeee754<T>
    {
        var half = T.CreateChecked(0.5);
        var i = (x >= -half && x <= half) ? (T.One + T.Cos(T.Tau * x)) * half : T.Zero;
        return i;
    }

    public void SampleIQ<T>(ComplexArraySpan<T> target, T x0, T dx)
        where T : unmanaged, IFloatingPointIeee754<T>
    {
        var length = target.Length;
        if (length == 0)
        {
            return;
        }
        var half = T.CreateChecked(0.5);
        var c = IqPair<T>.FromPolarCoordinates(half, T.Tau * x0);
        var w = IqPair<T>.FromPolarCoordinates(T.One, T.Tau * dx);
        ref var ti = ref MemoryMarshal.GetReference(target.DataI);
        ref var tq = ref MemoryMarshal.GetReference(target.DataQ);
        for (var i = 0; i < length; i++)
        {
            var x = x0 + T.CreateTruncating(i) * dx;
            Unsafe.Add(ref ti, i) = (x >= -half && x <= half) ? half + c.I : T.Zero;
            Unsafe.Add(ref tq, i) = T.Zero;
            c *= w;
        }
    }
}
