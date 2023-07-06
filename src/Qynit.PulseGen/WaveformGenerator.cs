//using System.Numerics;

//using CommunityToolkit.Diagnostics;

//namespace Qynit.PulseGen;
//public class WaveformGenerator<T>
//    where T : unmanaged, IFloatingPointIeee754<T>
//{
//    private readonly Dictionary<Channel, ChannelInfo> _channelInfo = new();

//    public void AddChannel(Channel channel, double frequency, int length, double sampleRate, int alignLevel)

//    {
//        if (_channelInfo.ContainsKey(channel))
//        {
//            ThrowHelper.ThrowArgumentException($"Channel {channel} already exists");
//        }
//        _channelInfo.Add(channel, new(channel, frequency, length, sampleRate, alignLevel));
//    }

//    public Dictionary<Channel, PooledComplexArray<T>> Run(IEnumerable<Instruction> instructions)
//    {
//        var phaseTrackingTransform = new PhaseTrackingTransform();
//        var channelIds = new Dictionary<Channel, int>();
//        foreach (var channel in _channelInfo.Values)
//        {
//            var channelId = phaseTrackingTransform.AddChannel(channel.Frequency);
//            channelIds.Add(channel.Channel, channelId);
//        }

//        foreach (var instruction in instructions)
//        {
//            switch (instruction)
//            {
//                case Play play:
//                    Play(play);
//                    break;
//                case ShiftFrequency shiftFrequency:
//                    ShiftFrequency(shiftFrequency);
//                    break;
//                case SetFrequency setFrequency:
//                    SetFrequency(setFrequency);
//                    break;
//                case ShiftPhase shiftPhase:
//                    ShiftPhase(shiftPhase);
//                    break;
//                case SetPhase setPhase:
//                    SetPhase(setPhase);
//                    break;
//                case SwapPhase swapPhase:
//                    SwapPhase(swapPhase);
//                    break;
//                default:
//                    ThrowHelper.ThrowArgumentException($"Unknown instruction {instruction}");
//                    break;
//            }
//        }

//        var pulseLists = phaseTrackingTransform.Finish();
//        var result = new Dictionary<Channel, PooledComplexArray<T>>();
//        foreach (var channel in _channelInfo.Values)
//        {
//            var channelId = channelIds[channel.Channel];
//            var pulseList = pulseLists[channelId];
//            var waveform = WaveformUtils.SampleWaveform<T>(pulseList, channel.SampleRate, 0, channel.Length, channel.AlignLevel);
//            result.Add(channel.Channel, waveform);
//        }
//        return result;

//        void SwapPhase(SwapPhase swapPhase)
//        {
//            var channel1 = swapPhase.Channel1;
//            var channelId1 = channelIds[channel1];
//            var channel2 = swapPhase.Channel2;
//            var channelId2 = channelIds[channel2];
//            phaseTrackingTransform.SwapPhase(channelId1, channelId2, swapPhase.ReferenceTime);
//        }

//        void ShiftPhase(ShiftPhase shiftPhase)
//        {
//            var channel = shiftPhase.Channel;
//            var channelId = channelIds[channel];
//            var deltaPhase = shiftPhase.Phase / Math.Tau;
//            phaseTrackingTransform.ShiftPhase(channelId, deltaPhase);
//        }

//        void SetPhase(SetPhase setPhase)
//        {
//            var channel = setPhase.Channel;
//            var channelId = channelIds[channel];
//            var phase = setPhase.Phase / Math.Tau;
//            phaseTrackingTransform.SetPhase(channelId, phase, 0);
//        }

//        void ShiftFrequency(ShiftFrequency shiftFrequency)
//        {
//            var channel = shiftFrequency.Channel;
//            var channelId = channelIds[channel];
//            var deltaFrequency = shiftFrequency.Frequency;
//            var referenceTime = shiftFrequency.ReferenceTime;
//            phaseTrackingTransform.ShiftFrequency(channelId, deltaFrequency, referenceTime);
//        }
//        void SetFrequency(SetFrequency setFrequency)
//        {
//            var channel = setFrequency.Channel;
//            var channelId = channelIds[channel];
//            var referenceTime = setFrequency.ReferenceTime;
//            var frequency = setFrequency.Frequency;
//            phaseTrackingTransform.SetFrequency(channelId, frequency, referenceTime);
//        }

//        void Play(Play play)
//        {
//            var channel = play.Channel;
//            var channelId = channelIds[channel];
//            var pulseShape = play.PulseShape;
//            var width = play.Width;
//            var plateau = play.Plateau;
//            var envelope = new Envelope(pulseShape, width, plateau);
//            var frequency = play.Frequency;
//            var phase = play.Phase / Math.Tau;
//            var amplitude = play.Amplitude;
//            var dragCoefficient = play.DragCoefficient;
//            var time = play.TStart;
//            phaseTrackingTransform.Play(channelId, envelope, frequency, phase, amplitude, dragCoefficient, time);
//        }
//    }

//    private record ChannelInfo(Channel Channel, double Frequency, int Length, double SampleRate, int AlignLevel);
//}
