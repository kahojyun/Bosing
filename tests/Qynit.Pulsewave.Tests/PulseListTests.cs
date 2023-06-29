namespace Qynit.Pulsewave.Tests;

public class PulseListTests
{
    [Fact]
    public void Builder_Compressed()
    {
        // Arrange
        var builder = new PulseList<double>.Builder();
        const double frequency1 = 120e6;
        const double frequency2 = 150e6;
        var envelope1 = Envelope.Rectangle(100e-9);
        var envelope2 = Envelope.Rectangle(150e-9);
        builder.Add(envelope1, frequency1, 0, Math.PI / 5, 0.53, 12e-9);
        builder.Add(envelope1, frequency1, 2e-9, Math.PI / 4, 0.55, 11e-9);
        builder.Add(envelope1, frequency1, -2e-9, Math.PI / 8, 0.57, 1e-9);
        builder.Add(envelope1, frequency2, 0, Math.PI / 55, 0.35, -1e-9);
        builder.Add(envelope2, frequency1, 0, Math.PI / 53, 0.25, 5e-9);
        builder.Add(envelope2, frequency1, 0, Math.PI / 52, 0.65, 3e-9);
        builder.Add(envelope2, frequency1, 1e-9, Math.PI / 51, 0.45, 5e-9);

        // Act
        var pulses = builder.Build();

        // Assert
        Assert.Equal(3, pulses.Items.Count);
        var bin1 = new PulseList<double>.BinInfo(envelope1, frequency1);
        Assert.Contains(bin1, pulses.Items);
        var bin2 = new PulseList<double>.BinInfo(envelope1, frequency2);
        Assert.Contains(bin2, pulses.Items);
        var bin3 = new PulseList<double>.BinInfo(envelope2, frequency1);
        Assert.Contains(bin3, pulses.Items);
        Assert.Equal(3, pulses.Items[bin1].Count);
        Assert.Equal(1, pulses.Items[bin2].Count);
        Assert.Equal(2, pulses.Items[bin3].Count);
    }

    [Fact]
    public void Builder_Compressed_Equal()
    {
        // Arrange
        var builder = new PulseList<double>.Builder();
        var envelope = Envelope.Rectangle(100e-9);
        const double frequency = 120e6;
        var amp1 = IqPair<double>.FromPolarCoordinates(0.5, Math.PI / 33);
        var amp2 = IqPair<double>.FromPolarCoordinates(0.6, Math.PI / 23);
        var amp3 = IqPair<double>.FromPolarCoordinates(0.7, Math.PI / 13);
        builder.Add(envelope, frequency, 2e-9, amp3, amp1);
        builder.Add(envelope, frequency, 0, amp1, amp2);
        builder.Add(envelope, frequency, 0, amp3, amp2);
        builder.Add(envelope, frequency, 2e-9, amp2, amp3);

        // Act
        var pulses = builder.Build();

        // Assert
        Assert.Equal(1, pulses.Items.Count);
        var bin = new PulseList<double>.BinInfo(envelope, frequency);
        Assert.Contains(bin, pulses.Items);
        Assert.Equal(2, pulses.Items[bin].Count);
        var list = pulses.Items[bin];
        Assert.Equal(0, list[0].Time);
        Assert.Equal(2e-9, list[1].Time);
        Assert.Equal(amp1 + amp3, list[0].Amplitude.Amplitude);
        Assert.Equal(amp2 + amp2, list[0].Amplitude.DragAmplitude);
        Assert.Equal(amp2 + amp3, list[1].Amplitude.Amplitude);
        Assert.Equal(amp1 + amp3, list[1].Amplitude.DragAmplitude);
    }

    [Fact]
    public void Builder_Sorted()
    {
        // Arrange
        var builder = new PulseList<double>.Builder();
        var envelope = Envelope.Rectangle(100e-9);
        const double frequency = 120e6;
        builder.Add(envelope, frequency, -2e-9, Math.PI / 77, 0.55, -55e-9);
        builder.Add(envelope, frequency, 0, Math.PI / 5, 0.5, 1e-9);
        builder.Add(envelope, frequency, 2e-9, Math.PI / 4, 0.4, 2e-9);
        builder.Add(envelope, frequency, 0, Math.PI / 3, 0.3, 1.5e-9);
        builder.Add(envelope, frequency, -2e-9, Math.PI / 2, 0.6, -1e-9);

        // Act
        var pulses = builder.Build();

        // Assert
        Assert.Equal(1, pulses.Items.Count);
        var bin = new PulseList<double>.BinInfo(envelope, frequency);
        Assert.Contains(bin, pulses.Items);
        Assert.Equal(3, pulses.Items[bin].Count);
        var list = pulses.Items[bin];
        Assert.Equal(-2e-9, list[0].Time);
        Assert.Equal(0, list[1].Time);
        Assert.Equal(2e-9, list[2].Time);
    }

