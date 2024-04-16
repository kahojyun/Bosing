use super::{
    arrange, measure, Alignment, ArrangeContext, ArrangeResult, ArrangeResultVariant, Element,
    MeasureContext, MeasureResult, MeasureResultVariant, Schedule,
};
use anyhow::{bail, Result};
use itertools::Itertools;
use ordered_float::OrderedFloat;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridLengthUnit {
    Seconds,
    Auto,
    Star,
}

#[derive(Debug, Clone)]
pub struct GridLength {
    value: f64,
    unit: GridLengthUnit,
}

impl GridLength {
    pub fn seconds(value: f64) -> Self {
        Self {
            value,
            unit: GridLengthUnit::Seconds,
        }
    }

    pub fn auto() -> Self {
        Self {
            value: 0.0,
            unit: GridLengthUnit::Auto,
        }
    }

    pub fn star(value: f64) -> Self {
        Self {
            value,
            unit: GridLengthUnit::Star,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GridEntry {
    pub(super) element: Rc<Element>,
    column: usize,
    span: usize,
}

#[derive(Debug, Clone)]
pub struct Grid {
    pub(super) children: Vec<GridEntry>,
    pub(super) columns: Vec<GridLength>,
    pub(super) channel_ids: Vec<usize>,
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

    fn channels(&self) -> &[usize] {
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
