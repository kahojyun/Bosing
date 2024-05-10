mod helper;

use std::sync::OnceLock;

use anyhow::Result;

use crate::{
    quant::{ChannelId, Time},
    schedule::{merge_channel_ids, stack::helper::Helper, ElementRef, Measure},
    Direction,
};

#[derive(Debug, Clone)]
pub(crate) struct Stack {
    children: Vec<ElementRef>,
    direction: Direction,
    channel_ids: Vec<ChannelId>,
    measure_result: OnceLock<(Time, Vec<Time>)>,
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

    fn measure_result(&self) -> &(Time, Vec<Time>) {
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
        let (total_duration, _) = self.measure_result();
        *total_duration
    }

    fn channels(&self) -> &[ChannelId] {
        &self.channel_ids
    }
}

fn arrange_stack<I, M>(
    children: I,
    final_duration: Time,
    direction: Direction,
) -> impl IntoIterator<Item = (M, Time, Time)>
where
    I: IntoIterator<Item = (M, Time)>,
    M: Measure,
{
    children.into_iter().map(move |(child, child_offset)| {
        let child_duration = child.measure();
        let final_offset = match direction {
            Direction::Forward => child_offset,
            Direction::Backward => final_duration - child_offset - child_duration,
        };
        (child, final_offset, child_duration)
    })
}

fn measure_stack<I>(children: I, channels: &[ChannelId], direction: Direction) -> (Time, Vec<Time>)
where
    I: IntoIterator,
    I::IntoIter: DoubleEndedIterator,
    I::Item: Measure,
{
    let mut helper = Helper::new(channels);
    let child_offsets = map_and_collect_by_direction(children, direction, |child| {
        let child_channels = child.channels();
        let child_duration = child.measure();
        let child_offset = helper.get_usage(child_channels);
        helper.update_usage(child_offset + child_duration, child_channels);
        Ok(child_offset)
    })
    .unwrap();
    (helper.into_max_usage(), child_offsets)
}

/// Map by direction but collect in the original order.
fn map_and_collect_by_direction<I, F, T>(source: I, direction: Direction, f: F) -> Result<Vec<T>>
where
    I: IntoIterator,
    I::IntoIter: DoubleEndedIterator,
    F: FnMut(I::Item) -> Result<T>,
{
    let mut ret: Vec<_> = match direction {
        Direction::Forward => source.into_iter().map(f).collect::<Result<_>>(),
        Direction::Backward => source.into_iter().rev().map(f).collect(),
    }?;
    if direction == Direction::Backward {
        ret.reverse();
    }
    Ok(ret)
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;
    use crate::schedule::MockMeasure;

    #[test]
    fn test_collect_by_direction() {
        let v = vec![1, 2, 3, 4, 5];
        let mut count = 0;
        let forward = map_and_collect_by_direction(&v, Direction::Forward, |&i| {
            let ret = (count, i);
            count += 1;
            Ok(ret)
        })
        .unwrap();
        assert_eq!(forward, vec![(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)]);
        let mut count = 0;
        let backward = map_and_collect_by_direction(&v, Direction::Backward, |&i| {
            let ret = (count, i);
            count += 1;
            Ok(ret)
        })
        .unwrap();
        assert_eq!(backward, vec![(4, 1), (3, 2), (2, 3), (1, 4), (0, 5)]);
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
        let (total_duration, child_offsets) = measure_stack(children, &[], direction);
        assert_eq!(total_duration, Time::new(60.0).unwrap());
        assert_eq!(
            child_offsets,
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

        let children = [
            create_mock(10.0, &[0]),
            create_mock(20.0, &[1]),
            create_mock(20.0, &[0, 1]),
            create_mock(20.0, &[0]),
            create_mock(10.0, &[1]),
        ];
        let channels = (0..2).map(create_channel).collect::<Vec<_>>();
        let (total_duration, child_offsets) = measure_stack(children, &channels, direction);
        assert_eq!(total_duration, Time::new(60.0).unwrap());
        assert_eq!(
            child_offsets,
            offsets
                .iter()
                .map(|&x| Time::new(x).unwrap())
                .collect::<Vec<_>>()
        );
    }
}
