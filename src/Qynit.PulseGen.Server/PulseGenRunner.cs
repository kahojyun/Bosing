using CommunityToolkit.Diagnostics;

namespace Qynit.PulseGen.Server;

public sealed class PulseGenRunner
{
    private readonly IList<ChannelInfo> _channelTable;
    private readonly IList<ShapeInfo> _shapeTable;
    private readonly IEnumerable<Instruction> _instructions;

    public PulseGenRunner(PulseGenRequest request)
    {
        _channelTable = request.ChannelTable;
        _shapeTable = request.ShapeTable;
        _instructions = request.Instructions;
    }

    public PulseGenResponse Run()
    {
        var phaseTrackingTransform = new PhaseTrackingTransform();
        foreach (var channel in _channelTable)
        {
            _ = phaseTrackingTransform.AddChannel(channel.BaseFrequency);
        }

        foreach (var instruction in _instructions)
        {
            switch (instruction)
            {
                case Play play:
                    Play(play);
                    break;
                case ShiftFrequency shiftFrequency:
                    ShiftFrequency(shiftFrequency);
                    break;
                case SetFrequency setFrequency:
                    SetFrequency(setFrequency);
                    break;
                case ShiftPhase shiftPhase:
                    ShiftPhase(shiftPhase);
                    break;
                case SetPhase setPhase:
                    SetPhase(setPhase);
                    break;
                case SwapPhase swapPhase:
                    SwapPhase(swapPhase);
                    break;
                default:
                    ThrowHelper.ThrowArgumentException($"Unknown instruction {instruction}");
                    break;
            }
        }

        var pulseLists = phaseTrackingTransform.Finish();
        var postProcessTransform = new PostProcessTransform();
        for (var i = 0; i < pulseLists.Count; i++)
        {
            var channel = _channelTable[i];
            var sourceId = postProcessTransform.AddSourceNode(pulseLists[i]);
            var delayId = postProcessTransform.AddDelay(channel.Delay);
            var terminalId = postProcessTransform.AddTerminalNode(out _);
            postProcessTransform.AddEdge(sourceId, delayId);
            postProcessTransform.AddEdge(delayId, terminalId);
        }
        var pulseLists2 = postProcessTransform.Finish();
        var result = pulseLists2.Zip(_channelTable).Select(x => WaveformUtils.SampleWaveform<double>(x.First, x.Second.SampleRate, 0, x.Second.Length, x.Second.AlignLevel));
        return new PulseGenResponse(result.ToList());

        void SwapPhase(SwapPhase swapPhase)
        {
            var channelId1 = swapPhase.ChannelId1;
            var channelId2 = swapPhase.ChannelId2;
            phaseTrackingTransform.SwapPhase(channelId1, channelId2, swapPhase.Time);
        }

        void ShiftPhase(ShiftPhase shiftPhase)
        {
            var channelId = shiftPhase.ChannelId;
            var deltaPhase = shiftPhase.Phase;
            phaseTrackingTransform.ShiftPhase(channelId, deltaPhase);
        }

        void SetPhase(SetPhase setPhase)
        {
            var channelId = setPhase.ChannelId;
            var phase = setPhase.Phase;
            phaseTrackingTransform.SetPhase(channelId, phase, 0);
        }

        void ShiftFrequency(ShiftFrequency shiftFrequency)
        {
            var channelId = shiftFrequency.ChannelId;
            var deltaFrequency = shiftFrequency.Frequency;
            var referenceTime = shiftFrequency.Time;
            phaseTrackingTransform.ShiftFrequency(channelId, deltaFrequency, referenceTime);
        }
        void SetFrequency(SetFrequency setFrequency)
        {
            var channelId = setFrequency.ChannelId;
            var frequency = setFrequency.Frequency;
            var referenceTime = setFrequency.Time;
            phaseTrackingTransform.SetFrequency(channelId, frequency, referenceTime);
        }

        void Play(Play play)
        {
            var channelId = play.ChannelId;
            var shapeId = play.ShapeId;
            var pulseShape = (shapeId == -1) ? null : _shapeTable[shapeId].GetPulseShape();
            var width = play.Width;
            var plateau = play.Plateau;
            var envelope = new Envelope(pulseShape, width, plateau);
            var frequency = play.FrequencyShift;
            var phase = play.PhaseShift;
            var amplitude = play.Amplitude;
            var dragCoefficient = play.DragCoefficient;
            var time = play.Time;
            phaseTrackingTransform.Play(channelId, envelope, frequency, phase, amplitude, dragCoefficient, time);
        }
    }
}
