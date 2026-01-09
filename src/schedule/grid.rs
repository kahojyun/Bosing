mod helper;

use std::{str::FromStr, sync::OnceLock};

use anyhow::{Result, bail};

use crate::quant::{ChannelId, Time};

use super::{Alignment, Arrange, Arranged, ElementRef, Measure, TimeRange, merge_channel_ids};

use self::helper::Helper;

#[derive(Debug, Clone, Copy)]
pub enum Length {
    Fixed(f64),
    Star(f64),
    Auto,
}

impl Length {
    pub const STAR: Self = Self::Star(1.0);

    #[must_use]
    pub const fn auto() -> Self {
        Self::Auto
    }

    pub fn star(value: f64) -> Result<Self> {
        if !(value.is_finite() && value > 0.0) {
            bail!("The value must be greater than 0.");
        }
        Ok(Self::Star(value))
    }

    pub fn fixed(value: f64) -> Result<Self> {
        if !(value.is_finite() && value >= 0.0) {
            bail!("The value must be greater than or equal to 0.");
        }
        Ok(Self::Fixed(value))
    }

    #[must_use]
    pub const fn is_auto(&self) -> bool {
        matches!(self, Self::Auto)
    }

    #[must_use]
    pub const fn is_star(&self) -> bool {
        matches!(self, Self::Star(_))
    }

    #[must_use]
    pub const fn is_fixed(&self) -> bool {
        matches!(self, Self::Fixed(_))
    }
}

impl FromStr for Length {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "auto" {
            return Ok(Self::auto());
        }
        if s == "*" {
            return Ok(Self::STAR);
        }
        if let Some(v) = s.strip_suffix('*').and_then(|x| x.parse().ok()) {
            return Self::star(v);
        }
        if let Ok(v) = s.parse() {
            return Self::fixed(v);
        }
        Err(anyhow::anyhow!("Invalid string: {s}"))
    }
}

#[derive(Debug, Clone)]
pub struct Entry {
    element: ElementRef,
    column: usize,
    span: usize,
}

#[derive(Debug, Clone)]
pub struct Grid {
    children: Vec<Entry>,
    columns: Vec<Length>,
    channel_ids: Vec<ChannelId>,
    measure_result: OnceLock<MeasureResult>,
}

#[derive(Debug, Clone)]
struct MeasureResult {
    total_duration: Time,
    column_sizes: Vec<Time>,
    child_durations: Vec<Time>,
}

struct MeasureItem {
    column: usize,
    span: usize,
    duration: Time,
}

impl Entry {
    pub const fn new(element: ElementRef) -> Self {
        Self {
            element,
            column: 0,
            span: 1,
        }
    }

    #[must_use]
    pub const fn with_column(mut self, column: usize) -> Self {
        self.column = column;
        self
    }

    pub fn with_span(mut self, span: usize) -> Result<Self> {
        if span == 0 {
            bail!("Span should be greater than 0");
        }
        self.span = span;
        Ok(self)
    }
}

impl Grid {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_columns(mut self, columns: Vec<Length>) -> Self {
        if columns.is_empty() {
            self.columns = vec![Length::STAR];
        } else {
            self.columns = columns;
        }
        self.measure_result.take();
        self
    }

    #[must_use]
    pub fn with_children(mut self, children: Vec<Entry>) -> Self {
        let channel_ids = merge_channel_ids(children.iter().map(|e| e.element.variant.channels()));
        self.children = children;
        self.channel_ids = channel_ids;
        self.measure_result.take();
        self
    }

    pub fn columns(&self) -> &[Length] {
        &self.columns
    }

    fn measure_result(&self) -> &MeasureResult {
        self.measure_result.get_or_init(|| {
            measure_grid(
                self.children.iter().map(|e| MeasureItem {
                    duration: e.element.measure(),
                    column: e.column,
                    span: e.span,
                }),
                &self.columns,
            )
        })
    }
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            children: vec![],
            columns: vec![Length::STAR],
            channel_ids: vec![],
            measure_result: OnceLock::new(),
        }
    }
}

