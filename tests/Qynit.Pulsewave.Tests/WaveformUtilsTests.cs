using System.Numerics;

namespace Qynit.Pulsewave.Tests
{
    public class WaveformUtilsTests
    {
        [Fact]
        public void SampleWaveform_Normal_Equal()
        {
            // Arrange
            Waveform target = new Waveform(100, 1e9, 5e-9);
            IPulseShape shape = new TrianglePulseShape();
            double tStart = 5.1e-9;
            double width = 30e-9;
            double plateau = 40e-9;

            // Act
            WaveformUtils.SampleWaveform(
                target,
                shape,
                tStart,
                width,
                plateau);

            // Assert
            var index = new[] { 0, 10, 15, 25, 55, 56, 65, 70, 80 };
            var valueI = new[] { 0, 9.9 / 15, 14.9 / 15, 1, 1, 14.1 / 15, 5.1 / 15, 0.1 / 15, 0 };
            var valueQ = new double[index.Length];
            var resultI = index.Select(i => target.DataI[i]);
            var resultQ = index.Select(i => target.DataQ[i]);
            var comparer = new ToleranceComparer(1e-9);
            Assert.Equal(valueI, resultI, comparer);
            Assert.Equal(valueQ, resultQ, comparer);
        }

        [Fact]
        public void AddPulseToWaveform_StateUnderTest_ExpectedBehavior()
        {
            // Arrange
            Waveform target = new Waveform(200, 1e9, 0);
            Waveform pulse = new Waveform(100, 1e9, 5e-9);
            IPulseShape shape = new TrianglePulseShape();
            double tStart = 5.1e-9;
            double width = 30e-9;
            double plateau = 40e-9;
            WaveformUtils.SampleWaveform(pulse, shape, tStart, width, plateau);

            double amplitude = 0.5;
            double frequency = 100e6;
            double phase = Math.PI / 6;
            double referenceTime = -40e-9;
            double tShift = 10e-9;

            // Act
            WaveformUtils.AddPulseToWaveform(
                target,
                pulse,
                amplitude,
                frequency,
                phase,
                referenceTime,
                tShift);
            WaveformUtils.AddPulseToWaveform(
               target,
               pulse,
               amplitude,
               frequency,
               phase,
               referenceTime,
               tShift);

            var expectI = new double[target.Length];
            var expectQ = new double[target.Length];
            var pulseTStart = pulse.TStart + tShift;
            for (var i = 0; i < target.Length; i++)
            {
                var t = target.TimeAt(i);
                int pulseIndex = (int)Math.Round((t - pulseTStart) * pulse.SampleRate);
                if (pulseIndex < 0 || pulseIndex >= pulse.Length)
                {
                    continue;
                }
                var cPhase = phase + Math.Tau * frequency * (t - referenceTime);
                var c = Complex.FromPolarCoordinates(amplitude * 2, cPhase);
                var p = new Complex(pulse.DataI[pulseIndex], pulse.DataQ[pulseIndex]) * c;
                expectI[i] = p.Real;
                expectQ[i] = p.Imaginary;
            }

            // Assert
            var comparer = new ToleranceComparer(1e-9);
            Assert.Equal(target.DataI.ToArray(), expectI, comparer);
            Assert.Equal(target.DataQ.ToArray(), expectQ, comparer);
        }
    }
}
