mod helper;

use std::sync::OnceLock;

use anyhow::{bail, Result};

use crate::{
    quant::{ChannelId, Time},
    schedule::{grid::helper::Helper, merge_channel_ids, Alignment, Arrange, ElementRef, Measure},
    GridLength,
};

#[derive(Debug, Clone)]
pub(crate) struct GridEntry {
    element: ElementRef,
    column: usize,
    span: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct Grid {
    children: Vec<GridEntry>,
    columns: Vec<GridLength>,
    channel_ids: Vec<ChannelId>,
    measure_result: OnceLock<(Time, Vec<Time>)>,
}

impl GridEntry {
    pub(crate) fn new(element: ElementRef) -> Self {
        Self {
            element,
            column: 0,
            span: 1,
        }
    }

    pub(crate) fn with_column(mut self, column: usize) -> Self {
        self.column = column;
        self
    }

    pub(crate) fn with_span(mut self, span: usize) -> Result<Self> {
        if span == 0 {
            bail!("Span should be greater than 0");
        }
        self.span = span;
        Ok(self)
    }
}

impl Grid {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn with_columns(mut self, columns: Vec<GridLength>) -> Self {
        if columns.is_empty() {
            self.columns = vec![GridLength::star(1.0).unwrap()];
        } else {
            self.columns = columns;
        }
        self.measure_result.take();
        self
    }

    pub(crate) fn with_children(mut self, children: Vec<GridEntry>) -> Self {
        let channel_ids = merge_channel_ids(children.iter().map(|e| e.element.variant.channels()));
        self.children = children;
        self.channel_ids = channel_ids;
        self.measure_result.take();
        self
    }

    pub(crate) fn columns(&self) -> &[GridLength] {
        &self.columns
    }

    fn measure_result(&self) -> &(Time, Vec<Time>) {
        self.measure_result.get_or_init(|| {
            measure_grid(
                self.children
                    .iter()
                    .map(|e| (e.element.clone(), e.column, e.span)),
                &self.columns,
            )
        })
    }
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            children: vec![],
            columns: vec![GridLength::star(1.0).unwrap()],
            channel_ids: vec![],
            measure_result: OnceLock::new(),
        }
    }
}

impl Measure for Grid {
    fn measure(&self) -> Time {
        let (total_duration, _) = self.measure_result();
        *total_duration
    }

    fn channels(&self) -> &[ChannelId] {
        &self.channel_ids
    }
}

// impl Schedule for Grid {
//     fn measure(&self) -> MeasureResult {
//         let mut helper = Helper::new(&self.columns);
//         let measured_children: Vec<_> = self
//             .children
//             .iter()
//             .map(|e| measure(e.element.clone()))
//             .collect();
//         let it = measured_children
//             .iter()
//             .zip(self.children.iter())
//             .map(|(m, e)| (m.duration, e.column, e.span));
//         for (duration, column, span) in it.clone() {
//             let span = helper.normalize_span(column, span);
//             if span.span() == 1 {
//                 helper.expand_span_to_fit(span, duration);
//             }
//         }
//         for (duration, column, span) in it {
//             let span = helper.normalize_span(column, span);
//             if span.span() != 1 {
//                 helper.expand_span_to_fit(span, duration);
//             }
//         }
//         let column_sizes = helper.into_column_sizes();
//         let wanted_duration = column_sizes.iter().sum();
//         MeasureResult(
//             wanted_duration,
//             super::MeasureResultVariant::Grid(measured_children, column_sizes),
//         )
//     }

//     fn arrange(&self, context: &ArrangeContext) -> Result<ArrangeResult> {
//         let (measured_children, column_sizes) = match &context.measured_self.data {
//             MeasureResultVariant::Grid(children, column_sizes) => (children, column_sizes.clone()),
//             _ => bail!("Invalid measure data"),
//         };
//         let mut helper = Helper::new_with_column_sizes(&self.columns, column_sizes);
//         helper.expand_to_fit(context.final_duration);
//         let column_starts = helper.column_starts();
//         let arranged_children = measured_children
//             .iter()
//             .zip(self.children.iter())
//             .map(|(m, c)| (m, c.column, c.span))
//             .map(|(measured, column, span)| {
//                 let span = helper.normalize_span(column, span);
//                 let start = span.start();
//                 let span = span.span();
//                 let span_duration = column_starts[start + span] - column_starts[start];
//                 let child_duration = match measured.element.common.alignment {
//                     Alignment::Stretch => span_duration,
//                     _ => measured.duration,
//                 }
//                 .min(span_duration);
//                 let child_time = match measured.element.common.alignment {
//                     Alignment::End => span_duration - child_duration,
//                     Alignment::Center => (span_duration - child_duration) / 2.0,
//                     _ => Time::ZERO,
//                 } + column_starts[start];
//                 arrange(measured, child_time, child_duration, context.options)
//             })
//             .collect::<Result<_>>()?;
//         Ok(ArrangeResult(
//             context.final_duration,
//             ArrangeResultVariant::Multiple(arranged_children),
//         ))
//     }