    [Fact]
    public void Builder_SecondBuild_Cleared()
    {
        // Arrange
        var builder = new PulseList<double>.Builder();
        var envelope = Envelope.Rectangle(100e-9);
        const double frequency = 120e6;
        builder.Add(envelope, frequency, -2e-9, Math.PI / 77, 0.55, -55e-9);
        builder.Add(envelope, frequency, 0, Math.PI / 5, 0.5, 1e-9);
        builder.Add(envelope, frequency, 2e-9, Math.PI / 4, 0.4, 2e-9);
        builder.Add(envelope, frequency, 0, Math.PI / 3, 0.3, 1.5e-9);
        builder.Add(envelope, frequency, -2e-9, Math.PI / 2, 0.6, -1e-9);

        // Act
        var pulses1 = builder.Build();
        var pulses2 = builder.Build();

        // Assert
        Assert.Equal(1, pulses1.Items.Count);
        Assert.Equal(0, pulses2.Items.Count);
    }

    [Fact]
    public void Merge()
    {
        // Arrange
        var builder1 = new PulseList<double>.Builder();
        var builder2 = new PulseList<double>.Builder();
        var envelope1 = Envelope.Rectangle(100e-9);
        var envelope2 = Envelope.Rectangle(150e-9);
        const double frequency = 120e6;
        var amp1 = IqPair<double>.FromPolarCoordinates(0.5, Math.PI / 33);
        var amp2 = IqPair<double>.FromPolarCoordinates(0.6, Math.PI / 23);
        var amp3 = IqPair<double>.FromPolarCoordinates(0.7, Math.PI / 13);
        builder1.Add(envelope1, frequency, 2e-9, amp3, amp1);
        builder1.Add(envelope1, frequency, 0, amp1, amp2);
        builder1.Add(envelope1, frequency, 0, amp3, amp2);
        builder1.Add(envelope1, frequency, 2e-9, amp2, amp3);
        builder1.Add(envelope2, frequency, 1e-9, amp2, amp3);
        builder1.Add(envelope2, frequency, 0e-9, amp3, amp2);
        builder1.Add(envelope2, frequency, -1e-9, amp2, amp1);

        builder2.Add(envelope1, frequency, 4e-9, amp3, amp1);
        builder2.Add(envelope1, frequency, 1e-9, amp1, amp2);
        builder2.Add(envelope1, frequency, 0, amp3, amp2);
        builder2.Add(envelope1, frequency, 2e-9, amp2, amp3);
        builder2.Add(envelope2, frequency, 1e-9, amp2, amp3);
        builder2.Add(envelope2, frequency, 2e-9, amp2, amp2);
        builder2.Add(envelope2, frequency, -1e-9, amp2, amp1);

        // Act
        var pulses1 = builder1.Build().TimeShifted(1e-9) * amp1;
        var pulses2 = builder2.Build().TimeShifted(-1e-9) * amp2;
        var pulses = pulses1 + pulses2;

        // Assert
        Assert.Equal(2, pulses.Items.Count);
        Assert.Equal(0, pulses.TimeOffset);
        Assert.Equal(1, pulses.AmplitudeMultiplier);
        var bin1 = new PulseList<double>.BinInfo(envelope1, frequency);
        var bin2 = new PulseList<double>.BinInfo(envelope2, frequency);
        Assert.Contains(bin1, pulses.Items);
        Assert.Contains(bin2, pulses.Items);
        var list1 = pulses.Items[bin1];
        var list2 = pulses.Items[bin2];
        Assert.Equal(4, list1.Count);
        Assert.Equal(4, list2.Count);
        var tolerance = 1e-9;
        Assert.Equal(-1e-9, list1[0].Time, tolerance);
        Assert.Equal(0, list1[1].Time, tolerance);
        Assert.Equal(1e-9, list1[2].Time, tolerance);
        Assert.Equal(3e-9, list1[3].Time, tolerance);
        Assert.Equal(-2e-9, list2[0].Time, tolerance);
        Assert.Equal(0, list2[1].Time, tolerance);
        Assert.Equal(1e-9, list2[2].Time, tolerance);
        Assert.Equal(2e-9, list2[3].Time, tolerance);
        var pulseAmplitude = new PulseList<double>.PulseAmplitude((amp3 + amp2) * amp1 + amp3 * amp2, (amp1 + amp3) * amp1 + amp1 * amp2);
        Assert.Equal(pulseAmplitude, list1[3].Amplitude);
    }
}
