using System.Numerics;

namespace Bosing.Tests;
public class IqPairTests
{
    [Theory]
    [InlineData(1.2, Math.PI / 5)]
    [InlineData(2.2, Math.PI / 9)]
    public void FromPolarCoordinates_Double_SameAsComplex(double amplitude, double phase)
    {
        var iqPair = IqPair<double>.FromPolarCoordinates(amplitude, phase);
        var complex = Complex.FromPolarCoordinates(amplitude, phase);

        Assert.Equal(complex.Real, iqPair.I);
        Assert.Equal(complex.Imaginary, iqPair.Q);
    }

    [Theory]
    [InlineData(1.2)]
    [InlineData(Math.PI / 5)]
    public void FromValue_Double_SameAsComplex(double value)
    {
        var iqPair = (IqPair<double>)value;
        var complex = (Complex)value;

        Assert.Equal(complex.Real, iqPair.I);
        Assert.Equal(complex.Imaginary, iqPair.Q);
    }

    [Theory]
    [InlineData(1.2, Math.PI / 5)]
    [InlineData(2.2, Math.PI / 9)]
    public void Conjugate_Double_SameAsComplex(double i, double q)
    {
        var iqPair = new IqPair<double>(i, q);
        var complex = new Complex(i, q);

        var iqPair2 = IqPair<double>.Conjugate(iqPair);
        var complex2 = Complex.Conjugate(complex);

        Assert.Equal(complex2.Real, iqPair2.I);
        Assert.Equal(complex2.Imaginary, iqPair2.Q);
    }

    [Theory]
    [InlineData(1.2, Math.PI / 5)]
    [InlineData(2.2, Math.PI / 9)]
    public void Unary_Double_SameAsComplex(double i, double q)
    {
        var iqPair = new IqPair<double>(i, q);
        var complex = new Complex(i, q);

        var iqPair2 = +iqPair;
        var complex2 = +complex;

        Assert.Equal(complex2.Real, iqPair2.I);
        Assert.Equal(complex2.Imaginary, iqPair2.Q);

        var iqPair3 = -iqPair;
        var complex3 = -complex;

        Assert.Equal(complex3.Real, iqPair3.I);
        Assert.Equal(complex3.Imaginary, iqPair3.Q);
    }

    [Theory]
    [InlineData(1.2, Math.PI / 5, 3.5)]
    [InlineData(2.2, Math.PI / 9, 1.3)]
    public void ScalarArithmetic_Double_SameAsComplex(double i, double q, double value)
    {
        var iqPair = new IqPair<double>(i, q);
        var complex = new Complex(i, q);

        var iqPair2 = value + iqPair;
        var complex2 = value + complex;

        Assert.Equal(complex2.Real, iqPair2.I);
        Assert.Equal(complex2.Imaginary, iqPair2.Q);

        var iqPair3 = value - iqPair;
        var complex3 = value - complex;

        Assert.Equal(complex3.Real, iqPair3.I);
        Assert.Equal(complex3.Imaginary, iqPair3.Q);

        var iqPair4 = iqPair + value;
        var complex4 = complex + value;

        Assert.Equal(complex4.Real, iqPair4.I);
        Assert.Equal(complex4.Imaginary, iqPair4.Q);

        var iqPair5 = iqPair - value;
        var complex5 = complex - value;

        Assert.Equal(complex5.Real, iqPair5.I);
        Assert.Equal(complex5.Imaginary, iqPair5.Q);

        var iqPair6 = value * iqPair;
        var complex6 = value * complex;

        Assert.Equal(complex6.Real, iqPair6.I);
        Assert.Equal(complex6.Imaginary, iqPair6.Q);

        var iqPair7 = iqPair * value;
        var complex7 = complex * value;

        Assert.Equal(complex7.Real, iqPair7.I);
        Assert.Equal(complex7.Imaginary, iqPair7.Q);

        var iqPair8 = value / iqPair;
        var complex8 = value / complex;

        Assert.Equal(complex8.Real, iqPair8.I);
        Assert.Equal(complex8.Imaginary, iqPair8.Q);

        var iqPair9 = iqPair / value;
        var complex9 = complex / value;

        Assert.Equal(complex9.Real, iqPair9.I);
        Assert.Equal(complex9.Imaginary, iqPair9.Q);
    }

    [Theory]
    [InlineData(1.2, Math.PI / 5, 3.5, -Math.PI * 2.3)]
    [InlineData(2.2, Math.PI / 9, 1.3, -Math.PI * Math.E)]
    public void ComplexArithmetic_Double_SameAsComplex(double i, double q, double valueI, double valueQ)
    {
        var iqPair = new IqPair<double>(i, q);
        var complex = new Complex(i, q);
        var iqValue = new IqPair<double>(valueI, valueQ);
        var complexValue = new Complex(valueI, valueQ);

        var iqPair2 = iqPair + iqValue;
        var complex2 = complex + complexValue;

        Assert.Equal(complex2.Real, iqPair2.I);
        Assert.Equal(complex2.Imaginary, iqPair2.Q);

        var iqPair3 = iqPair - iqValue;
        var complex3 = complex - complexValue;

        Assert.Equal(complex3.Real, iqPair3.I);
        Assert.Equal(complex3.Imaginary, iqPair3.Q);

        var iqPair4 = iqPair * iqValue;
        var complex4 = complex * complexValue;

        Assert.Equal(complex4.Real, iqPair4.I);
        Assert.Equal(complex4.Imaginary, iqPair4.Q);

        var iqPair5 = iqPair / iqValue;
        var complex5 = complex / complexValue;

        Assert.Equal(complex5.Real, iqPair5.I);
        Assert.Equal(complex5.Imaginary, iqPair5.Q);

        var iqPair6 = iqValue + iqPair;
        var complex6 = complexValue + complex;

        Assert.Equal(complex6.Real, iqPair6.I);
        Assert.Equal(complex6.Imaginary, iqPair6.Q);

        var iqPair7 = iqValue - iqPair;
        var complex7 = complexValue - complex;

        Assert.Equal(complex7.Real, iqPair7.I);
        Assert.Equal(complex7.Imaginary, iqPair7.Q);

        var iqPair8 = iqValue * iqPair;
        var complex8 = complexValue * complex;

        Assert.Equal(complex8.Real, iqPair8.I);
        Assert.Equal(complex8.Imaginary, iqPair8.Q);

        var iqPair9 = iqValue / iqPair;
        var complex9 = complexValue / complex;

        Assert.Equal(complex9.Real, iqPair9.I);
        Assert.Equal(complex9.Imaginary, iqPair9.Q);
    }
}
