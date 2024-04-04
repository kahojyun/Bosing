using System.Diagnostics;

using CommunityToolkit.Diagnostics;

namespace Bosing.Schedules;
public class GridSchedule : Schedule
{
    private readonly List<(int Column, int Span)> _elementColumns = [];
    private readonly List<GridLength> _columns = [];
    private List<double>? _minimumColumnSizes;

    public GridSchedule()
    {
        Alignment = Alignment.Stretch;
    }

    public void AddColumn(GridLength length)
    {
        Guard.IsTrue(length.IsValid);
        _columns.Add(length);
    }

    public void Add(ScheduleElement element)
    {
        Add(element, 0, 1);
    }

    public void Add(ScheduleElement element, int column, int span)
    {
        if (element.Parent is not null)
        {
            ThrowHelper.ThrowArgumentException("The element is already added to another schedule.");
        }
        Guard.IsGreaterThanOrEqualTo(column, 0);
        Guard.IsGreaterThanOrEqualTo(span, 1);
        Children.Add(element);
        element.Parent = this;
        _elementColumns.Add((column, span));
    }

    protected override double ArrangeOverride(double time, double finalDuration)
    {
        Debug.Assert(_minimumColumnSizes is not null);
        var columnSizes = _minimumColumnSizes.ToList();
        var numColumns = _columns.Count;
        var minimumDuration = columnSizes.Sum();
        ExpandColumnByRatio(columnSizes, 0, numColumns, finalDuration - minimumDuration);
        var columnStarts = new List<double>(numColumns) { 0 };
        for (var i = 1; i < numColumns; i++)
        {
            columnStarts.Add(columnStarts[i - 1] + columnSizes[i - 1]);
        }
        foreach (var (element, (column, span)) in Children.Zip(_elementColumns))
        {
            Debug.Assert(element.DesiredDuration is not null);
            var actualColumn = Math.Min(column, numColumns - 1);
            var actualSpan = Math.Min(span, numColumns - actualColumn);
            var spanDuration = Enumerable.Range(actualColumn, actualSpan).Sum(i => columnSizes[i]);
            var elementDuration = element.Alignment == Alignment.Stretch ? spanDuration : element.DesiredDuration.Value;
            var actualDuration = Math.Min(spanDuration, elementDuration);
            var elementTime = element.Alignment switch
            {
                Alignment.Start => 0,
                Alignment.Center => (spanDuration - actualDuration) / 2,
                Alignment.End => spanDuration - actualDuration,
                Alignment.Stretch => 0,
                _ => throw new NotImplementedException(),
            };
            var actualTime = columnStarts[actualColumn] + elementTime;
            element.Arrange(actualTime, actualDuration);
        }
        return finalDuration;
    }

    protected override double MeasureOverride(double maxDuration)
    {
        if (_columns.Count == 0)
        {
            _columns.Add(GridLength.Star(1));
        }
        var numColumns = _columns.Count;
        foreach (var element in Children)
        {
            element.Measure(maxDuration);
        }
        var columnSizes = _columns.Select(l => l.IsAbsolute ? l.Value : 0).ToList();
        foreach (var (element, (column, span)) in Children.Zip(_elementColumns))
        {
            var actualColumn = Math.Min(column, numColumns - 1);
            var actualSpan = Math.Min(span, numColumns - actualColumn);
            if (actualSpan > 1)
            {
                continue;
            }
            if (_columns[actualColumn].IsAbsolute)
            {
                continue;
            }
            Debug.Assert(element.DesiredDuration is not null);
            var elementDuration = element.DesiredDuration.Value;
            columnSizes[actualColumn] = Math.Max(columnSizes[actualColumn], elementDuration);
        }
        foreach (var (element, (column, span)) in Children.Zip(_elementColumns))
        {
            var actualColumn = Math.Min(column, numColumns - 1);
            var actualSpan = Math.Min(span, numColumns - actualColumn);
            if (actualSpan == 1)
            {
                continue;
            }
            Debug.Assert(element.DesiredDuration is not null);
            var elementDuration = element.DesiredDuration.Value;
            var columnSize = Enumerable.Range(actualColumn, actualSpan).Sum(i => columnSizes[i]);
            if (columnSize >= elementDuration)
            {
                continue;
            }
            var numStar = Enumerable.Range(actualColumn, actualSpan).Count(i => _columns[i].IsStar);
            if (numStar == 0)
            {
                var numAuto = Enumerable.Range(actualColumn, actualSpan).Count(i => _columns[i].IsAuto);
                if (numAuto == 0)
                {
                    continue;
                }
                var autoIncrement = (elementDuration - columnSize) / numAuto;
                for (var i = actualColumn; i < actualColumn + actualSpan; i++)
                {
                    if (_columns[i].IsAuto)
                    {
                        columnSizes[i] += autoIncrement;
                    }
                }
            }
            else
            {
                ExpandColumnByRatio(columnSizes, actualColumn, actualSpan, elementDuration - columnSize);
            }
        }
        _minimumColumnSizes = columnSizes;
        return columnSizes.Sum();
    }

    private void ExpandColumnByRatio(List<double> columnSizes, int start, int span, double remainingDuration)
    {
        var indexOrderByStarRatio = Enumerable.Range(start, span)
            .Where(i => _columns[i].IsStar)
            .Select(i => (Ratio: columnSizes[i] / _columns[i].Value, Index: i))
            .OrderBy(x => x.Ratio)
            .ToList();
        var cumulativeStar = 0.0;
        for (var i = 0; i < indexOrderByStarRatio.Count; i++)
        {
            var nextRatio = i + 1 < indexOrderByStarRatio.Count ? indexOrderByStarRatio[i + 1].Ratio : double.PositiveInfinity;
            var index = indexOrderByStarRatio[i].Index;
            cumulativeStar += _columns[index].Value;
            remainingDuration += columnSizes[index];
            var newRatio = remainingDuration / cumulativeStar;
            if (newRatio < nextRatio)
            {
                for (var j = 0; j <= i; j++)
                {
                    var index2 = indexOrderByStarRatio[j].Index;
                    columnSizes[index2] = newRatio * _columns[index2].Value;
                }
                break;
            }
        }
    }
}
