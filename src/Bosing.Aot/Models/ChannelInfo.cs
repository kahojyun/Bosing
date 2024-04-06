using MessagePack;

namespace Bosing.Aot.Models;

[MessagePackObject]
public sealed record ChannelInfo(
    [property: Key(0)] string Name,
    [property: Key(1)] double BaseFrequency,
    [property: Key(2)] double SampleRate,
    [property: Key(3)] int Length,
    [property: Key(4)] double Delay,
    [property: Key(5)] int AlignLevel,
    [property: Key(6)] IqCalibration? IqCalibration,
    [property: Key(7)] IList<BiquadDto> BiquadChain,
    [property: Key(8)] IList<double> FirCoefficients);