impl Measure for Grid {
    fn measure(&self) -> Time {
        let MeasureResult { total_duration, .. } = self.measure_result();
        *total_duration
    }

    fn channels(&self) -> &[ChannelId] {
        &self.channel_ids
    }
}

impl Arrange for Grid {
    fn arrange(&self, time_range: TimeRange) -> impl Iterator<Item = Arranged<&ElementRef>> {
        let MeasureResult {
            column_sizes,
            child_durations,
            ..
        } = self.measure_result();
        let mut helper = Helper::new_with_column_sizes(&self.columns, column_sizes.clone());
        helper.expand_to_fit(time_range.span);
        let column_starts = helper.column_starts();
        self.children.iter().zip(child_durations).map(
            move |(
                Entry {
                    element,
                    column,
                    span,
                },
                &child_duration,
            )| {
                let span = helper.normalize_span(*column, *span);
                let start = span.start();
                let span = span.span();
                let span_duration = column_starts[start + span] - column_starts[start];
                let child_duration = match element.common.alignment {
                    Alignment::Stretch => span_duration,
                    _ => child_duration,
                };
                let child_offset = match element.common.alignment {
                    Alignment::End => span_duration - child_duration,
                    Alignment::Center => (span_duration - child_duration) / 2.0,
                    _ => Time::ZERO,
                } + column_starts[start];
                let child_time_range = TimeRange {
                    start: time_range.start + child_offset,
                    span: child_duration,
                };
                Arranged {
                    item: element,
                    time_range: child_time_range,
                }
            },
        )
    }
}

fn measure_grid<I>(children: I, columns: &[Length]) -> MeasureResult
where
    I: IntoIterator<Item = MeasureItem>,
{
    let mut helper = Helper::new(columns);
    let children: Vec<MeasureItem> = children.into_iter().collect();
    for &MeasureItem {
        duration,
        column,
        span,
    } in &children
    {
        let span = helper.normalize_span(column, span);
        if span.span() == 1 {
            helper.expand_span_to_fit(span, duration);
        }
    }
    for &MeasureItem {
        duration,
        column,
        span,
    } in &children
    {
        let span = helper.normalize_span(column, span);
        if span.span() != 1 {
            helper.expand_span_to_fit(span, duration);
        }
    }
    let column_sizes = helper.into_column_sizes();
    let total_duration = column_sizes.iter().sum();
    MeasureResult {
        total_duration,
        column_sizes,
        child_durations: children.into_iter().map(|item| item.duration).collect(),
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    fn time_vec(v: &[f64]) -> Vec<Time> {
        v.iter().map(|&d| Time::new(d).unwrap()).collect()
    }

    #[test_case(&[(40.0, 0, 1)], &["30"], (30.0, &[30.0]); "not enough size")]
    #[test_case(
        &[(40.0, 0, 1), (40.0, 2, 1), (100.0, 0, 3)],
        &["auto", "*", "auto"],
        (100.0, &[40.0, 20.0, 40.0]);
        "sandwiched"
    )]
    #[test_case(&[], &["*"], (0.0, &[0.0]); "empty star")]
    #[test_case(&[], &["10"], (10.0, &[10.0]); "empty fixed")]
    fn measure_grid(children: &[(f64, usize, usize)], columns: &[&str], expected: (f64, &[f64])) {
        let children = children
            .iter()
            .map(|&(duration, column, span)| MeasureItem {
                duration: Time::new(duration).unwrap(),
                column,
                span,
            });
        let columns: Vec<Length> = columns.iter().map(|s| s.parse().unwrap()).collect();

        let MeasureResult {
            total_duration,
            column_sizes,
            ..
        } = super::measure_grid(children, &columns);

        assert_eq!(total_duration, Time::new(expected.0).unwrap());
        assert_eq!(column_sizes, time_vec(expected.1));
    }
}
