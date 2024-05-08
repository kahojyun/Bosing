use crate::{quant::Time, GridLength};

#[derive(Debug, Clone, Copy)]
pub(super) struct NormalizedSpan {
    start: usize,
    span: usize,
}

#[derive(Debug)]
pub(super) struct Helper<'a> {
    column_sizes: Vec<Time>,
    columns: &'a [GridLength],
}

impl NormalizedSpan {
    pub(super) fn start(&self) -> usize {
        self.start
    }

    pub(super) fn span(&self) -> usize {
        self.span
    }
}

impl<'a> Helper<'a> {
    pub(super) fn new(columns: &'a [GridLength]) -> Self {
        let column_sizes = columns
            .iter()
            .map(|c| {
                if c.is_fixed() {
                    Time::new(c.value).expect("Should be checked in GridLenth")
                } else {
                    Time::ZERO
                }
            })
            .collect();
        Self {
            column_sizes,
            columns,
        }
    }

    pub(super) fn new_with_column_sizes(
        columns: &'a [GridLength],
        column_sizes: Vec<Time>,
    ) -> Self {
        assert!(columns.len() == column_sizes.len());
        Self {
            column_sizes,
            columns,
        }
    }

    pub(super) fn column_starts(&self) -> Vec<Time> {
        prefix_sum(&self.column_sizes)
    }

    pub(super) fn into_column_sizes(self) -> Vec<Time> {
        self.column_sizes
    }

    pub(super) fn normalize_span(&self, col: usize, span: usize) -> NormalizedSpan {
        let n_col = self.columns.len();
        let col = col.min(n_col - 1);
        let span = span.min(n_col - col);
        NormalizedSpan { start: col, span }
    }

    pub(super) fn expand_to_fit(&mut self, required: Time) -> bool {
        let span = NormalizedSpan {
            start: 0,
            span: self.columns.len(),
        };
        self.expand_span_to_fit(span, required)
    }

    /// Expand span of columns to fit the new duration, return true if expanded
    /// or already fit.
    pub(super) fn expand_span_to_fit(&mut self, span: NormalizedSpan, required: Time) -> bool {
        let NormalizedSpan { start, span } = span;
        let current: Time = self.column_sizes.iter().skip(start).take(span).sum();
        if current >= required {
            return true;
        }
        if span == 1 {
            return if !self.columns[start].is_fixed() {
                self.column_sizes[start] = required;
                true
            } else {
                false
            };
        }
        let remaining = required - current;
        let span = NormalizedSpan { start, span };
        self.expand_span_by_star_ratio(span, remaining)
            || self.expand_span_by_auto_count(span, remaining)
    }

    fn expand_span_by_auto_count(&mut self, span: NormalizedSpan, remaining: Time) -> bool {
        let NormalizedSpan { start, span } = span;
        let n_auto = self
            .columns
            .iter()
            .skip(start)
            .take(span)
            .filter(|c| c.is_auto())
            .count();
        if n_auto == 0 {
            return false;
        }
        let increment = remaining / n_auto as f64;
        self.column_sizes
            .iter_mut()
            .zip(self.columns)
            .skip(start)
            .take(span)
            .filter(|(_, c)| c.is_auto())
            .for_each(|(s, _)| *s += increment);
        true
    }

    fn expand_span_by_star_ratio(&mut self, span: NormalizedSpan, mut remaining: Time) -> bool {
        let NormalizedSpan { start, span } = span;
        let mut sorted = {
            let mut items: Vec<StarItem> = self
                .column_sizes
                .iter_mut()
                .zip(self.columns)
                .skip(start)
                .take(span)
                .filter(|(_, column)| column.is_star())
                .map(|(column_size, column)| StarItem {
                    size_per_star: *column_size / column.value,
                    column_size,
                    star: column.value,
                })
                .collect();
            if items.is_empty() {
                return false;
            }
            items.sort_by_key(|x| x.size_per_star);
            items
        };
        let mut star_count = 0.0;
        for i in 0..sorted.len() {
            let next_size_per_star = sorted
                .get(i + 1)
                .map_or(Time::INFINITY, |x| x.size_per_star);
            star_count += sorted[i].star;
            remaining += *sorted[i].column_size;
            let new_size_per_star = remaining / star_count;
            if new_size_per_star < next_size_per_star {
                for item in sorted.iter_mut().take(i + 1) {
                    *item.column_size = new_size_per_star * item.star;
                }
                break;
            }
        }
        return true;

        struct StarItem<'a> {
            size_per_star: Time,
            column_size: &'a mut Time,
            star: f64,
        }
    }
}

