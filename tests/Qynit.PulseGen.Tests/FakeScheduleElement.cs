namespace Qynit.PulseGen.Tests;

public class FakeScheduleElement : ScheduleElement
{
    public override IReadOnlySet<int> Channels { get; }

    public bool Flexible { get; init; }
    public Action<double>? RenderCallback { get; init; }
    public double MeasureResult { get; }
    public double ArrangeResult { get; }

    public FakeScheduleElement() : this(0, 0)
    {
    }

    public FakeScheduleElement(double measureResult, double arrangeResult) : this(measureResult, arrangeResult, Enumerable.Empty<int>())
    {
    }

    public FakeScheduleElement(IEnumerable<int> channels) : this(0, 0, channels)
    {
    }

    public FakeScheduleElement(double measureResult, double arrangeResult, IEnumerable<int> channels)
    {
        MeasureResult = measureResult;
        ArrangeResult = arrangeResult;
        Channels = channels.ToHashSet();
    }

    protected override double ArrangeOverride(double time, double finalDuration)
    {
        return Flexible ? finalDuration : ArrangeResult;
    }

    protected override double MeasureOverride(double maxDuration)
    {
        return MeasureResult;
    }

    protected override void RenderOverride(double time, PhaseTrackingTransform phaseTrackingTransform)
    {
        RenderCallback?.Invoke(time);
    }
}
