using System.Diagnostics;
using System.Numerics;

namespace Bosing.Tests;

public class WaveformUtilsTests
{
    private const double MixFrequency = 253.1033e6;

    [Fact]
    public void SampleWaveform_Double_Equal()
    {
        // Arrange
        var envelopeInfo = new EnvelopeInfo(0.9, 1e9);
        var shape = new TrianglePulseShape();
        var width = 30e-9;
        var plateau = 40e-9;
        var envelope = new Envelope(shape, width, plateau);

        // Act
        var result = WaveformSampler<double>.GetEnvelopeSample(
            envelopeInfo,
            envelope);

        // Assert
        var index = new[] { 0, 9, 14, 24, 54, 55, 64, 69 };
        var valueI = new[] { 0.9 / 15, 9.9 / 15, 14.9 / 15, 1, 1, 14.1 / 15, 5.1 / 15, 0.1 / 15 };
        var valueQ = new double[index.Length];
        // TODO: get continuous envelope
        Assert.NotNull(result);
        Assert.Equal(0, result.Plateau);
        Assert.True(result.RightEdge.IsEmpty);
        var resultI = index.Select(i => result.LeftEdge.DataI[i]);
        var resultQ = index.Select(i => result.LeftEdge.DataQ[i]);
        var comparer = new ToleranceComparer(1e-9);
        Assert.Equal(valueI, resultI, comparer);
        Assert.Equal(valueQ, resultQ, comparer);
        var expectedLength = (int)Math.Round((width + plateau) * envelopeInfo.SampleRate);
        Assert.Equal(expectedLength, result.LeftEdge.Length);
    }

    [Fact]
    public void MixAddFrequency_Double_Equal()
    {
        // Arrange
        var sampleRate = 1e9;
        var envelope = GetEnvelope(sampleRate);
        var additionalLength = 10;
        using var expected = GetBuffer(envelope.LeftEdge, additionalLength);
        using var target = expected.Copy();

        var amplitude = 0.5;
        var frequency = MixFrequency;
        var phase = Math.PI / 6;

        var cAmplitude = IqPair<double>.FromPolarCoordinates(amplitude, phase);
        var dragAmplitude = IqPair<double>.Zero;
        var dPhase = Math.Tau * frequency / sampleRate;

        // Act
        WaveformUtils.MixAddFrequencyCore(target, envelope.LeftEdge, cAmplitude, dPhase);

        // Assert
        MixAddWithDragSimple(expected, envelope.LeftEdge, cAmplitude, dragAmplitude, dPhase);
        var comparer = new ToleranceComparer(1e-9);
        Assert.Equal(target.DataI.ToArray(), expected.DataI.ToArray(), comparer);
        Assert.Equal(target.DataQ.ToArray(), expected.DataQ.ToArray(), comparer);
    }

    [Fact]
    public void MixAdd_Double_Equal()
    {
        // Arrange
        var sampleRate = 1e9;
        var envelope = GetEnvelope(sampleRate);
        var additionalLength = 10;
        using var expected = GetBuffer(envelope.LeftEdge, additionalLength);
        using var target = expected.Copy();

        var amplitude = 0.5;
        var frequency = 0;
        var phase = Math.PI / 6;

        var cAmplitude = IqPair<double>.FromPolarCoordinates(amplitude, phase);
        var dragAmplitude = IqPair<double>.Zero;
        var dPhase = Math.Tau * frequency / sampleRate;

        // Act
        WaveformUtils.MixAddCore(target, envelope.LeftEdge, cAmplitude);

        // Assert
        MixAddWithDragSimple(expected, envelope.LeftEdge, cAmplitude, dragAmplitude, dPhase);
        var comparer = new ToleranceComparer(1e-9);
        Assert.Equal(target.DataI.ToArray(), expected.DataI.ToArray(), comparer);
        Assert.Equal(target.DataQ.ToArray(), expected.DataQ.ToArray(), comparer);
    }

    [Fact]
    public void MixAddPlateauFrequency_Double_Equal()
    {
        // Arrange
        var sampleRate = 1e9;
        using var envelope = GetPlateau();
        var additionalLength = 0;
        using var expected = GetBuffer(envelope, additionalLength);
        using var target = expected.Copy();

        var amplitude = 0.5;
        var frequency = MixFrequency;
        var phase = Math.PI / 6;

        var cAmplitude = IqPair<double>.FromPolarCoordinates(amplitude, phase);
        var dragAmplitude = IqPair<double>.Zero;
        var dPhase = Math.Tau * frequency / sampleRate;

        // Act
        WaveformUtils.MixAddPlateauFrequencyCore(target, cAmplitude, dPhase);

        // Assert
        MixAddWithDragSimple(expected, envelope, cAmplitude, dragAmplitude, dPhase);
        var comparer = new ToleranceComparer(1e-9);
        Assert.Equal(target.DataI.ToArray(), expected.DataI.ToArray(), comparer);
        Assert.Equal(target.DataQ.ToArray(), expected.DataQ.ToArray(), comparer);
    }

    [Fact]
    public void MixAddPlateau_Double_Equal()
    {
        // Arrange
        var sampleRate = 1e9;
        using var envelope = GetPlateau();
        var additionalLength = 0;
        using var expected = GetBuffer(envelope, additionalLength);
        using var target = expected.Copy();

        var amplitude = 0.5;
        var frequency = 0;
        var phase = Math.PI / 6;

        var cAmplitude = IqPair<double>.FromPolarCoordinates(amplitude, phase);
        var dragAmplitude = IqPair<double>.Zero;
        var dPhase = Math.Tau * frequency / sampleRate;

        // Act
        WaveformUtils.MixAddPlateauCore(target, cAmplitude);

        // Assert
        MixAddWithDragSimple(expected, envelope, cAmplitude, dragAmplitude, dPhase);
        var comparer = new ToleranceComparer(1e-9);
        Assert.Equal(target.DataI.ToArray(), expected.DataI.ToArray(), comparer);
        Assert.Equal(target.DataQ.ToArray(), expected.DataQ.ToArray(), comparer);
    }

