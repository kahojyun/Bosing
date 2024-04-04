namespace Bosing.Tests;

public class PooledComplexArrayTests
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
    public void Ctor_Clear_AllZero()
    {
        // Arrange
        var length = Length;
        var clear = true;

        // Act
        using var pooledComplexArray = new PooledComplexArray<double>(length, clear);

        // Assert
        Assert.Equal(length, pooledComplexArray.Length);
        Assert.True(pooledComplexArray.DataI.ToArray().All(x => x == 0));
        Assert.True(pooledComplexArray.DataQ.ToArray().All(x => x == 0));
        Assert.False(pooledComplexArray.IsEmpty);
    }

    [Fact]
    public void Ctor_ZeroLength_Empty()
    {
        // Arrange
        var length = 0;
        var clear = true;

        // Act
        using var pooledComplexArray = new PooledComplexArray<double>(length, clear);

        // Assert
        Assert.Equal(length, pooledComplexArray.Length);
        Assert.True(pooledComplexArray.IsEmpty);
        Assert.True(pooledComplexArray.DataI.IsEmpty);
        Assert.True(pooledComplexArray.DataQ.IsEmpty);
    }

    [Fact]
    public void CopyCtor_Normal_Equal()
    {
        // Arrange
        using var pooledComplexArray = GetInitializedPooledComplexArray();

        // Act
        var result = new PooledComplexArray<double>(pooledComplexArray);

        // Assert
        Assert.Equal(pooledComplexArray.DataI.ToArray(), result.DataI.ToArray());
        Assert.Equal(pooledComplexArray.DataQ.ToArray(), result.DataQ.ToArray());
        Assert.Equal(pooledComplexArray.Length, result.Length);
    }

    [Fact]
    public void Copy_Normal_Equal()
    {
        // Arrange
        using var pooledComplexArray = GetInitializedPooledComplexArray();

        // Act
        var result = pooledComplexArray.Copy();

        // Assert
        Assert.Equal(pooledComplexArray.DataI.ToArray(), result.DataI.ToArray());
        Assert.Equal(pooledComplexArray.DataQ.ToArray(), result.DataQ.ToArray());
        Assert.Equal(pooledComplexArray.Length, result.Length);
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
        source.CopyTo(
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
        source.CopyTo(
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
        Assert.Throws<ArgumentException>(() => source.CopyTo(
                           destination));
    }

    [Fact]
    public void Slice_Start_Equal()
    {
        // Arrange
        using var pooledComplexArray = GetInitializedPooledComplexArray();
        var start = 15;
        var remainingLength = pooledComplexArray.Length - start;

        // Act
        var result = pooledComplexArray.Slice(
            start);

        // Assert
        Assert.Equal(pooledComplexArray.DataI[start..].ToArray(), result.DataI.ToArray());
        Assert.Equal(pooledComplexArray.DataQ[start..].ToArray(), result.DataQ.ToArray());
        Assert.Equal(remainingLength, result.Length);
    }

    [Fact]
    public void Slice_StartEqualsLength_Empty()
    {
        // Arrange
        using var pooledComplexArray = GetInitializedPooledComplexArray();
        var start = pooledComplexArray.Length;
        var remainingLength = pooledComplexArray.Length - start;

        // Act
        var result = pooledComplexArray.Slice(
            start);

        // Assert
        Assert.Equal(pooledComplexArray.DataI[start..].ToArray(), result.DataI.ToArray());
        Assert.Equal(pooledComplexArray.DataQ[start..].ToArray(), result.DataQ.ToArray());
        Assert.Equal(remainingLength, result.Length);
        Assert.True(result.IsEmpty);
    }

    [Fact]
    public void Slice_StartOutOfRange_Throw()
    {
        // Arrange
        using var pooledComplexArray = GetInitializedPooledComplexArray();

        // Assert
        Assert.Throws<ArgumentOutOfRangeException>(() => pooledComplexArray.Slice(-1));
        Assert.Throws<ArgumentOutOfRangeException>(() => pooledComplexArray.Slice(pooledComplexArray.Length + 1));
        Assert.Throws<ArgumentOutOfRangeException>(() => pooledComplexArray.Slice(pooledComplexArray.Length, 1));
        Assert.Throws<ArgumentOutOfRangeException>(() => pooledComplexArray.Slice(-1, 1));
    }

    [Fact]
    public void Slice_StartLength_Equal()
    {
        // Arrange
        using var pooledComplexArray = GetInitializedPooledComplexArray();
        var start = 15;
        var length = 40;

        // Act
        var result = pooledComplexArray.Slice(
            start,
            length);

        // Assert
        Assert.Equal(pooledComplexArray.DataI.Slice(start, length).ToArray(), result.DataI.ToArray());
        Assert.Equal(pooledComplexArray.DataQ.Slice(start, length).ToArray(), result.DataQ.ToArray());
        Assert.Equal(length, result.Length);
    }

    [Fact]
    public void Clear_Normal_AllZero()
    {
        // Arrange
        using var pooledComplexArray = GetInitializedPooledComplexArray();

        // Act
        pooledComplexArray.Clear();

        // Assert
        Assert.True(pooledComplexArray.DataI.ToArray().All(x => x == 0));
        Assert.True(pooledComplexArray.DataQ.ToArray().All(x => x == 0));
        Assert.Equal(Length, pooledComplexArray.Length);
    }

    [Fact]
    public void Dispose_Normal_Throw()
    {
        // Arrange
        var pooledComplexArray = GetInitializedPooledComplexArray();

        // Act
        pooledComplexArray.Dispose();

        // Assert
        Assert.Throws<ObjectDisposedException>(() => pooledComplexArray.DataI[0]);
        Assert.Throws<ObjectDisposedException>(() => pooledComplexArray.DataQ[0]);
    }

    [Fact]
    public void Dispose_DoubleDispose_Allow()
    {
        // Arrange
        var pooledComplexArray = GetInitializedPooledComplexArray();

        // Act
        pooledComplexArray.Dispose();
        pooledComplexArray.Dispose();

        // Assert
        Assert.Throws<ObjectDisposedException>(() => pooledComplexArray.DataI[0]);
        Assert.Throws<ObjectDisposedException>(() => pooledComplexArray.DataQ[0]);
    }
}
