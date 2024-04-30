use anyhow::{bail, Result};
use itertools::Itertools as _;
use ordered_float::OrderedFloat;

use super::{
    arrange, measure, Alignment, ArrangeContext, ArrangeResult, ArrangeResultVariant, ElementRef,
    MeasureContext, MeasureResult, MeasureResultVariant, Schedule,
};
use crate::{GridLength, GridLengthUnit};

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

    pub fn with_span(mut self, span: usize) -> Self {
        self.span = span;
        self
    }
}

#[derive(Debug, Clone)]
pub struct Grid {
    children: Vec<GridEntry>,
    columns: Vec<GridLength>,
    channel_ids: Vec<String>,
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
        let channel_ids = children
            .iter()
            .flat_map(|e| e.element.variant.channels())
            .cloned()
            .unique()
            .collect();
        self.children = children;
        self.channel_ids = channel_ids;
        self
    }

    pub fn columns(&self) -> &[GridLength] {
        &self.columns
    }
}

impl Schedule for Grid {
    fn measure(&self, context: &MeasureContext) -> MeasureResult {
        let columns = &self.columns;
        let n_col = columns.len();
        let measured_children: Vec<_> = self
            .children
            .iter()
            .map(|e| measure(e.element.clone(), context.max_duration))
            .collect();
        let mut col_sizes: Vec<_> = columns
            .iter()
            .map(|c| match c.unit {
                GridLengthUnit::Seconds => c.value,
                _ => 0.0,
            })
            .collect();
        let it = measured_children
            .iter()
            .zip_eq(self.children.iter())
            .map(|(m, e)| (m.duration, e.column, e.span));
        for (dur, col, span) in it.clone() {
            let col = col.min(n_col - 1);
            let span = span.min(n_col - col);
            if span == 1 && columns[col].unit != GridLengthUnit::Seconds {
                col_sizes[col] = col_sizes[col].max(dur);
            }
        }
        for (dur, col, span) in it {
            let col = col.min(n_col - 1);
            let span = span.min(n_col - col);
            if span == 1 {
                continue;
            }
            let col_size: f64 = col_sizes.iter().skip(col).take(span).sum();
            if col_size >= dur {
                continue;
            }
            let n_star = columns
                .iter()
                .skip(col)
                .take(span)
                .filter(|c| c.unit == GridLengthUnit::Star)
                .count();
            if n_star == 0 {
                let n_auto = columns
                    .iter()
                    .skip(col)
                    .take(span)
                    .filter(|c| c.unit == GridLengthUnit::Auto)
                    .count();
                if n_auto == 0 {
                    continue;
                }
                let inc = (dur - col_size) / n_auto as f64;
                col_sizes
                    .iter_mut()
                    .zip(columns.iter())
                    .skip(col)
                    .take(span)
                    .filter(|(_, c)| c.unit == GridLengthUnit::Auto)
                    .for_each(|(s, _)| *s += inc);
            } else {
                expand_col_by_ratio(&mut col_sizes, col, span, dur - col_size, columns)
            }
        }
        let wanted_duration = col_sizes.iter().sum();
        MeasureResult(
            wanted_duration,
            super::MeasureResultVariant::Grid(measured_children, col_sizes),
        )
    }

    fn arrange(&self, context: &ArrangeContext) -> Result<ArrangeResult> {
        let (measured_children, mut col_sizes) = match &context.measured_self.data {
            MeasureResultVariant::Grid(children, col_sizes) => (children, col_sizes.clone()),
            _ => bail!("Invalid measure data"),
        };
        let columns = &self.columns;
        let n_col = columns.len();
        let min_duration: f64 = col_sizes.iter().sum();
        expand_col_by_ratio(
            &mut col_sizes,
            0,
            n_col,
            context.final_duration - min_duration,
            columns,
        );
        let col_starts: Vec<_> = std::iter::once(0.0)
            .chain(col_sizes.iter().copied())
            .scan(0.0, |state, x| {
                *state += x;
                Some(*state)
            })
            .collect();
        let arranged_children = measured_children
            .iter()
            .zip(self.children.iter())
            .map(|(m, c)| (m, c.column, c.span))
            .map(|(measured, col, span)| {
                let col = col.min(n_col - 1);
                let span = span.min(n_col - col);
                let span_duration = col_starts[col + span] - col_starts[col];
                let child_duration = match measured.element.common.alignment {
                    Alignment::Stretch => span_duration,
                    _ => measured.duration,
                }
                .min(span_duration);
                let child_time = match measured.element.common.alignment {
                    Alignment::End => span_duration - child_duration,
                    Alignment::Center => (span_duration - child_duration) / 2.0,
                    _ => 0.0,
                } + col_starts[col];
                arrange(measured, child_time, child_duration, context.options)
            })
            .collect::<Result<_>>()?;
        Ok(ArrangeResult(
            context.final_duration,
            ArrangeResultVariant::Multiple(arranged_children),
        ))
    }

    fn channels(&self) -> &[String] {
        &self.channel_ids
    }
}

fn expand_col_by_ratio(
    col_sizes: &mut [f64],
    start: usize,
    span: usize,
    mut left_dur: f64,
    columns: &[GridLength],
) {
    let mut sorted: Vec<_> = col_sizes
        .iter_mut()
        .zip(columns)
        .skip(start)
        .take(span)
        .filter(|(_, c)| c.unit == GridLengthUnit::Star)
        .map(|(s, c)| (*s / c.value, s, c.value))
        .sorted_by_key(|(k, _, _)| OrderedFloat(*k))
        .collect();
    let mut star_count = 0.0;
    for i in 0..sorted.len() {
        let next_ratio = if i + 1 < sorted.len() {
            sorted[i + 1].0
        } else {
            f64::INFINITY
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
}
