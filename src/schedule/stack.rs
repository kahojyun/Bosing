use anyhow::Result;
use hashbrown::HashMap;

use super::{
    arrange, measure, merge_channel_ids, ArrangeContext, ArrangeResult, ArrangeResultVariant,
    ElementRef, Measure, MeasureResult, MeasureResultVariant, Schedule,
};
use crate::{
    quant::{ChannelId, Time},
    Direction,
};

#[derive(Debug, Clone)]
pub(crate) struct Stack {
    children: Vec<ElementRef>,
    direction: Direction,
    channel_ids: Vec<ChannelId>,
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

impl Stack {
    pub(crate) fn new() -> Self {
        Self {
            children: vec![],
            direction: Direction::Backward,
            channel_ids: vec![],
        }
    }

    pub(crate) fn with_direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    pub(crate) fn with_children(mut self, children: Vec<ElementRef>) -> Self {
        let channel_ids = merge_channel_ids(children.iter().map(|e| e.variant.channels()));
        self.children = children;
        self.channel_ids = channel_ids;
        self
    }

    pub(crate) fn direction(&self) -> Direction {
        self.direction
    }
}

impl Schedule for Stack {
    fn measure(&self) -> MeasureResult {
        let mut helper = Helper::new(self.channels());
        let measured_children =
            map_and_collect_by_direction(&self.children, self.direction, |child| {
                let child_channels = child.variant.channels();
                let channel_used_duration = helper.get_usage(child_channels);
                let measured_child = measure(child.clone());
                let channel_used_duration = channel_used_duration + measured_child.duration;
                helper.update_usage(channel_used_duration, child_channels);
                Ok(measured_child)
            })
            .unwrap();
        let total_used_duration = helper.into_max_usage();
        MeasureResult(
            total_used_duration,
            MeasureResultVariant::Multiple(measured_children),
        )
    }

    fn arrange(&self, context: &ArrangeContext) -> Result<ArrangeResult> {
        let mut helper = Helper::new(self.channels());
        let measured_children = match &context.measured_self.data {
            MeasureResultVariant::Multiple(v) => v,
            _ => panic!("Invalid measure data"),
        };
        let arranged_children =
            map_and_collect_by_direction(measured_children, self.direction, |child| {
                let child_channels = child.element.variant.channels();
                let channel_used_duration = helper.get_usage(child_channels);
                let measured_duration = child.duration;
                let inner_time = match self.direction {
                    Direction::Forward => channel_used_duration,
                    Direction::Backward => {
                        context.final_duration - channel_used_duration - measured_duration
                    }
                };
                let channel_used_duration = channel_used_duration + measured_duration;
                helper.update_usage(channel_used_duration, child_channels);
                arrange(child, inner_time, measured_duration, context.options)
            })?;
        Ok(ArrangeResult(
            context.final_duration,
            ArrangeResultVariant::Multiple(arranged_children),
        ))
    }

    fn channels(&self) -> &[ChannelId] {
        &self.channel_ids
    }
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

#[derive(Debug)]
enum ChannelUsage {
    Single(Time),
    Multiple(HashMap<ChannelId, Time>),
}

#[derive(Debug)]
struct Helper<'a> {
    all_channels: &'a [ChannelId],
    usage: ChannelUsage,
}

impl<'a> Helper<'a> {
    fn new(all_channels: &'a [ChannelId]) -> Self {
        Self {
            all_channels,
            usage: if all_channels.is_empty() {
                ChannelUsage::Single(Time::ZERO)
            } else {
                ChannelUsage::Multiple(HashMap::with_capacity(all_channels.len()))
            },
        }
    }

    fn get_usage(&self, channels: &[ChannelId]) -> Time {
        match &self.usage {
            ChannelUsage::Single(v) => *v,
            ChannelUsage::Multiple(d) => (if channels.is_empty() {
                d.values().max()
            } else {
                channels.iter().filter_map(|i| d.get(i)).max()
            })
            .copied()
            .unwrap_or_default(),
        }
    }

    fn update_usage(&mut self, new_duration: Time, channels: &[ChannelId]) {
        let channels = if channels.is_empty() {
            self.all_channels
        } else {
            channels
        };
        match &mut self.usage {
            ChannelUsage::Single(v) => *v = new_duration,
            ChannelUsage::Multiple(d) => {
                for ch in channels {
                    d.insert(ch.clone(), new_duration);
                }
            }
        };
    }

    fn into_max_usage(self) -> Time {
        match self.usage {
            ChannelUsage::Single(v) => v,
            ChannelUsage::Multiple(d) => d.into_values().max().unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;
    use crate::schedule::MockMeasure;

    #[test]
    fn test_helper_no_channels() {
        let mut helper = Helper::new(&[]);
        assert_eq!(helper.get_usage(&[]), Time::ZERO);
        let time = Time::new(10.0).unwrap();
        helper.update_usage(time, &[]);
        assert_eq!(helper.get_usage(&[]), time);
        assert_eq!(helper.into_max_usage(), time);
    }

    #[test]
    fn test_helper_with_channels() {
        let channels = (0..5)
            .map(|i| ChannelId::new(i.to_string()))
            .collect::<Vec<_>>();
        let mut helper = Helper::new(&channels);
        assert_eq!(helper.get_usage(&[]), Time::ZERO);
        assert_eq!(helper.get_usage(&[channels[0].clone()]), Time::ZERO);

        let t1 = Time::new(10.0).unwrap();
        helper.update_usage(t1, &[]);
        assert_eq!(helper.get_usage(&[]), t1);
        assert_eq!(helper.get_usage(&[channels[0].clone()]), t1);

        let t2 = Time::new(20.0).unwrap();
        helper.update_usage(t2, &[channels[0].clone()]);
        assert_eq!(helper.get_usage(&[]), t2);
        assert_eq!(helper.get_usage(&[channels[0].clone()]), t2);
        assert_eq!(helper.get_usage(&[channels[1].clone()]), t1);
        assert_eq!(
            helper.get_usage(&[channels[0].clone(), channels[1].clone()]),
            t2
        );
        assert_eq!(helper.into_max_usage(), t2);
    }

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
