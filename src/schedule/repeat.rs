use std::sync::OnceLock;

use anyhow::{bail, Result};

use crate::{
    quant::{ChannelId, Time},
    schedule::{ElementRef, Measure, Visit, Visitor},
};

use super::{Arrange, Arranged};

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

impl Visit for Repeat {
    fn visit<V>(&self, visitor: &mut V, time: Time, duration: Time) -> Result<()>
    where
        V: Visitor,
    {
        visitor.visit_repeat(self, time, duration)?;
        let child_duration = self.child.measure();
        let offset_per_repeat = child_duration + self.spacing;
        for i in 0..self.count {
            let offset = offset_per_repeat * i as f64;
            self.child.visit(visitor, time + offset, child_duration)?;
        }
        Ok(())
    }
}

impl<'a> Arrange<'a> for Repeat {
    fn arrange(
        &'a self,
        time: Time,
        _duration: Time,
    ) -> impl Iterator<Item = Arranged<&'a ElementRef>> {
        let child_duration = self.child.measure();
        let offset_per_repeat = child_duration + self.spacing;
        (0..self.count).map(move |i| {
            let offset = offset_per_repeat * i as f64;
            Arranged {
                item: &self.child,
                offset: time + offset,
                duration: child_duration,
            }
        })
    }
}
