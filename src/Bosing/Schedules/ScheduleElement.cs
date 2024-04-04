using System.Diagnostics;

using CommunityToolkit.Diagnostics;

namespace Bosing.Schedules;
public abstract class ScheduleElement
{
    private double _maxDuration = double.PositiveInfinity;
    private double _minDuration;
    private double? _duration;

    public BosingOptions? BosingOptions { get; set; }
    public ScheduleElement? Parent { get; internal set; }
    public Thickness Margin { get; set; }
    public Alignment Alignment { get; set; }
    public bool IsVisible { get; set; } = true;
    public double? Duration
    {
        get => _duration;
        set
        {
            if (value is not null)
            {
                Guard.IsGreaterThanOrEqualTo(value.Value, 0);
            }
            _duration = value;
        }
    }
    public double MaxDuration
    {
        get => _maxDuration;
        set
        {
            Guard.IsGreaterThanOrEqualTo(value, 0);
            _maxDuration = value;
        }
    }
    public double MinDuration
    {
        get => _minDuration;
        set
        {
            Guard.IsGreaterThanOrEqualTo(value, 0);
            _minDuration = value;
        }
    }
    public double? DesiredDuration { get; private set; }
    public double? ActualDuration { get; private set; }
    public double? ActualTime { get; private set; }
    public abstract IReadOnlySet<int> Channels { get; }
    internal bool IsMeasuring { get; private set; }
    internal double? UnclippedDesiredDuration { get; private set; }
    public void Measure(double availableDuration)
    {
        Debug.Assert(availableDuration >= 0 || double.IsPositiveInfinity(availableDuration));
        if (IsMeasuring)
        {
            ThrowHelper.ThrowInvalidOperationException("Already measuring");
        }
        IsMeasuring = true;
        var margin = Margin.Total;
        Debug.Assert(double.IsFinite(margin));
        var maxDuration = MathUtils.Clamp(Duration ?? double.PositiveInfinity, MinDuration, MaxDuration);
        var minDuration = MathUtils.Clamp(Duration ?? 0, MinDuration, MaxDuration);
        var innerDuration = Math.Max(availableDuration - margin, 0);
        var clampedDuration = MathUtils.Clamp(innerDuration, minDuration, maxDuration);
        var measuredDuration = MeasureOverride(clampedDuration);
        Debug.Assert(double.IsFinite(measuredDuration));
        UnclippedDesiredDuration = Math.Max(measuredDuration + margin, 0);
        DesiredDuration = MathUtils.Clamp(MathUtils.Clamp(measuredDuration, minDuration, maxDuration) + margin, 0, availableDuration);
        IsMeasuring = false;
    }
    protected abstract double MeasureOverride(double maxDuration);
    public void Arrange(double time, double finalDuration)
    {
        Debug.Assert(double.IsFinite(time));
        Debug.Assert(double.IsFinite(finalDuration) && finalDuration >= 0);
        if (DesiredDuration is null || UnclippedDesiredDuration is null)
        {
            ThrowHelper.ThrowInvalidOperationException("Not measured");
        }
        var options = BosingOptions ?? BosingOptions.Default;
        if (finalDuration < UnclippedDesiredDuration - options.TimeTolerance && !options.AllowOversize)
        {
            ThrowHelper.ThrowInvalidOperationException("Final duration is less than unclipped desired duration");
        }
        var innerTime = time + Margin.Start;
        Debug.Assert(double.IsFinite(innerTime));
        var maxDuration = MathUtils.Clamp(Duration ?? double.PositiveInfinity, MinDuration, MaxDuration);
        var minDuration = MathUtils.Clamp(Duration ?? 0, MinDuration, MaxDuration);
        var margin = Margin.Total;
        var innerDuration = Math.Max(finalDuration - margin, 0);
        var clampedDuration = MathUtils.Clamp(innerDuration, minDuration, maxDuration);
        if (clampedDuration + margin < UnclippedDesiredDuration - options.TimeTolerance && !options.AllowOversize)
        {
            ThrowHelper.ThrowInvalidOperationException("User specified duration is less than unclipped desired duration");
        }
        var actualDuration = ArrangeOverride(innerTime, clampedDuration);
        Debug.Assert(double.IsFinite(actualDuration));
        ActualDuration = actualDuration;
        ActualTime = innerTime;
    }
    protected abstract double ArrangeOverride(double time, double finalDuration);
    public void Render(double time, PhaseTrackingTransform phaseTrackingTransform)
    {
        if (ActualTime is null || ActualDuration is null)
        {
            ThrowHelper.ThrowInvalidOperationException("Not arranged");
        }
        if (!IsVisible)
        {
            return;
        }
        var innerTime = time + ActualTime.Value;
        Debug.Assert(double.IsFinite(innerTime));
        RenderOverride(innerTime, phaseTrackingTransform);
    }
    protected abstract void RenderOverride(double time, PhaseTrackingTransform phaseTrackingTransform);
}
