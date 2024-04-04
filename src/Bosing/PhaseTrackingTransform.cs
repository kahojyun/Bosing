namespace Bosing;
public class PhaseTrackingTransform
{
    private readonly List<ChannelStatus> _channels = [];

    public int AddChannel(double baseFrequency, double timeTolerance)
    {
        var id = _channels.Count;
        _channels.Add(new ChannelStatus(timeTolerance) { BaseFrequency = baseFrequency });
        return id;
    }

    public void ShiftFrequency(int channelId, double deltaFrequency, double time)
    {
        _channels[channelId].ShiftFrequency(deltaFrequency, time);
    }

    public void SetFrequency(int channelId, double frequency, double time)
    {
        _channels[channelId].SetFrequency(frequency, time);
    }

    public void ShiftPhase(int channelId, double deltaPhase)
    {
        _channels[channelId].ShiftPhase(deltaPhase);
    }

    public void SetPhase(int channelId, double phase, double time)
    {
        _channels[channelId].SetPhase(phase, time);
    }

    public void SwapPhase(int channelId1, int channelId2, double time)
    {
        ChannelStatus.SwapPhase(_channels[channelId1], _channels[channelId2], time);
    }

    public void Play(int channelId, Envelope envelope, double frequency, double phase, double amplitude, double dragCoefficient, double time)
    {
        var globalFrequency = _channels[channelId].TotalFrequency;
        var totalPhase = _channels[channelId].Phase + phase;
        _channels[channelId].Builder.Add(envelope, globalFrequency, frequency, time, amplitude, totalPhase, dragCoefficient);
    }

    public List<PulseList> Finish()
    {
        var pulseLists = new List<PulseList>();
        foreach (var channel in _channels)
        {
            pulseLists.Add(channel.Builder.Build());
        }
        return pulseLists;
    }

    private class ChannelStatus(double timeTolerance)
    {
        public double BaseFrequency { get; init; }
        public double DeltaFrequency { get; private set; }
        public double Phase { get => _phase; private set => _phase = value % 1.0; }
        public double TotalFrequency => BaseFrequency + DeltaFrequency;
        private double _phase;
        public PulseList.Builder Builder { get; } = new PulseList.Builder(timeTolerance);

        public void ShiftFrequency(double deltaFrequency, double time)
        {
            var deltaPhase = -deltaFrequency * time;
            DeltaFrequency += deltaFrequency;
            Phase += deltaPhase;
        }

        public void SetFrequency(double frequency, double time)
        {
            var deltaFrequency = frequency - DeltaFrequency;
            var deltaPhase = -deltaFrequency * time;
            DeltaFrequency = frequency;
            Phase += deltaPhase;
        }

        public void ShiftPhase(double deltaPhase)
        {
            Phase += deltaPhase;
        }

        public void SetPhase(double phase, double time)
        {
            Phase = phase - DeltaFrequency * time;
        }

        public static void SwapPhase(ChannelStatus status1, ChannelStatus status2, double time)
        {
            var deltaFrequency12 = status1.TotalFrequency - status2.TotalFrequency;
            var phase1 = status1.Phase;
            var phase2 = status2.Phase;
            status1.Phase = phase2 - deltaFrequency12 * time;
            status2.Phase = phase1 + deltaFrequency12 * time;
        }
    }
}
