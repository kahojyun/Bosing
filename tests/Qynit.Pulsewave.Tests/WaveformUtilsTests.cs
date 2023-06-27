using System.Numerics;

namespace Qynit.Pulsewave.Tests;

public class WaveformUtilsTests
{
    [Fact]
    public void SampleWaveform_Double_Equal()
    {
        // Arrange
        var envelopeInfo = new EnvelopeInfo(0.9, 1e9);
        var shape = new TrianglePulseShape();
        var width = 30e-9;
        var plateau = 40e-9;

        // Act
        using var result = WaveformUtils.SampleWaveform<double>(
            envelopeInfo,
            shape,
            width,
            plateau);

        // Assert
        var index = new[] { 0, 9, 14, 24, 54, 55, 64, 69 };
        var valueI = new[] { 0.9 / 15, 9.9 / 15, 14.9 / 15, 1, 1, 14.1 / 15, 5.1 / 15, 0.1 / 15 };
        var valueQ = new double[index.Length];
        var resultI = index.Select(i => result.DataI[i]);
        var resultQ = index.Select(i => result.DataQ[i]);
        var comparer = new ToleranceComparer(1e-9);
        Assert.Equal(valueI, resultI, comparer);
        Assert.Equal(valueQ, resultQ, comparer);
        var expectedLength = (int)Math.Round((width + plateau) * envelopeInfo.SampleRate);
        Assert.Equal(expectedLength, result.Length);
    }

    [Fact]
    public void MixAddFrequency_Double_Equal()
    {
        // Arrange
        var envelopeInfo = new EnvelopeInfo(0.9, 1e9);
        var shape = new TrianglePulseShape();
        var width = 30e-9;
        var plateau = 40e-9;
        using var envelope = WaveformUtils.SampleWaveform<double>(
            envelopeInfo,
            shape,
            width,
            plateau);
        var additionalLength = 10;
        using var target = new PooledComplexArray<double>(envelope.Length + additionalLength, true);

        var amplitude = 0.5;
        var frequency = 100e6;
        var phase = Math.PI / 6;

        var cAmplitude = IqPair<double>.FromPolarCoordinates(amplitude, phase);
        var dt = 1 / envelopeInfo.SampleRate;
        var dPhase = Math.Tau * frequency * dt;

        // Act
        WaveformUtils.MixAddFrequency(target, envelope, cAmplitude, dPhase);
        WaveformUtils.MixAddFrequency(target, envelope, cAmplitude, dPhase);

        var expectI = new double[target.Length];
        var expectQ = new double[target.Length];
        for (var i = 0; i < envelope.Length; i++)
        {
            var t = i * dt;
            var cPhase = phase + Math.Tau * frequency * t;
            var c = Complex.FromPolarCoordinates(amplitude * 2, cPhase);
            var p = new Complex(envelope.DataI[i], envelope.DataQ[i]) * c;
            expectI[i] = p.Real;
            expectQ[i] = p.Imaginary;
        }

        // Assert
        var comparer = new ToleranceComparer(1e-9);
        Assert.Equal(target.DataI.ToArray(), expectI, comparer);
        Assert.Equal(target.DataQ.ToArray(), expectQ, comparer);
    }
}
