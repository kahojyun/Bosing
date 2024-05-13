use std::sync::OnceLock;

use anyhow::{bail, Result};

use crate::{
    quant::{ChannelId, Time},
    schedule::{merge_channel_ids, ElementRef, Measure},
};

use super::{Arrange, Arranged, TimeRange};

#[derive(Debug, Clone)]
pub(crate) struct AbsoluteEntry {
    time: Time,
    element: ElementRef,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Absolute {
    children: Vec<AbsoluteEntry>,
    channel_ids: Vec<ChannelId>,
    measure_result: OnceLock<Time>,
}

impl AbsoluteEntry {
    pub(crate) fn new(element: ElementRef) -> Self {
        Self {
            time: Time::ZERO,
            element,
        }
    }

    pub(crate) fn with_time(mut self, time: Time) -> Result<Self> {
        if !time.value().is_finite() {
            bail!("Invalid time {:?}", time);
        }
        self.time = time;
        Ok(self)
    }
}

impl Absolute {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn with_children(mut self, children: Vec<AbsoluteEntry>) -> Self {
        let channel_ids = merge_channel_ids(children.iter().map(|e| e.element.variant.channels()));
        self.children = children;
        self.channel_ids = channel_ids;
        self
    }

    fn measure_result(&self) -> &Time {
        self.measure_result
            .get_or_init(|| measure_absolute(self.children.iter().map(|e| (&e.element, e.time))))
    }
}

impl Measure for Absolute {
    fn measure(&self) -> Time {
        *self.measure_result()
    }

    fn channels(&self) -> &[ChannelId] {
        &self.channel_ids
    }
}

impl Arrange for Absolute {
    fn arrange(&self, time_range: TimeRange) -> impl Iterator<Item = Arranged<&ElementRef>> {
        self.children.iter().map(
            move |AbsoluteEntry {
                      time: offset,
                      element,
                  }| {
                Arranged {
                    item: element,
                    time_range: TimeRange {
                        start: time_range.start + offset,
                        span: element.measure(),
                    },
                }
            },
        )
    }
}

fn measure_absolute<I, M>(children: I) -> Time
where
    I: IntoIterator<Item = (M, Time)>,
    M: Measure,
{
    children
        .into_iter()
        .map(|(child, offset)| offset + child.measure())
        .max()
        .unwrap_or(Time::ZERO)
}
