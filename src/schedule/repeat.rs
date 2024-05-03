use anyhow::{bail, Result};

use super::{
    arrange, measure, ArrangeContext, ArrangeResult, ArrangeResultVariant, ElementRef,
    MeasureContext, MeasureResult, MeasureResultVariant, Schedule,
};
use crate::quant::{ChannelId, Time};

#[derive(Debug, Clone)]
pub struct Repeat {
    child: ElementRef,
    count: usize,
    spacing: Time,
}

impl Repeat {
    pub fn new(child: ElementRef, count: usize) -> Self {
        Self {
            child,
            count,
            spacing: Time::ZERO,
        }
    }

    pub fn with_spacing(mut self, spacing: Time) -> Result<Self> {
        if !spacing.value().is_finite() {
            bail!("Invalid spacing {:?}", spacing);
        }
        self.spacing = spacing;
        Ok(self)
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn spacing(&self) -> Time {
        self.spacing
    }
}

impl Schedule for Repeat {
    fn measure(&self, context: &MeasureContext) -> MeasureResult {
        if self.count == 0 {
            return MeasureResult(Time::ZERO, MeasureResultVariant::Simple);
        }
        let n = self.count as f64;
        let duration_per_repeat = (context.max_duration - self.spacing * (n - 1.0)) / n;
        let measured_child = measure(self.child.clone(), duration_per_repeat);
        let wanted_duration = measured_child.duration * n + self.spacing * (n - 1.0);
        MeasureResult(
            wanted_duration,
            MeasureResultVariant::Multiple(vec![measured_child]),
        )
    }

    fn arrange(&self, context: &ArrangeContext) -> Result<ArrangeResult> {
        if self.count == 0 {
            return Ok(ArrangeResult(Time::ZERO, ArrangeResultVariant::Simple));
        }
        let n = self.count as f64;
        let duration_per_repeat = (context.final_duration - self.spacing * (n - 1.0)) / n;
        let measured_child = match &context.measured_self.data {
            MeasureResultVariant::Multiple(c) if c.len() == 1 => &c[0],
            _ => bail!("Invalid measure data"),
        };
        let arranged_child = arrange(
            measured_child,
            Time::ZERO,
            duration_per_repeat,
            context.options,
        )?;
        let arranged = arranged_child.inner_duration * n + self.spacing * (n - 1.0);
        Ok(ArrangeResult(
            arranged,
            ArrangeResultVariant::Multiple(vec![arranged_child]),
        ))
    }

    fn channels(&self) -> &[ChannelId] {
        self.child.variant.channels()
    }
}
