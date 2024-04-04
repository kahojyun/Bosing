using System.Numerics;

namespace Bosing.Tests;

public class PulseListTests
{
    private const double TimeTolerance = 1e-9 / 1e6;

    [Fact]
    public void Builder_Compressed()
    {
        // Arrange
        var builder = new PulseList.Builder(TimeTolerance);
        const double frequency1 = 120e6;
        const double frequency2 = 150e6;
        var envelope1 = Envelope.Rectangle(100e-9);
        var envelope2 = Envelope.Rectangle(150e-9);
        var bin1 = new PulseList.BinInfo(envelope1, frequency1, frequency2, 0);
        var bin2 = new PulseList.BinInfo(envelope1, frequency2, frequency2, 0);
        var bin3 = new PulseList.BinInfo(envelope2, frequency1, frequency2, 0.1e-9);
        builder.Add(bin1, new(0, new(new(0.51, 0.1), new(0.6, 0.2))));
        builder.Add(bin1, new(2e9, new(new(0.52, 0.2), new(0.5, 0.2))));
        builder.Add(bin1, new(-2e9, new(new(0.53, 0.3), new(0.4, 0.2))));
        builder.Add(bin2, new(0, new(new(0.54, 0.4), new(0.3, 0.2))));
        builder.Add(bin3, new(0, new(new(0.55, 0.5), new(0.2, 0.2))));
        builder.Add(bin3, new(0, new(new(0.56, 0.6), new(0.1, 0.2))));
        builder.Add(bin3, new(1e9, new(new(0.57, 0.7), new(0, 0.2))));

        // Act
        var pulses = builder.Build();

        // Assert
        Assert.Equal(3, pulses.Items.Count);
        Assert.Contains(bin1, pulses.Items);
        Assert.Contains(bin2, pulses.Items);
        Assert.Contains(bin3, pulses.Items);
        Assert.Equal(3, pulses.Items[bin1].Count);
        Assert.Single(pulses.Items[bin2]);
        Assert.Equal(2, pulses.Items[bin3].Count);
    }

    [Fact]
    public void Builder_Compressed_Equal()
    {
        // Arrange
        var builder = new PulseList.Builder(TimeTolerance);
        var envelope = Envelope.Rectangle(100e-9);
        const double frequency = 120e6;
        var bin = new PulseList.BinInfo(envelope, frequency, frequency, 0.1e-9);
        var amp1 = Complex.FromPolarCoordinates(0.5, Math.PI / 33);
        var amp2 = Complex.FromPolarCoordinates(0.6, Math.PI / 23);
        var amp3 = Complex.FromPolarCoordinates(0.7, Math.PI / 13);
        builder.Add(bin, new(2e-9, new(amp3, amp1)));
        builder.Add(bin, new(0, new(amp1, amp2)));
        builder.Add(bin, new(0, new(amp3, amp2)));
        builder.Add(bin, new(2e-9, new(amp2, amp3)));

        // Act
        var pulses = builder.Build();

        // Assert
        Assert.Single(pulses.Items);
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
        var builder = new PulseList.Builder(TimeTolerance);
        var envelope = Envelope.Rectangle(100e-9);
        const double frequency = 120e6;
        var bin = new PulseList.BinInfo(envelope, frequency, -frequency, 0.1e-9);
        var amp1 = Complex.FromPolarCoordinates(0.5, Math.PI / 33);
        var amp2 = Complex.FromPolarCoordinates(0.6, Math.PI / 23);
        var amp3 = Complex.FromPolarCoordinates(0.7, Math.PI / 13);
        builder.Add(bin, new(-2e-9, new(amp3, amp1)));
        builder.Add(bin, new(0, new(amp1, amp2)));
        builder.Add(bin, new(2e-9, new(amp1, amp2)));
        builder.Add(bin, new(0, new(amp3, amp2)));
        builder.Add(bin, new(-2e-9, new(amp2, amp3)));

        // Act
        var pulses = builder.Build();

        // Assert
        Assert.Single(pulses.Items);
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
        var builder = new PulseList.Builder(TimeTolerance);
        var envelope = Envelope.Rectangle(100e-9);
        const double frequency = 120e6;
        var bin = new PulseList.BinInfo(envelope, frequency, -frequency, 0.1e-9);
        var amp1 = Complex.FromPolarCoordinates(0.5, Math.PI / 33);
        var amp2 = Complex.FromPolarCoordinates(0.6, Math.PI / 23);
        var amp3 = Complex.FromPolarCoordinates(0.7, Math.PI / 13);
        builder.Add(bin, new(-2e-9, new(amp3, amp1)));
        builder.Add(bin, new(0, new(amp1, amp2)));
        builder.Add(bin, new(2e-9, new(amp1, amp2)));
        builder.Add(bin, new(0, new(amp3, amp2)));
        builder.Add(bin, new(-2e-9, new(amp2, amp3)));

        // Act
        var pulses1 = builder.Build();
        var pulses2 = builder.Build();

        // Assert
        Assert.Single(pulses1.Items);
        Assert.Empty(pulses2.Items);
    }

    [Fact]
    public void Merge()
    {
        // Arrange
        var builder1 = new PulseList.Builder(TimeTolerance);
        var builder2 = new PulseList.Builder(TimeTolerance);
        var envelope1 = Envelope.Rectangle(100e-9);
        var envelope2 = Envelope.Rectangle(150e-9);
        const double frequency = 120e6;
        var bin1 = new PulseList.BinInfo(envelope1, frequency, frequency, 0);
        var bin2 = new PulseList.BinInfo(envelope2, frequency, frequency, 0);
        var amp1 = Complex.FromPolarCoordinates(0.5, Math.PI / 33);
        var amp2 = Complex.FromPolarCoordinates(0.6, Math.PI / 23);
        var amp3 = Complex.FromPolarCoordinates(0.7, Math.PI / 13);
        builder1.Add(bin1, new(2e-9, new(amp3, amp1)));
        builder1.Add(bin1, new(0, new(amp1, amp2)));
        builder1.Add(bin1, new(0, new(amp3, amp2)));
        builder1.Add(bin1, new(2e-9, new(amp2, amp3)));
        builder1.Add(bin2, new(1e-9, new(amp2, amp3)));
        builder1.Add(bin2, new(0e-9, new(amp3, amp2)));
        builder1.Add(bin2, new(-1e-9, new(amp2, amp1)));

        builder2.Add(bin1, new(4e-9, new(amp3, amp1)));
        builder2.Add(bin1, new(1e-9, new(amp1, amp2)));
        builder2.Add(bin1, new(0, new(amp3, amp2)));
        builder2.Add(bin1, new(2e-9, new(amp2, amp3)));
        builder2.Add(bin2, new(1e-9, new(amp2, amp3)));
        builder2.Add(bin2, new(2e-9, new(amp2, amp2)));
        builder2.Add(bin2, new(-1e-9, new(amp2, amp1)));

        // Act
        var pulses1 = builder1.Build().TimeShifted(1e-9) * amp1;
        var pulses2 = builder2.Build().TimeShifted(-1e-9) * amp2;
        var pulses = PulseList.Sum(0, 0, pulses1, pulses2);

        // Assert
        Assert.Equal(4, pulses.Items.Count);
        Assert.Equal(0, pulses.TimeOffset);
        Assert.Equal(1, pulses.AmplitudeMultiplier);
    }
}
