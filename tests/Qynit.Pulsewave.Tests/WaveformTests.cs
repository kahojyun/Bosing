namespace Qynit.Pulsewave.Tests
{
    public class WaveformTests
    {
        [Fact]
        public void Data_Disposed_ShouldThrow()
        {
            var length = 100;
            var sampleRate = 1e9;
            var t0 = 0.0;
            // Arrange
            var waveform = new Waveform(length, sampleRate, t0);

            // Act
            waveform.Dispose();

            // Assert
            Assert.Throws<ObjectDisposedException>(() => _ = waveform.DataI);
            Assert.Throws<ObjectDisposedException>(() => _ = waveform.DataQ);
        }

        [Fact]
        public void Data_New_AlignToDt()
        {
            var length = 100;
            var sampleRate = 1e9;
            var t0 = 0.1e-9;
            // Arrange
            var waveform = new Waveform(length, sampleRate, t0);

            // Assert
            var tolerance = 1e-6 / sampleRate;
            Assert.Equal(0.0, waveform.TStart, tolerance);
        }

        [Fact]
        public void Data_New_AllZero()
        {
            var length = 100;
            var sampleRate = 1e9;
            var t0 = 0.0;
            // Arrange
            var waveform = new Waveform(length, sampleRate, t0);

            // Assert
            var allZero = true;
            foreach (var item in waveform.DataI)
            {
                if (item != 0)
                {
                    allZero = false;
                    break;
                }
            }
            foreach (var item in waveform.DataQ)
            {
                if (item != 0)
                {
                    allZero = false;
                    break;
                }
            }
            Assert.True(allZero);
        }

        [Fact]
        public void Data_New_Props()
        {
            var length = 100;
            var sampleRate = 1e9;
            var t0 = 0;
            // Arrange
            var waveform = new Waveform(length, sampleRate, t0);

            // Assert
            var tolerance = 1e-6 / sampleRate;
            Assert.Equal(0.0, waveform.TStart, tolerance);
            Assert.Equal(length, waveform.Length);
            Assert.Equal(length, waveform.DataI.Length);
            Assert.Equal(length, waveform.DataQ.Length);
            Assert.Equal(t0 + length / sampleRate, waveform.TEnd, tolerance);
            Assert.Equal(sampleRate, waveform.SampleRate);
            Assert.Equal(1 / sampleRate, waveform.Dt, tolerance);
        }

        [Fact]
        public void Data_Copy_Equals()
        {
            var length = 100;
            var sampleRate = 1e9;
            var t0 = 0.0;
            // Arrange
            var waveform = new Waveform(length, sampleRate, t0);
            waveform.DataI.Fill(1);
            waveform.DataQ[10..50].Fill(2);

            var copy = new Waveform(waveform);
            var copy2 = waveform.Copy();

            // Assert
            Assert.Equal(waveform.DataI.ToArray(), copy.DataI.ToArray());
            Assert.Equal(waveform.DataQ.ToArray(), copy.DataQ.ToArray());
            Assert.Equal(waveform.DataI.ToArray(), copy2.DataI.ToArray());
            Assert.Equal(waveform.DataQ.ToArray(), copy2.DataQ.ToArray());
        }

        [Fact]
        public void ShiftTime_Normal_Equal()
        {
            var length = 100;
            var sampleRate = 1e9;
            var t0 = 0.0;
            // Arrange
            var waveform = new Waveform(length, sampleRate, t0);
            double deltaT = 3.6e-9;

            // Act
            waveform.ShiftTime(
                deltaT);

            // Assert
            var tolerance = 1e-6 / sampleRate;
            Assert.Equal(4e-9, waveform.TStart, tolerance);
        }

        [Fact]
        public void TimeAt_Normal_Equal()
        {
            var length = 100;
            var sampleRate = 1e9;
            var t0 = 0.0;
            // Arrange
            var waveform = new Waveform(length, sampleRate, t0);
            int index = 30;

            // Act
            var result = waveform.TimeAt(
                index);

            // Assert
            var tolerance = 1e-6 / sampleRate;
            Assert.Equal(t0 + index / sampleRate, result, tolerance);
        }
    }
}
