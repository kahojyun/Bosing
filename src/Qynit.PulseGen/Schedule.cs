﻿namespace Qynit.PulseGen;
public abstract class Schedule : ScheduleElement
{
    public override IReadOnlySet<int> Channels => _channels ??= Children.SelectMany(e => e.Channels).ToHashSet();
    private HashSet<int>? _channels;
    protected List<ScheduleElement> Children { get; } = new();


    protected override void RenderOverride(double time, PhaseTrackingTransform phaseTrackingTransform)
    {
        foreach (var element in Children)
        {
            element.Render(time, phaseTrackingTransform);
        }
    }
}
