use anyhow::{bail, Result};
use itertools::Itertools as _;

use super::{
    arrange, measure, merge_channel_ids, Alignment, ArrangeContext, ArrangeResult,
    ArrangeResultVariant, ElementRef, Measure, MeasureResult, MeasureResultVariant, Schedule,
};
use crate::{
    quant::{ChannelId, Time},
    GridLength,
};

#[derive(Debug, Clone)]
pub struct GridEntry {
    element: ElementRef,
    column: usize,
    span: usize,
}

impl GridEntry {
    pub fn new(element: ElementRef) -> Self {
        Self {
            element,
            column: 0,
            span: 1,
        }
    }

    pub fn with_column(mut self, column: usize) -> Self {
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

#[derive(Debug, Clone)]
pub struct Grid {
    children: Vec<GridEntry>,
    columns: Vec<GridLength>,
    channel_ids: Vec<ChannelId>,
}

impl Default for Grid {
    fn default() -> Self {
        Self::new()
    }
}

impl Grid {
    pub fn new() -> Self {
        Self {
            children: vec![],
            columns: vec![GridLength::star(1.0).unwrap()],
            channel_ids: vec![],
        }
    }

    pub fn with_columns(mut self, columns: Vec<GridLength>) -> Self {
        if columns.is_empty() {
            self.columns = vec![GridLength::star(1.0).unwrap()];
        } else {
            self.columns = columns;
        }
        self
    }

    pub fn with_children(mut self, children: Vec<GridEntry>) -> Self {
        let channel_ids = merge_channel_ids(children.iter().map(|e| e.element.variant.channels()));
        self.children = children;
        self.channel_ids = channel_ids;
        self
    }

    pub fn columns(&self) -> &[GridLength] {
        &self.columns
    }
}

impl Schedule for Grid {
    fn measure(&self) -> MeasureResult {
        let mut helper = Helper::new(&self.columns);
        let measured_children: Vec<_> = self
            .children
            .iter()
            .map(|e| measure(e.element.clone()))
            .collect();
        let it = measured_children
            .iter()
            .zip(self.children.iter())
            .map(|(m, e)| (m.duration, e.column, e.span));
        for (dur, col, span) in it.clone() {
            helper.expand_col_to_fit(col, span, dur);
        }
        for (dur, col, span) in it {
            helper.expand_span_to_fit(col, span, dur);
        }
        let col_sizes = helper.into_col_sizes();
        let wanted_duration = col_sizes.iter().sum();
        MeasureResult(
            wanted_duration,
            super::MeasureResultVariant::Grid(measured_children, col_sizes),
        )
    }

    fn arrange(&self, context: &ArrangeContext) -> Result<ArrangeResult> {
        let (measured_children, col_sizes) = match &context.measured_self.data {
            MeasureResultVariant::Grid(children, col_sizes) => (children, col_sizes.clone()),
            _ => bail!("Invalid measure data"),
        };
        let mut helper = Helper::new_with_col_sizes(&self.columns, col_sizes);
        helper.expand_span_to_fit(0, self.columns.len(), context.final_duration);
        let col_starts = helper.col_starts();
        let arranged_children = measured_children
            .iter()
            .zip(self.children.iter())
            .map(|(m, c)| (m, c.column, c.span))
            .map(|(measured, col, span)| {
                let (col, span) = helper.normalize_span(col, span);
                let span_duration = col_starts[col + span] - col_starts[col];
                let child_duration = match measured.element.common.alignment {
                    Alignment::Stretch => span_duration,
                    _ => measured.duration,
                }
                .min(span_duration);
                let child_time = match measured.element.common.alignment {
                    Alignment::End => span_duration - child_duration,
                    Alignment::Center => (span_duration - child_duration) / 2.0,
                    _ => Time::ZERO,
                } + col_starts[col];
                arrange(measured, child_time, child_duration, context.options)
            })
            .collect::<Result<_>>()?;
        Ok(ArrangeResult(
            context.final_duration,
            ArrangeResultVariant::Multiple(arranged_children),
        ))
    }

    fn channels(&self) -> &[ChannelId] {
        &self.channel_ids
    }
}

/// Measure grid children and return a tuple of minimum duration, minimum column
/// sizes and child offsets.
fn measure_grid<I, M>(children: I, columns: &[GridLength]) -> (Time, Vec<Time>, Vec<Time>)
where
    I: IntoIterator<Item = (M, usize, usize)>,
    I::IntoIter: DoubleEndedIterator,
    M: Measure,
{
    let mut helper = Helper::new(columns);
    let children = children
        .into_iter()
        .map(|(m, col, span)| {
            let dur = m.measure();
            let alignment = m.alignment();
            (dur, col, span, alignment)
        })
        .collect::<Vec<_>>();
    for (dur, col, span, _) in &children {
        helper.expand_col_to_fit(*col, *span, *dur);
    }
    for (dur, col, span, _) in &children {
        helper.expand_span_to_fit(*col, *span, *dur);
    }
    let col_starts = helper.col_starts();
    let child_offsets = children
        .into_iter()
        .map(|(dur, col, span, alignment)| {
            let (col, span) = helper.normalize_span(col, span);
            let span_duration = col_starts[col + span] - col_starts[col];
            let child_duration = match alignment {
                Alignment::Stretch => span_duration,
                _ => dur,
            }
            .min(span_duration);
            col_starts[col]
                + match alignment {
                    Alignment::End => span_duration - child_duration,
                    Alignment::Center => (span_duration - child_duration) / 2.0,
                    _ => Time::ZERO,
                }
        })
        .collect();
    let col_sizes = helper.into_col_sizes();
    let total = col_sizes.iter().sum();
    (total, col_sizes, child_offsets)
}

#[derive(Debug)]
struct Helper<'a> {
    col_sizes: Vec<Time>,
    columns: &'a [GridLength],
}

impl<'a> Helper<'a> {
    fn new(columns: &'a [GridLength]) -> Self {
        let col_sizes = columns
            .iter()
            .map(|c| {
                if c.is_fixed() {
                    Time::new(c.value).expect("Should be checked in GridLenth")
                } else {
                    Time::ZERO
                }
            })
            .collect();
        Self { col_sizes, columns }
    }

