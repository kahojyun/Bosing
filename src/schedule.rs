use std::sync::Arc;

use anyhow::{bail, Result};
use hashbrown::HashSet;

use crate::{
    quant::{ChannelId, Time},
    Alignment,
};
pub use absolute::{Absolute, AbsoluteEntry};
pub use grid::{Grid, GridEntry};
pub use play::Play;
pub use repeat::Repeat;
pub use simple::{Barrier, SetFreq, SetPhase, ShiftFreq, ShiftPhase, SwapPhase};
pub use stack::Stack;

mod absolute;
mod grid;
mod play;
mod repeat;
mod simple;
mod stack;

pub type ElementRef = Arc<Element>;

#[derive(Debug, Clone)]
pub struct Element {
    pub common: ElementCommon,
    pub variant: ElementVariant,
}

impl Element {
    pub fn new(common: ElementCommon, variant: impl Into<ElementVariant>) -> Self {
        Self {
            common,
            variant: variant.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MeasuredElement {
    element: ElementRef,
    /// Desired duration without clipping. Doesn't include margin.
    unclipped_duration: Time,
    /// Clipped desired duration. Used by scheduling system.
    duration: Time,
    data: MeasureResultVariant,
}

impl MeasuredElement {
    pub fn duration(&self) -> Time {
        self.duration
    }
}

#[derive(Debug, Clone)]
pub struct ArrangedElement {
    element: ElementRef,
    /// Start time of the inner block without margin relative to its parent.
    inner_time: Time,
    /// Duration of the inner block without margin.
    inner_duration: Time,
    data: ArrangeResultVariant,
}

impl ArrangedElement {
    pub fn inner_time(&self) -> Time {
        self.inner_time
    }

    pub fn inner_duration(&self) -> Time {
        self.inner_duration
    }

    pub fn element(&self) -> &ElementRef {
        &self.element
    }

    pub fn try_get_children(&self) -> Option<&[ArrangedElement]> {
        match &self.data {
            ArrangeResultVariant::Multiple(v) => Some(v),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScheduleOptions {
    pub time_tolerance: Time,
    pub allow_oversize: bool,
}

#[derive(Debug, Clone)]
enum MeasureResultVariant {
    Simple,
    Multiple(Vec<MeasuredElement>),
    Grid(Vec<MeasuredElement>, Vec<Time>),
}

#[derive(Debug, Clone)]
struct MeasureResult(Time, MeasureResultVariant);

#[derive(Debug, Clone)]
pub enum ArrangeResultVariant {
    Simple,
    Multiple(Vec<ArrangedElement>),
}

#[derive(Debug, Clone)]
struct ArrangeResult(Time, ArrangeResultVariant);

#[derive(Debug, Clone)]
struct ArrangeContext<'a> {
    final_duration: Time,
    options: &'a ScheduleOptions,
    measured_self: &'a MeasuredElement,
}

trait Schedule {
    /// Measure the element and return desired inner size and measured children.
    fn measure(&self) -> MeasureResult;
    /// Arrange the element and return final inner size and arranged children.
    fn arrange(&self, context: &ArrangeContext) -> Result<ArrangeResult>;
    /// Channels used by this element. Empty means all of parent's channels.
    fn channels(&self) -> &[ChannelId];
}

fn clamp_duration(duration: Time, min_duration: Time, max_duration: Time) -> Time {
    duration.min(max_duration).max(min_duration)
}

pub fn measure(element: ElementRef) -> MeasuredElement {
    let common = &element.common;
    let total_margin = common.total_margin();
    assert!(total_margin.value().is_finite());
    let (min_duration, max_duration) = common.clamp_min_max_duration();
    let result = element.variant.measure();
    let unclipped_duration = (result.0 + total_margin).max(Time::ZERO);
    let duration = clamp_duration(unclipped_duration, min_duration, max_duration) + total_margin;
    let duration = duration.max(Time::ZERO);
    MeasuredElement {
        element,
        unclipped_duration,
        duration,
        data: result.1,
    }
}

pub fn arrange(
    measured: &MeasuredElement,
    time: Time,
    duration: Time,
    options: &ScheduleOptions,
) -> Result<ArrangedElement> {
    let MeasuredElement {
        element,
        unclipped_duration,
        ..
    } = measured;
    let common = &element.common;
    if duration < unclipped_duration - options.time_tolerance && !options.allow_oversize {
        bail!(
            "Oversizing is configured to be disallowed: available duration {:?} < measured duration {:?}",
            duration,
            unclipped_duration
        );
    }
    let inner_time = time + common.margin.0;
    assert!(inner_time.value().is_finite());
    let (min_duration, max_duration) = common.clamp_min_max_duration();
    let total_margin = common.total_margin();
    let inner_duration = (duration - total_margin).max(Time::ZERO);
    let inner_duration = clamp_duration(inner_duration, min_duration, max_duration);
    if inner_duration + total_margin < unclipped_duration - options.time_tolerance
        && !options.allow_oversize
    {
        bail!(
            "Oversizing is configured to be disallowed: user requested duration {:?} < measured duration {:?}",
            inner_duration + total_margin,
            unclipped_duration
        );
    }
    let result = element.variant.arrange(&ArrangeContext {
        final_duration: inner_duration,
        options,
        measured_self: measured,
    })?;
    Ok(ArrangedElement {
        element: element.clone(),
        inner_time,
        inner_duration: result.0,
        data: result.1,
    })
}

#[derive(Debug, Clone)]
pub struct ElementCommon {
    margin: (Time, Time),
    alignment: Alignment,
    phantom: bool,
    duration: Option<Time>,
    max_duration: Time,
    min_duration: Time,
}

impl ElementCommon {
    pub fn margin(&self) -> (Time, Time) {
        self.margin
    }

    pub fn alignment(&self) -> Alignment {
        self.alignment
    }

    pub fn phantom(&self) -> bool {
        self.phantom
    }

    pub fn duration(&self) -> Option<Time> {
        self.duration
    }

    pub fn max_duration(&self) -> Time {
        self.max_duration
    }

    pub fn min_duration(&self) -> Time {
        self.min_duration
    }

    fn clamp_min_max_duration(&self) -> (Time, Time) {
        let max_duration = clamp_duration(
            self.duration.unwrap_or(Time::INFINITY),
            self.min_duration,
            self.max_duration,
        );
        let min_duration = clamp_duration(
            self.duration.unwrap_or(Time::ZERO),
            self.min_duration,
            self.max_duration,
        );
        (min_duration, max_duration)
    }

    fn total_margin(&self) -> Time {
        self.margin.0 + self.margin.1
    }
}

#[derive(Debug, Clone)]
pub struct ElementCommonBuilder(ElementCommon);

impl Default for ElementCommonBuilder {
    fn default() -> Self {
        Self(ElementCommon {
            margin: Default::default(),
            alignment: Alignment::End,
            phantom: false,
            duration: None,
            max_duration: Time::INFINITY,
            min_duration: Default::default(),
        })
    }
}

impl ElementCommonBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn margin(&mut self, margin: (Time, Time)) -> &mut Self {
        self.0.margin = margin;
        self
    }

    pub fn alignment(&mut self, alignment: Alignment) -> &mut Self {
        self.0.alignment = alignment;
        self
    }

    pub fn phantom(&mut self, phantom: bool) -> &mut Self {
        self.0.phantom = phantom;
        self
    }

    pub fn duration(&mut self, duration: Option<Time>) -> &mut Self {
        self.0.duration = duration;
        self
    }

    pub fn max_duration(&mut self, max_duration: Time) -> &mut Self {
        self.0.max_duration = max_duration;
        self
    }

    pub fn min_duration(&mut self, min_duration: Time) -> &mut Self {
        self.0.min_duration = min_duration;
        self
    }

    pub fn validate(&self) -> Result<()> {
        let v = &self.0;
        if !v.margin.0.value().is_finite() || !v.margin.1.value().is_finite() {
            bail!("Invalid margin {:?}", v.margin);
        }
        if let Some(v) = v.duration {
            if !v.value().is_finite() || v.value() < 0.0 {
                bail!("Invalid duration {:?}", v);
            }
        }
        if !v.min_duration.value().is_finite() || v.min_duration.value() < 0.0 {
            bail!("Invalid min_duration {:?}", v.min_duration);
        }
        if v.max_duration.value() < 0.0 {
            bail!("Invalid max_duration {:?}", v.max_duration);
        }
        Ok(())
    }

    pub fn build(&self) -> Result<ElementCommon> {
        self.validate()?;
        Ok(self.0.clone())
    }
}

macro_rules! impl_variant {
    ($($variant:ident),*$(,)?) => {
#[derive(Debug, Clone)]
pub enum ElementVariant {
    $($variant($variant),)*
}

$(
impl From<$variant> for ElementVariant {
    fn from(v: $variant) -> Self {
        Self::$variant(v)
    }
}

impl TryFrom<ElementVariant> for $variant {
    type Error = anyhow::Error;

    fn try_from(value: ElementVariant) -> Result<Self, Self::Error> {
        match value {
            ElementVariant::$variant(v) => Ok(v),
            _ => bail!("Expected {} variant", stringify!($variant)),
        }
    }
}

impl<'a> TryFrom<&'a ElementVariant> for &'a $variant {
    type Error = anyhow::Error;

    fn try_from(value: &'a ElementVariant) -> Result<Self, Self::Error> {
        match value {
            ElementVariant::$variant(v) => Ok(v),
            _ => bail!("Expected {} variant", stringify!($variant)),
        }
    }
}
)*

impl Schedule for ElementVariant {
    fn measure(&self) -> MeasureResult {
        match self {
            $(ElementVariant::$variant(v) => v.measure(),)*
        }
    }

    fn arrange(&self, context: &ArrangeContext) -> Result<ArrangeResult> {
        match self {
            $(ElementVariant::$variant(v) => v.arrange(context),)*
        }
    }

    fn channels(&self) -> &[ChannelId] {
        match self {
            $(ElementVariant::$variant(v) => v.channels(),)*
        }
    }
}
    };
}

impl_variant!(
    Play, ShiftPhase, SetPhase, ShiftFreq, SetFreq, SwapPhase, Barrier, Repeat, Stack, Absolute,
    Grid,
);

fn merge_channel_ids<'a, I>(ids: I) -> Vec<ChannelId>
where
    I: IntoIterator,
    I::Item: IntoIterator<Item = &'a ChannelId>,
{
    let set = ids.into_iter().flatten().collect::<HashSet<_>>();
    set.into_iter().cloned().collect()
}
