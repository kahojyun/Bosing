use anyhow::{bail, Result};
use hashbrown::HashMap;
use itertools::Itertools as _;

use super::{
    arrange, measure, ArrangeContext, ArrangeResult, ArrangeResultVariant, ElementRef,
    MeasureContext, MeasureResult, MeasureResultVariant, Schedule,
};
use crate::{
    quant::{ChannelId, Time},
    Direction,
};

#[derive(Debug, Clone)]
pub struct Stack {
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
    pub fn new() -> Self {
        Self {
            children: vec![],
            direction: Direction::Backward,
            channel_ids: vec![],
        }
    }

    pub fn with_direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    pub fn with_children(mut self, children: Vec<ElementRef>) -> Self {
        let channel_ids = children
            .iter()
            .flat_map(|e| e.variant.channels())
            .unique()
            .cloned()
            .collect();
        self.children = children;
        self.channel_ids = channel_ids;
        self
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }
}

impl Schedule for Stack {
    fn measure(&self, context: &MeasureContext) -> MeasureResult {
        let mut helper = Helper::new(self.channels());
        let measured_children =
            map_and_collect_by_direction(&self.children, self.direction, |child| {
                let child_channels = child.variant.channels();
                let channel_used_duration = helper.get_usage(child_channels);
                let child_available_duration = context.max_duration - channel_used_duration;
                let measured_child = measure(child.clone(), child_available_duration);
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
                ChannelUsage::Multiple(HashMap::new())
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
                d.extend(channels.iter().map(|ch| (ch.clone(), new_duration)))
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
    use super::*;

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
}