    [Fact]
    public void MixAddFrequencyWithDrag_Double_Equal()
    {
        // Arrange
        var sampleRate = 1e9;
        var envelope = GetEnvelope(sampleRate);
        var additionalLength = 10;
        using var expected = GetBuffer(envelope.LeftEdge, additionalLength);
        using var target = expected.Copy();

        var amplitude = 0.5;
        var frequency = MixFrequency;
        var phase = Math.PI / 6;
        var dragCoefficient = 2e-9;

        var cAmplitude = IqPair<double>.FromPolarCoordinates(amplitude, phase);
        var dragAmplitude = cAmplitude * dragCoefficient * sampleRate * IqPair<double>.ImaginaryOne;
        var dPhase = Math.Tau * frequency / sampleRate;

        // Act
        WaveformUtils.MixAddFrequencyWithDragCore(target, envelope.LeftEdge, cAmplitude, dragAmplitude, dPhase);

        // Assert
        MixAddWithDragSimple(expected, envelope.LeftEdge, cAmplitude, dragAmplitude, dPhase);
        var comparer = new ToleranceComparer(1e-9);
        Assert.Equal(target.DataI.ToArray(), expected.DataI.ToArray(), comparer);
        Assert.Equal(target.DataQ.ToArray(), expected.DataQ.ToArray(), comparer);
    }

    [Fact]
    public void MixAddWithDrag_Double_Equal()
    {
        // Arrange
        var sampleRate = 1e9;
        var envelope = GetEnvelope(sampleRate);
        var additionalLength = 10;
        using var expected = GetBuffer(envelope.LeftEdge, additionalLength);
        using var target = expected.Copy();

        var amplitude = 0.5;
        var frequency = 0;
        var phase = Math.PI / 6;
        var dragCoefficient = 2e-9;

        var cAmplitude = IqPair<double>.FromPolarCoordinates(amplitude, phase);
        var dragAmplitude = cAmplitude * dragCoefficient * sampleRate * IqPair<double>.ImaginaryOne;
        var dPhase = Math.Tau * frequency / sampleRate;

        // Act
        WaveformUtils.MixAddWithDragCore(target, envelope.LeftEdge, cAmplitude, dragAmplitude);

        // Assert
        MixAddWithDragSimple(expected, envelope.LeftEdge, cAmplitude, dragAmplitude, dPhase);
        var comparer = new ToleranceComparer(1e-9);
        Assert.Equal(target.DataI.ToArray(), expected.DataI.ToArray(), comparer);
        Assert.Equal(target.DataQ.ToArray(), expected.DataQ.ToArray(), comparer);
    }

    // TODO: get continuous envelope
    private static EnvelopeSample<double> GetEnvelope(double sampleRate)
    {
        var envelopeInfo = new EnvelopeInfo(0.9, sampleRate);
        var shape = new TrianglePulseShape();
        var width = 30e-9;
        var plateau = 45e-9;
        var envelope = new Envelope(shape, width, plateau);
        var result = WaveformSampler<double>.GetEnvelopeSample(envelopeInfo, envelope);
        Debug.Assert(result is not null);
        Debug.Assert(result.Plateau == 0);
        Debug.Assert(result.RightEdge.IsEmpty);
        return result;
    }

    private static PooledComplexArray<double> GetPlateau()
    {
        var array = new PooledComplexArray<double>(101, true);
        array.DataI.Fill(1);
        return array;
    }

    private static PooledComplexArray<double> GetBuffer(ComplexReadOnlySpan<double> envelope, int additionalLength)
    {
        var expected = new PooledComplexArray<double>(envelope.Length + additionalLength, true);
        var rng = new Random(42);
        for (var i = 0; i < expected.Length; i++)
        {
            expected.DataI[i] = rng.NextDouble();
            expected.DataQ[i] = rng.NextDouble();
        }

        return expected;
    }

    private static void MixAddWithDragSimple<T>(ComplexSpan<T> target, ComplexReadOnlySpan<T> source, IqPair<T> amplitude, IqPair<T> dragAmplitude, T dPhase)
        where T : unmanaged, IFloatingPointIeee754<T>
    {
        var length = source.Length;
        var sourceI = source.DataI;
        var sourceQ = source.DataQ;
        var targetI = target.DataI;
        var targetQ = target.DataQ;

        var carrier = amplitude;
        var dragCarrier = dragAmplitude;
        var phaser = IqPair<T>.FromPolarCoordinates(T.One, dPhase);
        for (var i = 0; i < length; i++)
        {
            var diff = i switch
            {
                0 => new IqPair<T>(sourceI[i + 1], sourceQ[i + 1]) - new IqPair<T>(sourceI[i], sourceQ[i]),
                _ when i == length - 1 => new IqPair<T>(sourceI[i], sourceQ[i]) - new IqPair<T>(sourceI[i - 1], sourceQ[i - 1]),
                _ => (new IqPair<T>(sourceI[i + 1], sourceQ[i + 1]) - new IqPair<T>(sourceI[i - 1], sourceQ[i - 1])) * T.CreateChecked(0.5),
            };
            var sourceIq = new IqPair<T>(sourceI[i], sourceQ[i]);
            var totalIq = sourceIq * carrier + diff * dragCarrier;
            targetI[i] += totalIq.I;
            targetQ[i] += totalIq.Q;
            carrier *= phaser;
            dragCarrier *= phaser;
        }
    }
}
