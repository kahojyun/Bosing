using CommunityToolkit.Diagnostics;

namespace Bosing;
public readonly record struct Envelope
{
    public IPulseShape? Shape { get; }
    public double Width { get; }
    public double Plateau { get; }
    public double Duration => Plateau + Width;
    public bool IsRectangle => Shape is null;
    public bool IsZero => Width == 0 && Plateau == 0;
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
