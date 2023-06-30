using System.Numerics;

using CommunityToolkit.Diagnostics;

namespace Qynit.Pulsewave;
public class WaveformGenerator<T>
    where T : unmanaged, IFloatingPointIeee754<T>
{
    private readonly Dictionary<Channel, ChannelContext> _channelContexts = new();
    private readonly PostProcessGraph<T> _graph = new();

    public void AddChannel(Channel channel, double frequency, int length, double sampleRate, int alignLevel)

    {
        if (_channelContexts.ContainsKey(channel))
        {
            ThrowHelper.ThrowArgumentException($"Channel {channel} already exists");
        }
        var builder = _graph.AddSourceNode(channel.Name);
        var context = new ChannelContext
        {
            Channel = channel,
            Frequency = frequency,
            Builder = builder,
            Length = length,
            SampleRate = sampleRate,
            AlignLevel = alignLevel,
        };
        _channelContexts.Add(channel, context);
    }

    public void Run(IEnumerable<Instruction> instructions)
    {
        foreach (var instruction in instructions)
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
        _graph.Run();
        foreach (var context in _channelContexts.Values)
        {
            var name = context.Channel.Name;
            var pulseList = _graph.GetPulseList(name);
            var waveform = WaveformUtils.SampleWaveform(pulseList, context.SampleRate, context.Length, context.AlignLevel);
            context.Waveform = waveform;
        }
    }

    public PooledComplexArray<T> TakeWaveform(Channel channel)
    {
        var context = _channelContexts[channel];
        var waveform = context.Waveform;
        if (waveform is null)
        {
            ThrowHelper.ThrowArgumentException($"Channel {channel} has not been run");
        }
        context.Waveform = null;
        return waveform;
    }

    private void SwapPhase(SwapPhase swapPhase)
    {
        var channel1 = swapPhase.Channel1;
        var context1 = _channelContexts[channel1];
        var channel2 = swapPhase.Channel2;
        var context2 = _channelContexts[channel2];
        (context1.Phase, context2.Phase) = (context2.Phase, context1.Phase);
    }

    private void ShiftPhase(ShiftPhase shiftPhase)
    {
        var channel = shiftPhase.Channel;
        var context = _channelContexts[channel];
        context.Phase += shiftPhase.Phase;
    }

    private void SetPhase(SetPhase setPhase)
    {
        var channel = setPhase.Channel;
        var context = _channelContexts[channel];
        context.Phase = setPhase.Phase;
    }

    private void ShiftFrequency(ShiftFrequency shiftFrequency)
    {
        var channel = shiftFrequency.Channel;
        var context = _channelContexts[channel];
        var referenceTime = shiftFrequency.ReferenceTime;
        var deltaFrequency = shiftFrequency.Frequency;
        context.ShiftFrequency(deltaFrequency, referenceTime);
    }
    private void SetFrequency(SetFrequency setFrequency)
    {
        var channel = setFrequency.Channel;
        var context = _channelContexts[channel];
        var referenceTime = setFrequency.ReferenceTime;
        var frequency = setFrequency.Frequency;
        context.SetFrequency(frequency, referenceTime);
    }

    private void Play(Play play)
    {
        if (play.Amplitude == 0)
        {
            return;
        }
        var channel = play.Channel;
        var context = _channelContexts[channel];
        var builder = context.Builder;
        var pulseShape = play.PulseShape;
        var tStart = play.TStart;
        var width = play.Width;
        var plateau = play.Plateau;
        var amplitude = play.Amplitude;
        var dragCoefficient = play.DragCoefficient;
        var frameFrequency = context.Frequency + context.FrequencyShift;
        var totalFrequency = play.Frequency + frameFrequency;
        var phase = (play.Phase + context.Phase + Math.Tau * frameFrequency * tStart) % Math.Tau;
        var envelope = new Envelope(pulseShape, width, plateau);
        builder.Add(envelope, totalFrequency, tStart, T.CreateChecked(amplitude), T.CreateChecked(phase), T.CreateChecked(dragCoefficient));
    }

    private class ChannelContext
    {
        public required Channel Channel { get; init; }
        public required PulseList<T>.Builder Builder { get; init; }
        public required int Length { get; init; }
        public required double SampleRate { get; init; }
        public double Frequency { get; init; }
        public double FrequencyShift { get; set; }
        public double Phase { get; set; }
        public PooledComplexArray<T>? Waveform { get; set; }
        public int AlignLevel { get; init; }

        public void ShiftFrequency(double deltaFrequency, double referenceTime)
        {
            var deltaPhase = -Math.Tau * deltaFrequency * referenceTime;
            FrequencyShift += deltaFrequency;
            Phase += deltaPhase;
        }

        public void SetFrequency(double frequency, double referenceTime)
        {
            var deltaFrequency = frequency - FrequencyShift;
            ShiftFrequency(deltaFrequency, referenceTime);
        }
    }
}
