using System.Diagnostics;

using CommunityToolkit.Diagnostics;

using Bosing.Aot.Models;

namespace Bosing.Aot;

public sealed class ScheduleRunner
{
    private readonly ScheduleRequest _scheduleRequest;
    private readonly BosingOptions _options = new();

    public ScheduleRunner(ScheduleRequest scheduleRequest)
    {
        Guard.IsNotNull(scheduleRequest.Schedule);
        Guard.IsNotNull(scheduleRequest.ChannelTable);
        Guard.IsNotNull(scheduleRequest.ShapeTable);
        _scheduleRequest = scheduleRequest;
    }

    public List<PooledComplexArray<float>> Run()
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
            _ = phaseTrackingTransform.AddChannel(channel.BaseFrequency, _options.TimeTolerance);
        }
        schedule.Render(0, phaseTrackingTransform);

        var pulseLists = phaseTrackingTransform.Finish();
        var postProcessTransform = new PostProcessTransform(_options);
        for (var i = 0; i < pulseLists.Count; i++)
        {
            var channel = channels[i];
            var sourceId = postProcessTransform.AddSourceNode(pulseLists[i]);
            var filterId = postProcessTransform.AddFilter(new(channel.BiquadChain.Select(x => x.GetBiquad()), channel.FirCoefficients));
            var delayId = postProcessTransform.AddDelay(channel.Delay);
            var terminalId = postProcessTransform.AddTerminalNode(out _);
            postProcessTransform.AddEdge(sourceId, filterId);
            postProcessTransform.AddEdge(filterId, delayId);
            postProcessTransform.AddEdge(delayId, terminalId);
        }
        var pulseLists2 = postProcessTransform.Finish();
        var result = pulseLists2.Zip(channels).Select(x =>
        {
            using var waveform = WaveformUtils.SampleWaveform<double>(x.First, x.Second.SampleRate, 0, x.Second.Length, x.Second.AlignLevel);
            if (x.Second.IqCalibration is { A: var a, B: var b, C: var c, D: var d, IOffset: var iOffset, QOffset: var qOffset })
            {
                WaveformUtils.IqTransform(waveform, a, b, c, d, iOffset, qOffset);
            }
            var floatArray = new PooledComplexArray<float>(waveform.Length, false);
            WaveformUtils.ConvertDoubleToFloat(floatArray, waveform);
            return floatArray;
        });
        return result.ToList();
    }
}