//     fn channels(&self) -> &[ChannelId] {
//         &self.channel_ids
//     }
// }

fn arrange_grid<'a, I, M>(
    children: I,
    columns: &'a [GridLength],
    final_duration: Time,
    column_sizes: Vec<Time>,
) -> impl IntoIterator<Item = (M, Time, Time)> + 'a
where
    I: IntoIterator<Item = (M, usize, usize)>,
    I::IntoIter: 'a,
    M: Measure + Arrange + 'a,
{
    let mut helper = Helper::new_with_column_sizes(columns, column_sizes);
    helper.expand_to_fit(final_duration);
    let column_starts = helper.column_starts();
    children.into_iter().map(move |(m, column, span)| {
        let span = helper.normalize_span(column, span);
        let start = span.start();
        let span = span.span();
        let span_duration = column_starts[start + span] - column_starts[start];
        let child_alignment = m.alignment();
        let child_duration = match child_alignment {
            Alignment::Stretch => span_duration,
            _ => m.measure(),
        }
        .min(span_duration);
        let child_time = match child_alignment {
            Alignment::End => span_duration - child_duration,
            Alignment::Center => (span_duration - child_duration) / 2.0,
            _ => Time::ZERO,
        } + column_starts[start];
        (m, child_time, child_duration)
    })
}

/// Measure grid children and return a tuple of minimum duration, minimum column
/// sizes and child offsets.
fn measure_grid<I, M>(children: I, columns: &[GridLength]) -> (Time, Vec<Time>)
where
    I: IntoIterator<Item = (M, usize, usize)>,
    M: Measure,
{
    let mut helper = Helper::new(columns);
    let children = children
        .into_iter()
        .map(|(m, column, span)| {
            let duration = m.measure();
            (duration, column, span)
        })
        .collect::<Vec<_>>();
    for &(duration, column, span) in &children {
        let span = helper.normalize_span(column, span);
        if span.span() == 1 {
            helper.expand_span_to_fit(span, duration);
        }
    }
    for &(duration, column, span) in &children {
        let span = helper.normalize_span(column, span);
        if span.span() != 1 {
            helper.expand_span_to_fit(span, duration);
        }
    }
    let column_sizes = helper.into_column_sizes();
    let total_duration = column_sizes.iter().sum();
    (total_duration, column_sizes)
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use crate::schedule::MockMeasure;

    use super::*;

    fn time_vec(v: &[f64]) -> Vec<Time> {
        v.iter().map(|&d| Time::new(d).unwrap()).collect()
    }

    fn create_mock(duration: f64) -> MockMeasure {
        let mut mock = MockMeasure::new();
        mock.expect_measure()
            .return_const(Time::new(duration).unwrap())
            .once();
        mock
    }

    #[test_case(&[(40.0, 0, 1)], &["30"], (30.0, vec![30.0]); "not enough size")]
    #[test_case(
        &[(40.0, 0, 1), (40.0, 2, 1), (100.0, 0, 3)],
        &["auto", "*", "auto"],
        (100.0, vec![40.0, 20.0, 40.0]);
        "sandwiched"
    )]
    #[test_case(&[], &["*"], (0.0, vec![0.0]); "empty star")]
    #[test_case(&[], &["10"], (10.0, vec![10.0]); "empty fixed")]
    fn measure_grid(children: &[(f64, usize, usize)], columns: &[&str], expected: (f64, Vec<f64>)) {
        let children = children.iter().map(|&(d, c, s)| (create_mock(d), c, s));
        let columns: Vec<GridLength> = columns.iter().map(|s| s.parse().unwrap()).collect();

        let (total_duration, column_sizes) = super::measure_grid(children, &columns);

        assert_eq!(total_duration, Time::new(expected.0).unwrap());
        assert_eq!(column_sizes, time_vec(&expected.1));
    }
}
