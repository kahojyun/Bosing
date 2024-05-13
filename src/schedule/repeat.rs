use std::sync::OnceLock;

use anyhow::{bail, Result};

use crate::{
    quant::{ChannelId, Time},
    schedule::{ElementRef, Measure, Visit, Visitor},
};

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

pub(crate) fn walk_repeat<V>(variant: &Repeat, visitor: &mut V, time: Time) -> Result<(), V::Error>
where
    V: Visitor + ?Sized,
{
    let child_duration = variant.child.measure();
    let offset_per_repeat = child_duration + variant.spacing;
    for i in 0..variant.count {
        let offset = offset_per_repeat * i as f64;
        variant
            .child
            .visit(visitor, time + offset, child_duration)?;
    }
    Ok(())
}

impl Visit for Repeat {
    fn visit<V>(&self, visitor: &mut V, time: Time, duration: Time) -> Result<(), V::Error>
    where
        V: Visitor + ?Sized,
    {
        visitor.visit_repeat(self, time, duration)
    }
}
