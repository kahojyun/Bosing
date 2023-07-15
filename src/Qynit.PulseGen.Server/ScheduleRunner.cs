using System.Diagnostics;

using CommunityToolkit.Diagnostics;

namespace Qynit.PulseGen.Server;

public sealed class ScheduleRunner
{
    private readonly ScheduleRequest _scheduleRequest;

    public ScheduleRunner(ScheduleRequest scheduleRequest)
    {
        Guard.IsNotNull(scheduleRequest.Schedule);
        Guard.IsNotNull(scheduleRequest.ChannelTable);
        Guard.IsNotNull(scheduleRequest.ShapeTable);
        _scheduleRequest = scheduleRequest;
    }

    public PulseGenResponse Run()
    {
        var scheduleDto = _scheduleRequest.Schedule;
        Debug.Assert(scheduleDto is not null);
        var schedule = scheduleDto.GetScheduleElement(_scheduleRequest);
        schedule.Measure(double.PositiveInfinity);
        Debug.Assert(schedule.DesiredDuration is not null);
        var duration = schedule.DesiredDuration.Value;
        schedule.Arrange(0, duration);

        var phaseTrackingTransform = new PhaseTrackingTransform();
        var channels = _scheduleRequest.ChannelTable;
        Debug.Assert(channels is not null);
        foreach (var channel in channels)
        {
            _ = phaseTrackingTransform.AddChannel(channel.BaseFrequency);
        }
        schedule.Render(0, phaseTrackingTransform);

        var pulseLists = phaseTrackingTransform.Finish();
        var postProcessTransform = new PostProcessTransform();
        for (var i = 0; i < pulseLists.Count; i++)
        {
            var channel = channels[i];
            var sourceId = postProcessTransform.AddSourceNode(pulseLists[i]);
            var delayId = postProcessTransform.AddDelay(channel.Delay);
            var terminalId = postProcessTransform.AddTerminalNode(out _);
            postProcessTransform.AddEdge(sourceId, delayId);
            postProcessTransform.AddEdge(delayId, terminalId);
        }
        var pulseLists2 = postProcessTransform.Finish();
        var result = pulseLists2.Zip(channels).Select(x => WaveformUtils.SampleWaveform<double>(x.First, x.Second.SampleRate, 0, x.Second.Length, x.Second.AlignLevel));
        return new PulseGenResponse(result.ToList());
    }
}
