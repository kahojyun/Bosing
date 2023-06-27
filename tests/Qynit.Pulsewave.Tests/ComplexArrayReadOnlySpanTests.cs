namespace Qynit.Pulsewave.Tests;

public class ComplexArrayReadOnlySpanTests
{
    private const int Length = 100;

    private static PooledComplexArray<double> GetInitializedPooledComplexArray()
    {
        var length = Length;
        var clear = true;
        var pooledComplexArray = new PooledComplexArray<double>(length, clear);
        for (var i = 0; i < length; i++)
        {
            pooledComplexArray.DataI[i] = i;
            pooledComplexArray.DataQ[i] = -i;
        }

        return pooledComplexArray;
    }

    [Fact]
    public void Slice_Start_Equal()
    {
        // Arrange
        using var array = GetInitializedPooledComplexArray();
        var complexArrayReadOnlySpan = (ComplexArrayReadOnlySpan<double>)array;
        var start = 15;
        var remainingLength = array.Length - start;

        // Act
#pragma warning disable IDE0057 // Use range operator
        var result = complexArrayReadOnlySpan.Slice(
            start);
#pragma warning restore IDE0057 // Use range operator

        // Assert
        Assert.Equal(complexArrayReadOnlySpan.DataI[start..].ToArray(), result.DataI.ToArray());
        Assert.Equal(complexArrayReadOnlySpan.DataQ[start..].ToArray(), result.DataQ.ToArray());
        Assert.Equal(remainingLength, result.Length);
    }

    [Fact]
    public void Slice_StartLength_Equal()
    {
        // Arrange
        using var array = GetInitializedPooledComplexArray();
        var complexArrayReadOnlySpan = (ComplexArrayReadOnlySpan<double>)array;
        var start = 15;
        var length = 40;

        // Act
        var result = complexArrayReadOnlySpan.Slice(
            start,
            length);

        // Assert
        Assert.Equal(complexArrayReadOnlySpan.DataI.Slice(start, length).ToArray(), result.DataI.ToArray());
        Assert.Equal(complexArrayReadOnlySpan.DataQ.Slice(start, length).ToArray(), result.DataQ.ToArray());
        Assert.Equal(length, result.Length);
    }

    [Fact]
    public void CopyTo_EqualLength_Equal()
    {
        // Arrange
        var length = Length;
        var clear = true;

        using var source = GetInitializedPooledComplexArray();
        using var destination = new PooledComplexArray<double>(length, clear);


        // Act
        ((ComplexArrayReadOnlySpan<double>)source).CopyTo(
            destination);

        // Assert
        Assert.Equal(source.DataI.ToArray(), destination.DataI.ToArray());
        Assert.Equal(source.DataQ.ToArray(), destination.DataQ.ToArray());
    }

    [Fact]
    public void CopyTo_DestinationLonger_Equal()
    {
        // Arrange
        var length = Length + 1;
        var clear = true;

        using var source = GetInitializedPooledComplexArray();
        using var destination = new PooledComplexArray<double>(length, clear);


        // Act
        ((ComplexArrayReadOnlySpan<double>)source).CopyTo(
            destination);

        // Assert
        Assert.Equal(source.DataI.ToArray(), destination.DataI[..Length].ToArray());
        Assert.Equal(source.DataQ.ToArray(), destination.DataQ[..Length].ToArray());
        Assert.Equal(0, destination.DataI[^1]);
        Assert.Equal(0, destination.DataQ[^1]);
    }

    [Fact]
    public void CopyTo_DestinationShorter_Throw()
    {
        // Arrange
        var length = Length - 1;
        var clear = true;

        using var source = GetInitializedPooledComplexArray();
        using var destination = new PooledComplexArray<double>(length, clear);


        // Act
        Assert.Throws<ArgumentException>(() => ((ComplexArrayReadOnlySpan<double>)source).CopyTo(
                           destination));
    }
}
