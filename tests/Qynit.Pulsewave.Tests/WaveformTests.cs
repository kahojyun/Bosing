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
        public void ShiftTime_NotAlign_Equal()
        {
            var length = 100;
            var sampleRate = 1e9;
            var t0 = 0.0;
            // Arrange
            var waveform = new Waveform(length, sampleRate, t0);
            double deltaT = 3.5e-9;
            bool alignToDt = false;

            // Act
            waveform.ShiftTime(
                deltaT,
                alignToDt);

            // Assert
            Assert.Equal(deltaT, waveform.TStart);
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
            Assert.Equal(t0 + index / sampleRate, result);
        }
    }
}
