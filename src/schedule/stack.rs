use anyhow::{Ok, Result};
use itertools::Either;

use crate::schedule::{arrange, measure};

use super::{
    ArrangeResult, Element, ElementCommon, MeasureResult, MeasuredElement, Schedule,
    ScheduleOptions,
};

use std::{collections::HashMap, rc::Rc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Backward,
    Forward,
}

#[derive(Debug, Clone)]
pub struct Stack {
    children: Vec<Rc<Element>>,
    direction: Direction,
    channel_ids: Vec<usize>,
}

impl Schedule for Stack {
    fn measure(&self, common: &ElementCommon, max_duration: f64) -> MeasureResult {
        let mut used_duration = if self.channel_ids.is_empty() {
            Either::Left(0.0)
        } else {
            Either::Right(HashMap::<usize, f64>::new())
        };
        let mut measured_children = vec![];
        let it = match self.direction {
            Direction::Forward => Either::Left(self.children.iter()),
            Direction::Backward => Either::Right(self.children.iter().rev()),
        };
        for child in it {
            let child_channels = child.variant.channels();
            let channel_used_duration = get_channel_usage(&used_duration, child_channels);
            let child_available_duration = max_duration - channel_used_duration;
            let measured_child = measure(child.clone(), child_available_duration);
            let channel_used_duration = channel_used_duration + measured_child.duration;
            measured_children.push(measured_child);
            let channels = if child_channels.is_empty() {
                self.channels()
            } else {
                child_channels
            };
            update_channel_usage(&mut used_duration, channel_used_duration, channels);
        }
        let total_used_duration = match used_duration {
            Either::Left(v) => v,
            Either::Right(d) => d
                .into_values()
                .max_by(|a, b| a.total_cmp(b))
                .unwrap_or_default(),
        };
        (total_used_duration, measured_children)
    }

    fn arrange(
        &self,
        common: &ElementCommon,
        measured_children: &[MeasuredElement],
        time: f64,
        final_duration: f64,
        options: &ScheduleOptions,
    ) -> Result<ArrangeResult> {
        let mut used_duration = if self.channel_ids.is_empty() {
            Either::Left(0.0)
        } else {
            Either::Right(HashMap::<usize, f64>::new())
        };
        let mut arranged_children = vec![];
        let it = match self.direction {
            Direction::Forward => Either::Left(measured_children.iter()),
            Direction::Backward => Either::Right(measured_children.iter().rev()),
        };
        for child in it {
            let child_channels = child.element.variant.channels();
            let channel_used_duration = get_channel_usage(&used_duration, child_channels);
            let measured_duration = child.duration;
            let inner_time = match self.direction {
                Direction::Forward => channel_used_duration,
                Direction::Backward => final_duration - channel_used_duration - measured_duration,
            };
            let arranged_child = arrange(child, inner_time, measured_duration, options)?;
            let channel_used_duration = channel_used_duration + measured_duration;
            arranged_children.push(arranged_child);
            let channels = if child_channels.is_empty() {
                self.channels()
            } else {
                child_channels
            };
            update_channel_usage(&mut used_duration, channel_used_duration, channels);
        }
        Ok((final_duration, arranged_children))
    }

    fn channels(&self) -> &[usize] {
        &self.channel_ids
    }
}

fn update_channel_usage(
    used_duration: &mut Either<f64, HashMap<usize, f64>>,
    new_duration: f64,
    channels: &[usize],
) {
    match used_duration {
        Either::Left(v) => *v = new_duration,
        Either::Right(d) => {
            for &ch in channels {
                d.insert(ch, new_duration);
            }
        }
    }
}

fn get_channel_usage(used_duration: &Either<f64, HashMap<usize, f64>>, channels: &[usize]) -> f64 {
    match used_duration {
        Either::Left(v) => *v,
        Either::Right(d) => (if channels.is_empty() {
            d.values().max_by(|a, b| a.total_cmp(b)).copied()
        } else {
            channels
                .iter()
                .map(|i| d.get(i).copied().unwrap_or_default())
                .max_by(|a, b| a.total_cmp(b))
        })
        .unwrap_or_default()
        .into(),
    }
}