    fn new_with_col_sizes(columns: &'a [GridLength], col_sizes: Vec<Time>) -> Self {
        assert!(columns.len() == col_sizes.len());
        Self { col_sizes, columns }
    }

    fn normalize_span(&self, col: usize, span: usize) -> (usize, usize) {
        let n_col = self.columns.len();
        let col = col.min(n_col - 1);
        let span = span.min(n_col - col);
        (col, span)
    }

    /// Expand span of columns to fit the new duration, return true if expanded
    /// or already fit.
    fn expand_span_to_fit(&mut self, col: usize, span: usize, new_dur: Time) -> bool {
        let (col, span) = self.normalize_span(col, span);
        let current_dur: Time = self.col_sizes.iter().skip(col).take(span).sum();
        if current_dur >= new_dur {
            return true;
        }
        if span == 1 {
            return if !self.columns[col].is_fixed() {
                self.col_sizes[col] = new_dur;
                true
            } else {
                false
            };
        }
        let left_dur = new_dur - current_dur;
        self.expand_span_by_star_ratio(col, span, left_dur)
            || self.expand_span_by_auto_count(col, span, left_dur)
    }

    /// Expand only one column to fit the new duration, return true if expanded
    fn expand_col_to_fit(&mut self, col: usize, span: usize, new_dur: Time) -> bool {
        let (col, span) = self.normalize_span(col, span);
        if span == 1 {
            self.expand_span_to_fit(col, span, new_dur)
        } else {
            false
        }
    }

    fn expand_span_by_auto_count(&mut self, col: usize, span: usize, left_dur: Time) -> bool {
        let n_auto = self
            .columns
            .iter()
            .skip(col)
            .take(span)
            .filter(|c| c.is_auto())
            .count();
        if n_auto == 0 {
            return false;
        }
        let inc = left_dur / n_auto as f64;
        self.col_sizes
            .iter_mut()
            .zip(self.columns)
            .skip(col)
            .take(span)
            .filter(|(_, c)| c.is_auto())
            .for_each(|(s, _)| *s += inc);
        true
    }

    fn expand_span_by_star_ratio(&mut self, col: usize, span: usize, mut left_dur: Time) -> bool {
        let mut sorted: Vec<_> = self
            .col_sizes
            .iter_mut()
            .zip(self.columns)
            .skip(col)
            .take(span)
            .filter(|(_, c)| c.is_star())
            .map(|(s, c)| (*s / c.value, s, c.value))
            .sorted_by_key(|(k, _, _)| *k)
            .collect();
        if sorted.is_empty() {
            return false;
        }
        let mut star_count = 0.0;
        for i in 0..sorted.len() {
            let next_ratio = if i + 1 < sorted.len() {
                sorted[i + 1].0
            } else {
                Time::INFINITY
            };
            star_count += sorted[i].2;
            left_dur += *sorted[i].1;
            let new_ratio = left_dur / star_count;
            if new_ratio < next_ratio {
                for (_, s, v) in sorted.iter_mut().take(i + 1) {
                    **s = new_ratio * *v;
                }
                break;
            }
        }
        true
    }

    fn col_starts(&self) -> Vec<Time> {
        std::iter::once(Time::ZERO)
            .chain(self.col_sizes.iter().copied())
            .scan(Time::ZERO, |state, x| {
                *state += x;
                Some(*state)
            })
            .collect()
    }

    fn into_col_sizes(self) -> Vec<Time> {
        self.col_sizes
    }
}
