mod helper;

use std::sync::OnceLock;

use crate::{
    quant::{ChannelId, Time},
    schedule::{merge_channel_ids, stack::helper::Helper, Arranged, ElementRef, Measure},
    Direction,
};

use super::{Arrange, TimeRange};

#[derive(Debug, Clone)]
pub(crate) struct Stack {
    children: Vec<ElementRef>,
    direction: Direction,
    channel_ids: Vec<ChannelId>,
    measure_result: OnceLock<MeasureResult>,
}

#[derive(Debug, Clone)]
struct MeasureResult {
    total_duration: Time,
    child_timings: Vec<TimeRange>,
}

impl Stack {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn with_direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self.measure_result.take();
        self
    }

    pub(crate) fn with_children(mut self, children: Vec<ElementRef>) -> Self {
        let channel_ids = merge_channel_ids(children.iter().map(|e| e.channels()));
        self.children = children;
        self.channel_ids = channel_ids;
        self.measure_result.take();
        self
    }

    pub(crate) fn direction(&self) -> Direction {
        self.direction
    }

    fn measure_result(&self) -> &MeasureResult {
        self.measure_result
            .get_or_init(|| measure_stack(&self.children, &self.channel_ids, self.direction))
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self {
            children: vec![],
            direction: Direction::Backward,
            channel_ids: vec![],
            measure_result: OnceLock::new(),
        }
    }
}

impl Measure for Stack {
    fn measure(&self) -> Time {
        let MeasureResult { total_duration, .. } = self.measure_result();
        *total_duration
    }

    fn channels(&self) -> &[ChannelId] {
        &self.channel_ids
    }
}

impl Arrange for Stack {
    fn arrange(&self, time_range: TimeRange) -> impl Iterator<Item = Arranged<&ElementRef>> {
        let MeasureResult { child_timings, .. } = self.measure_result();
        self.children.iter().zip(child_timings).map(
            move |(
                item,
                &TimeRange {
                    start: child_start,
                    span: child_span,
                },
            )| {
                let final_start = match self.direction {
                    Direction::Forward => time_range.start + child_start,
                    Direction::Backward => {
                        time_range.start + time_range.span - child_start - child_span
                    }
                };
                let child_time_range = TimeRange {
                    start: final_start,
                    span: child_span,
                };
                Arranged {
                    item,
                    time_range: child_time_range,
                }
            },
        )
    }
}

fn measure_stack<I>(children: I, channels: &[ChannelId], direction: Direction) -> MeasureResult
where
    I: IntoIterator,
    I::IntoIter: DoubleEndedIterator,
    I::Item: Measure,
{
    let mut helper = Helper::new(channels);
    let child_timings = map_and_collect_by_direction(children, direction, |child| {
        let child_channels = child.channels();
        let span = child.measure();
        let start = helper.get_usage(child_channels);
        helper.update_usage(start + span, child_channels);
        TimeRange { start, span }
    });
    MeasureResult {
        total_duration: helper.into_max_usage(),
        child_timings,
    }
}

/// Map by direction but collect in the original order.
fn map_and_collect_by_direction<I, F, T>(source: I, direction: Direction, f: F) -> Vec<T>
where
    I: IntoIterator,
    I::IntoIter: DoubleEndedIterator,
    F: FnMut(I::Item) -> T,
{
    let mut ret: Vec<_> = match direction {
        Direction::Forward => source.into_iter().map(f).collect(),
        Direction::Backward => source.into_iter().rev().map(f).collect(),
    };
    if direction == Direction::Backward {
        ret.reverse();
    }
    ret
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;
    use crate::schedule::MockMeasure;

    #[test_case(Direction::Forward; "forward")]
    #[test_case(Direction::Backward; "backward")]

    fn collect_by_direction(direction: Direction) {
        let v = [1, 2, 3];

        let mut count = 0;
        let res = map_and_collect_by_direction(&v, direction, |&i| {
            match direction {
                Direction::Forward => assert_eq!(i, v[count]),
                Direction::Backward => assert_eq!(i, v[v.len() - 1 - count]),
            }
            count += 1;
            i
        });

        assert_eq!(res, v);
    }

    #[test_case(Direction::Forward, &[0.0, 10.0, 30.0]; "forward")]
    #[test_case(Direction::Backward, &[50.0, 30.0, 0.0]; "backward")]
    fn test_measure_no_channels(direction: Direction, offsets: &[f64]) {
        let children = [10.0, 20.0, 30.0].map(|duration| {
            let mut mock = MockMeasure::new();
            mock.expect_measure()
                .return_const(Time::new(duration).unwrap());
            mock.expect_channels().return_const(vec![]);
            mock
        });

        let MeasureResult {
            total_duration,
            child_timings,
        } = measure_stack(children, &[], direction);

        assert_eq!(total_duration, Time::new(60.0).unwrap());
        assert_eq!(
            child_timings
                .into_iter()
                .map(|TimeRange { start, .. }| start)
                .collect::<Vec<_>>(),
            offsets
                .iter()
                .map(|&x| Time::new(x).unwrap())
                .collect::<Vec<_>>()
        );
    }

    /// Test case diagram:
    ///
    /// ```text
    ///            +----+   +----+   +----+
    /// ch[0] -----| 10 |---|    |---| 20 |-----
    ///            +----+   |    |   +----+
    ///                     | 20 |
    ///            +----+   |    |   +----+
    /// ch[1] -----| 20 |---|    |---| 10 |-----
    ///            +----+   +----+   +----+
    /// ```
    #[test_case(Direction::Forward, &[0.0, 0.0, 20.0, 40.0, 40.0]; "forward")]
    #[test_case(Direction::Backward, &[40.0, 40.0, 20.0, 0.0, 0.0]; "backward")]
    fn test_measure_with_channels(direction: Direction, offsets: &[f64]) {
        let children = [
            create_mock(10.0, &[0]),
            create_mock(20.0, &[1]),
            create_mock(20.0, &[0, 1]),
            create_mock(20.0, &[0]),
            create_mock(10.0, &[1]),
        ];
        let channels = (0..2).map(create_channel).collect::<Vec<_>>();

        let MeasureResult {
            total_duration,
            child_timings,
        } = measure_stack(children, &channels, direction);

        assert_eq!(total_duration, Time::new(60.0).unwrap());
        assert_eq!(
            child_timings
                .into_iter()
                .map(|TimeRange { start, .. }| start.value())
                .collect::<Vec<_>>(),
            offsets
        );

        fn create_channel(i: usize) -> ChannelId {
            ChannelId::new(i.to_string())
        }
        fn create_mock(duration: f64, channels: &[usize]) -> MockMeasure {
            let mut mock = MockMeasure::new();
            mock.expect_measure()
                .return_const(Time::new(duration).unwrap());
            mock.expect_channels()
                .return_const(channels.iter().copied().map(create_channel).collect());
            mock
        }
    }
}
