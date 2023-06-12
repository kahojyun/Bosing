using CommunityToolkit.Diagnostics;

namespace Qynit.Pulsewave;
public abstract class FilterNodeBase : IFilterNode
{
    public virtual double SampleRate
    {
        get
        {
            if (Outputs.Count == 0)
            {
                ThrowHelper.ThrowInvalidOperationException("No outputs");
            }
            var result = Outputs[0].SampleRate;
            if (Outputs.Any(o => o.SampleRate != result))
            {
                ThrowHelper.ThrowInvalidOperationException("Inconsistent sample rates");
            }
            return result;
        }
    }

    public virtual double TStart
    {
        get
        {
            if (Outputs.Count == 0)
            {
                ThrowHelper.ThrowInvalidOperationException("No outputs");
            }
            return Outputs.Max(x => x.TStart);
        }
    }

    public virtual double TEnd
    {
        get
        {
            if (Outputs.Count == 0)
            {
                ThrowHelper.ThrowInvalidOperationException("No outputs");
            }
            return Outputs.Min(x => x.TEnd);
        }
    }

    public string? Name { get; set; }
    public IList<IFilterNode> Outputs { get; } = new List<IFilterNode>();
    public IList<IFilterNode> Inputs { get; } = new List<IFilterNode>();

    public virtual void Initialize()
    {
        foreach (var output in Outputs)
        {
            output.Initialize();
        }
    }

    public virtual void Complete()
    {
        foreach (var output in Outputs)
        {
            output.Complete();
        }
    }

    public abstract void AddPulse(IPulseShape shape, double tStart, double width, double plateau, double amplitude, double frequency, double phase, double referenceTime);
    public abstract void AddWaveform(Waveform waveform, double tShift, double amplitude, double frequency, double phase, double referenceTime);
}
