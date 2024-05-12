mod helper;

use std::sync::OnceLock;

use anyhow::Result;

use crate::{
    quant::{ChannelId, Time},
    schedule::{
        merge_channel_ids, stack::helper::Helper, Arranged, ElementRef, Measure, Visit, Visitor,
    },
    Direction,
};

use super::Arrange;

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
    child_timings: Vec<(Time, Time)>,
}

#[derive(Debug, Clone)]
struct ArrangeItem<T> {
    item: T,
    offset: Time,
    duration: Time,
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

impl Visit for Stack {
    fn visit<V>(&self, visitor: &mut V, time: Time, duration: Time) -> Result<()>
    where
        V: Visitor,
    {
        visitor.visit_stack(self, time, duration)?;
        let MeasureResult { child_timings, .. } = self.measure_result();
        let arranged = arrange_stack(
            self.children
                .iter()
                .zip(child_timings)
                .map(|(c, t)| ArrangeItem {
                    item: c,
                    offset: t.0,
                    duration: t.1,
                }),
            duration,
            self.direction,
        );
        for Arranged {
            item,
            offset,
            duration,
        } in arranged
        {
            item.visit(visitor, time + offset, duration)?;
        }
        Ok(())
    }
}

impl<'a> Arrange<'a> for Stack {
    fn arrange(
        &'a self,
        time: Time,
        duration: Time,
    ) -> impl Iterator<Item = Arranged<&'a ElementRef>> {
        let MeasureResult { child_timings, .. } = self.measure_result();
        self.children.iter().zip(child_timings).map(
            move |(item, &(child_offset, child_duration))| {
                let final_offset = match self.direction {
                    Direction::Forward => child_offset,
                    Direction::Backward => duration - child_offset - child_duration,
                };
                Arranged {
                    item,
                    offset: time + final_offset,
                    duration: child_duration,
                }
            },
        )
    }
}

fn arrange_stack<I, T>(
    children: I,
    final_duration: Time,
    direction: Direction,
) -> impl IntoIterator<Item = Arranged<T>>
where
    I: IntoIterator<Item = ArrangeItem<T>>,
{
    children.into_iter().map(move |child| {
        let final_offset = match direction {
            Direction::Forward => child.offset,
            Direction::Backward => final_duration - child.offset - child.duration,
        };
        Arranged {
            item: child.item,
            offset: final_offset,
            duration: child.duration,
        }
    })
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
        let child_duration = child.measure();
        let child_offset = helper.get_usage(child_channels);
        helper.update_usage(child_offset + child_duration, child_channels);
        Ok((child_offset, child_duration))
    })
    .unwrap();
    MeasureResult {
        total_duration: helper.into_max_usage(),
        child_timings,
    }
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
    use itertools::Itertools;
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
            Ok(i)
        })
        .unwrap();

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
                .map(|(offset, _)| offset)
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
                .map(|(offset, _)| offset.value())
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

    #[test_case(
        Direction::Forward,
        &[0.0, 0.0, 20.0, 40.0, 40.0],
        &[0.0, 0.0, 20.0, 40.0, 40.0];
        "forward"
    )]
    #[test_case(
        Direction::Backward,
        &[40.0, 40.0, 20.0, 0.0, 0.0],
        &[50.0, 40.0, 60.0, 80.0, 90.0];
        "backward"
    )]
    fn test_arrange(direction: Direction, offsets: &[f64], expected_offsets: &[f64]) {
        let children =
            [10.0, 20.0, 20.0, 20.0, 10.0]
                .into_iter()
                .zip(offsets)
                .map(|(duration, offset)| ArrangeItem {
                    item: (),
                    offset: Time::new(*offset).unwrap(),
                    duration: Time::new(duration).unwrap(),
                });

        let res = arrange_stack(children, Time::new(100.0).unwrap(), direction);

        for (item, expected_offset) in res.into_iter().zip_eq(expected_offsets.iter()) {
            assert_eq!(item.offset.value(), *expected_offset);
        }
    }
}
