using CommunityToolkit.Diagnostics;

namespace Qynit.Pulsewave;
public record Envelope
{
    public IPulseShape? Shape { get; }
    public double Width { get; }
    public double Plateau { get; }
    public Envelope(IPulseShape? shape, double width, double plateau)
    {
        Guard.IsGreaterThanOrEqualTo(width, 0);
        Guard.IsGreaterThanOrEqualTo(plateau, 0);
        if (shape is null)
        {
            Plateau = width + plateau;
            return;
        }
        Shape = (width > 0) ? shape : null;
        Width = width;
        Plateau = plateau;
    }
    public static Envelope Rectangle(double plateau)
    {
        return new(null, 0, plateau);
    }
}