fn prefix_sum<T>(arr: &[T]) -> Vec<T>
where
    T: std::ops::Add<Output = T> + Copy + Default,
{
    let mut res = Vec::with_capacity(arr.len() + 1);
    let mut sum = T::default();
    res.push(sum);
    for &x in arr {
        sum = sum + x;
        res.push(sum);
    }
    res
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test]
    fn prefix_sum_should_prepend_zero() {
        let arr = vec![1, 2, 3, 4, 5];
        let expected = vec![0, 1, 3, 6, 10, 15];
        assert_eq!(prefix_sum(&arr), expected);
    }

    #[test]
    fn new_should_init_fixed_columns() {
        let columns = ["1", "2", "3", "auto", "*"].map(|s| s.parse().unwrap());
        let helper = Helper::new(&columns);
        assert_eq!(
            helper.column_sizes,
            [1.0, 2.0, 3.0, 0.0, 0.0].map(|s| Time::new(s).unwrap())
        );
    }

    #[test_case(&["*", "2*", "3*"], 0, 3, true, &[1.0, 2.0, 3.0]; "increase by ratio")]
    #[test_case(&["auto", "1"], 0, 2, false, &[0.0, 1.0]; "ignore auto and fixed")]
    #[test_case(&["*", "2*", "3*"], 0, 2, true, &[2.0, 4.0, 0.0]; "respect span")]
    #[test_case(&["*", "10"], 0, 2, true, &[6.0, 10.0]; "expand more")]
    fn expand_star(
        columns: &[&str],
        start: usize,
        span: usize,
        expected_expanded: bool,
        expected_column_sizes: &[f64],
    ) {
        let columns: Vec<GridLength> = columns.iter().map(|s| s.parse().unwrap()).collect();
        let expected_column_sizes: Vec<Time> = expected_column_sizes
            .iter()
            .map(|s| Time::new(*s).unwrap())
            .collect();
        let mut helper = Helper::new(&columns);
        let span = NormalizedSpan { start, span };

        let expanded = helper.expand_span_by_star_ratio(span, Time::new(6.0).unwrap());

        assert_eq!(expanded, expected_expanded);
        assert_eq!(helper.column_sizes, expected_column_sizes);
    }

    #[test]
    fn expand_star_towards_ratio() {
        let columns = ["*", "2*", "3*"].map(|s| s.parse().unwrap());
        let column_sizes = [0.0, 0.0, 9.0].map(|s| Time::new(s).unwrap()).to_vec();

        let mut helper = Helper::new_with_column_sizes(&columns, column_sizes);
        let span = NormalizedSpan { start: 0, span: 3 };

        let expanded = helper.expand_span_by_star_ratio(span, Time::new(6.0).unwrap());

        assert!(expanded);
        assert_eq!(
            helper.column_sizes,
            [2.0, 4.0, 9.0].map(|s| Time::new(s).unwrap())
        );
    }

    #[test_case(&["auto", "auto", "auto"], 0, 3, true, &[2.0, 2.0, 2.0]; "increase by count")]
    #[test_case(&["*", "1"], 0, 2, false, &[0.0, 1.0]; "ignore star and fixed")]
    #[test_case(&["auto", "auto", "auto"], 0, 2, true, &[3.0, 3.0, 0.0]; "respect span")]
    #[test_case(&["auto", "10"], 0, 2, true, &[6.0, 10.0]; "expand more")]
    fn expand_auto(
        columns: &[&str],
        start: usize,
        span: usize,
        expected_expanded: bool,
        expected_column_sizes: &[f64],
    ) {
        let columns: Vec<GridLength> = columns.iter().map(|s| s.parse().unwrap()).collect();
        let expected_column_sizes: Vec<Time> = expected_column_sizes
            .iter()
            .map(|s| Time::new(*s).unwrap())
            .collect();
        let mut helper = Helper::new(&columns);
        let span = NormalizedSpan { start, span };

        let expanded = helper.expand_span_by_auto_count(span, Time::new(6.0).unwrap());

        assert_eq!(expanded, expected_expanded);
        assert_eq!(helper.column_sizes, expected_column_sizes);
    }

    #[test_case(&["auto", "*", "auto"], 0, 3, true, &[0.0, 6.0, 0.0]; "expand star first")]
    #[test_case(&["1", "auto"], 0, 2, true, &[1.0, 5.0]; "expand auto if no star")]
    #[test_case(&["7", "*", "auto"], 0, 3, true, &[7.0, 0.0, 0.0]; "already fit")]
    #[test_case(&["1", "1", "1"], 0, 3, false, &[1.0, 1.0, 1.0]; "cannot expand")]
    #[test_case(&["7", "*", "auto"], 1, 2, true, &[7.0, 6.0, 0.0]; "respect span")]
    fn expand_fit(
        columns: &[&str],
        start: usize,
        span: usize,
        expected_expanded_or_fit: bool,
        expected_column_sizes: &[f64],
    ) {
        let columns: Vec<GridLength> = columns.iter().map(|s| s.parse().unwrap()).collect();
        let expected_column_sizes: Vec<Time> = expected_column_sizes
            .iter()
            .map(|s| Time::new(*s).unwrap())
            .collect();
        let mut helper = Helper::new(&columns);
        let span = NormalizedSpan { start, span };

        let expanded_or_fit = helper.expand_span_to_fit(span, Time::new(6.0).unwrap());

        assert_eq!(expanded_or_fit, expected_expanded_or_fit);
        assert_eq!(helper.column_sizes, expected_column_sizes);
    }
}
