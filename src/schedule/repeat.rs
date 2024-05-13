use std::sync::OnceLock;

use anyhow::{bail, Result};

use crate::{
    quant::{ChannelId, Time},
    schedule::{ElementRef, Measure},
};

use super::{Arrange, Arranged, TimeRange};

#[derive(Debug, Clone)]
pub(crate) struct Repeat {
    child: ElementRef,
    count: usize,
    spacing: Time,
    measure_result: OnceLock<Time>,
}

impl Repeat {
    pub(crate) fn new(child: ElementRef, count: usize) -> Self {
        Self {
            child,
            count,
            spacing: Time::ZERO,
            measure_result: OnceLock::new(),
        }
    }

    pub(crate) fn with_spacing(mut self, spacing: Time) -> Result<Self> {
        if !spacing.value().is_finite() {
            bail!("Invalid spacing {:?}", spacing);
        }
        self.spacing = spacing;
        self.measure_result.take();
        Ok(self)
    }

    pub(crate) fn count(&self) -> usize {
        self.count
    }

    pub(crate) fn spacing(&self) -> Time {
        self.spacing
    }
}

impl Measure for Repeat {
    fn channels(&self) -> &[ChannelId] {
        self.child.channels()
    }

    fn measure(&self) -> Time {
        if self.count == 0 {
            return Time::ZERO;
        }
        *self.measure_result.get_or_init(|| {
            let n = self.count as f64;
            let child_duration = self.child.measure();
            child_duration * n + self.spacing * (n - 1.0)
        })
    }
}

impl Arrange for Repeat {
    fn arrange(&self, time_range: TimeRange) -> impl Iterator<Item = Arranged<&ElementRef>> {
        let child_duration = self.child.measure();
        let offset_per_repeat = child_duration + self.spacing;
        (0..self.count).map(move |i| {
            let offset = offset_per_repeat * i as f64;
            let child_time_range = TimeRange {
                start: time_range.start + offset,
                span: child_duration,
            };
            Arranged {
                item: &self.child,
                time_range: child_time_range,
            }
        })
    }
}
