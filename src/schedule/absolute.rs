use std::sync::OnceLock;

use anyhow::{bail, Result};

use crate::{
    quant::{ChannelId, Time},
    schedule::{merge_channel_ids, ElementRef, Measure},
};

#[derive(Debug, Clone)]
pub(crate) struct AbsoluteEntry {
    time: Time,
    element: ElementRef,
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

#[derive(Debug, Clone, Default)]
pub(crate) struct Absolute {
    children: Vec<AbsoluteEntry>,
    channel_ids: Vec<ChannelId>,
    measure_result: OnceLock<Time>,
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

// impl Schedule for Absolute {
//     fn measure(&self) -> MeasureResult {
//         let mut max_time = Time::ZERO;
//         let mut measured_children = vec![];
//         for e in &self.children {
//             let measured_child = measure(e.element.clone());
//             max_time = max_time.max(e.time + measured_child.duration);
//             measured_children.push(measured_child);
//         }
//         MeasureResult(max_time, MeasureResultVariant::Multiple(measured_children))
//     }

//     fn arrange(&self, context: &ArrangeContext) -> Result<ArrangeResult> {
//         let measured_children = match &context.measured_self.data {
//             MeasureResultVariant::Multiple(v) => v,
//             _ => bail!("Invalid measure data"),
//         };
//         let arranged_children = self
//             .children
//             .iter()
//             .map(|e| e.time)
//             .zip(measured_children.iter())
//             .map(|(t, mc)| arrange(mc, t, mc.duration, context.options))
//             .collect::<Result<_>>()?;
//         Ok(ArrangeResult(
//             context.final_duration,
//             ArrangeResultVariant::Multiple(arranged_children),
//         ))
//     }

//     fn channels(&self) -> &[ChannelId] {
//         &self.channel_ids
//     }
// }
