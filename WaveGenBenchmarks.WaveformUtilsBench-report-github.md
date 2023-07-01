``` ini

BenchmarkDotNet=v0.13.5, OS=Windows 11 (10.0.22621.1848/22H2/2022Update/SunValley2)
AMD Ryzen 5 5600, 1 CPU, 12 logical and 6 physical cores
.NET SDK=7.0.304
  [Host]     : .NET 7.0.7 (7.0.723.27404), X64 RyuJIT AVX2
  DefaultJob : .NET 7.0.7 (7.0.723.27404), X64 RyuJIT AVX2


```
|                  Method | Length |        Mean |     Error |    StdDev |      Median | Ratio | RatioSD |
|------------------------ |------- |------------:|----------:|----------:|------------:|------:|--------:|
|           **MixAddPlateau** |     **16** |    **12.94 ns** |  **0.027 ns** |  **0.023 ns** |    **12.95 ns** |  **0.25** |    **0.00** |
|  MixAddPlateauFrequency |     16 |    28.13 ns |  0.090 ns |  0.080 ns |    28.13 ns |  0.54 |    0.00 |
|                  MixAdd |     16 |    20.65 ns |  0.047 ns |  0.042 ns |    20.66 ns |  0.40 |    0.00 |
|         MixAddFrequency |     16 |    38.46 ns |  0.113 ns |  0.088 ns |    38.49 ns |  0.74 |    0.00 |
|          MixAddWithDrag |     16 |    29.16 ns |  0.143 ns |  0.134 ns |    29.16 ns |  0.56 |    0.00 |
| MixAddFrequencyWithDrag |     16 |    51.62 ns |  0.134 ns |  0.119 ns |    51.61 ns |  1.00 |    0.00 |
|                  Simple |     16 |    63.72 ns |  0.322 ns |  0.301 ns |    63.59 ns |  1.23 |    0.01 |
|                         |        |             |           |           |             |       |         |
|           **MixAddPlateau** |     **64** |    **42.69 ns** |  **0.113 ns** |  **0.105 ns** |    **42.71 ns** |  **0.53** |    **0.01** |
|  MixAddPlateauFrequency |     64 |    46.73 ns |  0.218 ns |  0.193 ns |    46.74 ns |  0.58 |    0.01 |
|                  MixAdd |     64 |    37.26 ns |  0.101 ns |  0.095 ns |    37.27 ns |  0.46 |    0.00 |
|         MixAddFrequency |     64 |    62.61 ns |  0.192 ns |  0.160 ns |    62.61 ns |  0.78 |    0.01 |
|          MixAddWithDrag |     64 |    51.32 ns |  0.466 ns |  0.363 ns |    51.19 ns |  0.64 |    0.01 |
| MixAddFrequencyWithDrag |     64 |    80.17 ns |  0.779 ns |  0.691 ns |    80.01 ns |  1.00 |    0.00 |
|                  Simple |     64 |   217.39 ns |  4.377 ns |  7.892 ns |   213.14 ns |  2.78 |    0.13 |
|                         |        |             |           |           |             |       |         |
|           **MixAddPlateau** |    **256** |    **72.68 ns** |  **0.321 ns** |  **0.300 ns** |    **72.73 ns** |  **0.37** |    **0.00** |
|  MixAddPlateauFrequency |    256 |   122.84 ns |  0.210 ns |  0.197 ns |   122.80 ns |  0.62 |    0.00 |
|                  MixAdd |    256 |    74.62 ns |  0.266 ns |  0.249 ns |    74.60 ns |  0.38 |    0.00 |
|         MixAddFrequency |    256 |   163.94 ns |  0.327 ns |  0.306 ns |   164.00 ns |  0.83 |    0.01 |
|          MixAddWithDrag |    256 |   141.93 ns |  0.305 ns |  0.285 ns |   141.89 ns |  0.72 |    0.00 |
| MixAddFrequencyWithDrag |    256 |   196.62 ns |  1.133 ns |  1.060 ns |   196.16 ns |  1.00 |    0.00 |
|                  Simple |    256 |   782.64 ns |  4.104 ns |  3.839 ns |   781.45 ns |  3.98 |    0.03 |
|                         |        |             |           |           |             |       |         |
|           **MixAddPlateau** |   **1024** |   **277.91 ns** |  **0.331 ns** |  **0.294 ns** |   **277.91 ns** |  **0.40** |    **0.00** |
|  MixAddPlateauFrequency |   1024 |   424.67 ns |  0.439 ns |  0.389 ns |   424.60 ns |  0.61 |    0.00 |
|                  MixAdd |   1024 |   274.94 ns |  1.182 ns |  1.048 ns |   275.11 ns |  0.39 |    0.00 |
|         MixAddFrequency |   1024 |   566.09 ns |  1.301 ns |  1.086 ns |   566.40 ns |  0.81 |    0.00 |
|          MixAddWithDrag |   1024 |   534.09 ns |  2.571 ns |  2.405 ns |   533.32 ns |  0.76 |    0.00 |
| MixAddFrequencyWithDrag |   1024 |   701.37 ns |  2.718 ns |  2.270 ns |   701.43 ns |  1.00 |    0.00 |
|                  Simple |   1024 | 3,061.59 ns | 12.202 ns | 10.189 ns | 3,060.17 ns |  4.37 |    0.02 |
