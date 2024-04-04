using System.Numerics;

using Bosing.Schedules;

namespace Bosing.Tests;

// TODO better test
public class IntegrationTests
{
    [Fact]
    public void RunDouble()
    {
        Run<double>();
    }

    [Fact]
    public void RunSingle()
    {
        Run<float>();
    }

    private static void Run<T>() where T : unmanaged, IFloatingPointIeee754<T>
    {
        var phaseTrackingTransform = new PhaseTrackingTransform();
        var ch1 = phaseTrackingTransform.AddChannel(100e6, 0);
        var ch2 = phaseTrackingTransform.AddChannel(250e6, 0);
        var shape = new HannPulseShape();
        var stack = new StackSchedule
        {
            new PlayElement(ch1, new(shape, 30e-9, 100e-9), 0, 0, 0.5, 2e-9),
            new PlayElement(ch2, new(shape, 30e-9, 50e-9), 0, 0, 0.6, 2e-9),
            new ShiftPhaseElement(ch1, 0.25),
            new ShiftPhaseElement(ch2, -0.25),
            new BarrierElement(ch1, ch2) { Margin = new(15e-9) }
        };

        var stack2 = new StackSchedule
        {
            new PlayElement(ch1, new(shape, 30e-9, 0), 0, 0, 0.5, 2e-9) { Margin = new(15e-9) },
            new PlayElement(ch1, new(shape, 30e-9, 0), 0, 0, 0.5, 2e-9) { Margin = new(15e-9) },
            new PlayElement(ch1, new(shape, 30e-9, 0), 0, 0, 0.5, 2e-9) { Margin = new(15e-9) }
        };
        stack.Add(stack2);

        stack.Add(new BarrierElement(ch1, ch2) { Margin = new(15e-9) });

        var grid = new GridSchedule() { Duration = 500e-9 };
        grid.AddColumn(GridLength.Absolute(90e-9));
        grid.AddColumn(GridLength.Star(1));
        grid.AddColumn(GridLength.Absolute(90e-9));
        grid.Add(new PlayElement(ch1, new(shape, 30e-9, 200e-9), 0, 0, 0.5, 2e-9) { Alignment = Bosing.Schedules.Alignment.Center }, 1, 1);
        grid.Add(new PlayElement(ch2, new(shape, 30e-9, 50e-9), -250e6, 0, 0.6, 2e-9) { Alignment = Bosing.Schedules.Alignment.Stretch, FlexiblePlateau = true }, 0, 3);
        stack.Add(grid);

        stack.Add(new ShiftFrequencyElement(ch1, -100e6));
        stack.Add(new ShiftFrequencyElement(ch2, -250e6));
        stack.Add(new BarrierElement(ch1, ch2) { Margin = new(15e-9) });

        var abs = new AbsoluteSchedule
        {
            { new PlayElement(ch1, new(null, 200e-9, 0), 0, 0, 0.5, 2e-9), 10e-9 },
            { new RepeatElement(new PlayElement(ch2, new(null, 100e-9, 0), 0, 0, 0.6, 2e-9), 2) { Spacing = 10e-9 }, 210e-9 }
        };
        stack.Add(abs);

        stack.Add(new SetFrequencyElement(ch1, 0));
        stack.Add(new SetFrequencyElement(ch2, 0));
        stack.Add(new BarrierElement(ch1, ch2) { Margin = new(15e-9) });
        stack.Add(new PlayElement(ch1, new(shape, 200e-9, 0), 0, 0, 0.5, 2e-9));
        stack.Add(new PlayElement(ch2, new(shape, 100e-9, 0), 0, 0, 0.6, 2e-9));
        var main = new GridSchedule
        {
            stack
        };
        main.Measure(49.9e-6);
        main.Arrange(0, 49.9e-6);
        main.Render(0, phaseTrackingTransform);
        var pulseLists = phaseTrackingTransform.Finish();

        var sampleRate = 2e9;
        var n = 100000;
        var alignLevel = -4;
        var waveforms = pulseLists.Select(x => WaveformUtils.SampleWaveform<T>(x, sampleRate, 0, n, alignLevel)).ToList();
    }
}
