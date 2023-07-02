``` ini

BenchmarkDotNet=v0.13.5, OS=Windows 11 (10.0.22621.1848/22H2/2022Update/SunValley2)
AMD Ryzen 5 5600, 1 CPU, 12 logical and 6 physical cores
.NET SDK=7.0.304
  [Host]     : .NET 7.0.7 (7.0.723.27404), X64 RyuJIT AVX2
  DefaultJob : .NET 7.0.7 (7.0.723.27404), X64 RyuJIT AVX2


```
|                     Method |     Mean |    Error |   StdDev |   Gen0 | Allocated |
|--------------------------- |---------:|---------:|---------:|-------:|----------:|
|       GetEnvelopeFromCache | 92.48 μs | 1.164 μs | 1.032 μs | 3.2959 |   56000 B |
| GetEnvelopeFromCacheFaster | 61.34 μs | 0.544 μs | 0.455 μs |      - |         - |
