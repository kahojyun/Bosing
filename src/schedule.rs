use std::sync::Arc;

use anyhow::{bail, Result};
use enum_dispatch::enum_dispatch;

use crate::Alignment;
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
    common: ElementCommon,
    variant: ElementVariant,
}

impl Element {
    pub fn new(common: ElementCommon, variant: impl Into<ElementVariant>) -> Self {
        Self {
            common,
            variant: variant.into(),
        }
    }

    pub fn common(&self) -> &ElementCommon {
        &self.common
    }

    pub fn variant(&self) -> &ElementVariant {
        &self.variant
    }

    pub fn try_get_play(&self) -> Option<&Play> {
        match &self.variant {
            ElementVariant::Play(v) => Some(v),
            _ => None,
        }
    }

    pub fn try_get_shift_phase(&self) -> Option<&ShiftPhase> {
        match &self.variant {
            ElementVariant::ShiftPhase(v) => Some(v),
            _ => None,
        }
    }

    pub fn try_get_set_phase(&self) -> Option<&SetPhase> {
        match &self.variant {
            ElementVariant::SetPhase(v) => Some(v),
            _ => None,
        }
    }

    pub fn try_get_shift_freq(&self) -> Option<&ShiftFreq> {
        match &self.variant {
            ElementVariant::ShiftFreq(v) => Some(v),
            _ => None,
        }
    }

    pub fn try_get_set_freq(&self) -> Option<&SetFreq> {
        match &self.variant {
            ElementVariant::SetFreq(v) => Some(v),
            _ => None,
        }
    }

    pub fn try_get_swap_phase(&self) -> Option<&SwapPhase> {
        match &self.variant {
            ElementVariant::SwapPhase(v) => Some(v),
            _ => None,
        }
    }

    pub fn try_get_barrier(&self) -> Option<&Barrier> {
        match &self.variant {
            ElementVariant::Barrier(v) => Some(v),
            _ => None,
        }
    }

    pub fn try_get_repeat(&self) -> Option<&Repeat> {
        match &self.variant {
            ElementVariant::Repeat(v) => Some(v),
            _ => None,
        }
    }

    pub fn try_get_stack(&self) -> Option<&Stack> {
        match &self.variant {
            ElementVariant::Stack(v) => Some(v),
            _ => None,
        }
    }

    pub fn try_get_absolute(&self) -> Option<&Absolute> {
        match &self.variant {
            ElementVariant::Absolute(v) => Some(v),
            _ => None,
        }
    }

    pub fn try_get_grid(&self) -> Option<&Grid> {
        match &self.variant {
            ElementVariant::Grid(v) => Some(v),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MeasuredElement {
    element: ElementRef,
    /// Desired duration without clipping. Doesn't include margin.
    unclipped_duration: f64,
    /// Clipped desired duration. Used by scheduling system.
    duration: f64,
    data: MeasureResultVariant,
}

impl MeasuredElement {
    pub fn duration(&self) -> f64 {
        self.duration
    }
}

#[derive(Debug, Clone)]
pub struct ArrangedElement {
    element: ElementRef,
    /// Start time of the inner block without margin relative to its parent.
    inner_time: f64,
    /// Duration of the inner block without margin.
    inner_duration: f64,
    data: ArrangeResultVariant,
}

impl ArrangedElement {
    pub fn inner_time(&self) -> f64 {
        self.inner_time
    }

    pub fn inner_duration(&self) -> f64 {
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
    pub time_tolerance: f64,
    pub allow_oversize: bool,
}

#[derive(Debug, Clone)]
enum MeasureResultVariant {
    Simple,
    Multiple(Vec<MeasuredElement>),
    Grid(Vec<MeasuredElement>, Vec<f64>),
}

#[derive(Debug, Clone)]
struct MeasureResult(f64, MeasureResultVariant);

#[derive(Debug, Clone)]
pub enum ArrangeResultVariant {
    Simple,
    Multiple(Vec<ArrangedElement>),
}

#[derive(Debug, Clone)]
struct ArrangeResult(f64, ArrangeResultVariant);

#[derive(Debug, Clone)]
struct MeasureContext {
    max_duration: f64,
}

#[derive(Debug, Clone)]
struct ArrangeContext<'a> {
    final_duration: f64,
    options: &'a ScheduleOptions,
    measured_self: &'a MeasuredElement,
}

#[enum_dispatch]
trait Schedule {
    /// Measure the element and return desired inner size and measured children.
    fn measure(&self, context: &MeasureContext) -> MeasureResult;
    /// Arrange the element and return final inner size and arranged children.
    fn arrange(&self, context: &ArrangeContext) -> Result<ArrangeResult>;
    /// Channels used by this element. Empty means all of parent's channels.
    fn channels(&self) -> &[String];
}

fn clamp_duration(duration: f64, min_duration: f64, max_duration: f64) -> f64 {
    duration.min(max_duration).max(min_duration)
}

pub fn measure(element: ElementRef, available_duration: f64) -> MeasuredElement {
    assert!(available_duration >= 0.0 || available_duration.is_infinite());
    let common = &element.common;
    let total_margin = common.margin.0 + common.margin.1;
    assert!(total_margin.is_finite());
    let max_duration = clamp_duration(
        common.duration.unwrap_or(f64::INFINITY),
        common.min_duration,
        common.max_duration,
    );
    let min_duration = clamp_duration(
        common.duration.unwrap_or(0.0),
        common.min_duration,
        common.max_duration,
    );
    let inner_duration = (available_duration - total_margin).max(0.0);
    let inner_duration = clamp_duration(inner_duration, min_duration, max_duration);
    let result = element.variant.measure(&MeasureContext {
        max_duration: inner_duration,
    });
    let unclipped_duration = (result.0 + total_margin).max(0.0);
    let duration = clamp_duration(unclipped_duration, min_duration, max_duration) + total_margin;
    let duration = clamp_duration(duration, 0.0, available_duration);
    MeasuredElement {
        element,
        unclipped_duration,
        duration,
        data: result.1,
    }
}

pub fn arrange(
    measured: &MeasuredElement,
    time: f64,
    duration: f64,
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
            "Oversizing is configured to be disallowed: available duration {} < measured duration {}",
            duration,
            unclipped_duration
        );
    }
    let inner_time = time + common.margin.0;
    assert!(inner_time.is_finite());
    let max_duration = clamp_duration(
        common.duration.unwrap_or(f64::INFINITY),
        common.min_duration,
        common.max_duration,
    );
    let min_duration = clamp_duration(
        common.duration.unwrap_or(0.0),
        common.min_duration,
        common.max_duration,
    );
    let total_margin = common.margin.0 + common.margin.1;
    let inner_duration = (duration - total_margin).max(0.0);
    let inner_duration = clamp_duration(inner_duration, min_duration, max_duration);
    if inner_duration + total_margin < unclipped_duration - options.time_tolerance
        && !options.allow_oversize
    {
        bail!(
            "Oversizing is configured to be disallowed: user requested duration {} < measured duration {}",
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
    margin: (f64, f64),
    alignment: Alignment,
    phantom: bool,
    duration: Option<f64>,
    max_duration: f64,
    min_duration: f64,
}

impl ElementCommon {
    pub fn margin(&self) -> (f64, f64) {
        self.margin
    }

    pub fn alignment(&self) -> Alignment {
        self.alignment
    }

    pub fn phantom(&self) -> bool {
        self.phantom
    }

    pub fn duration(&self) -> Option<f64> {
        self.duration
    }

    pub fn max_duration(&self) -> f64 {
        self.max_duration
    }

    pub fn min_duration(&self) -> f64 {
        self.min_duration
    }
}

#[derive(Debug, Clone)]
pub struct ElementCommonBuilder(ElementCommon);

impl Default for ElementCommonBuilder {
    fn default() -> Self {
        Self(ElementCommon {
            margin: (0.0, 0.0),
            alignment: Alignment::End,
            phantom: false,
            duration: None,
            max_duration: f64::INFINITY,
            min_duration: 0.0,
        })
    }
}

impl ElementCommonBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn margin(&mut self, margin: (f64, f64)) -> &mut Self {
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

    pub fn duration(&mut self, duration: Option<f64>) -> &mut Self {
        self.0.duration = duration;
        self
    }

    pub fn max_duration(&mut self, max_duration: f64) -> &mut Self {
        self.0.max_duration = max_duration;
        self
    }

    pub fn min_duration(&mut self, min_duration: f64) -> &mut Self {
        self.0.min_duration = min_duration;
        self
    }

    pub fn validate(&self) -> Result<()> {
        let v = &self.0;
        if !v.margin.0.is_finite() || !v.margin.1.is_finite() {
            bail!("Invalid margin {:?}", v.margin);
        }
        if let Some(v) = v.duration {
            if !v.is_finite() || v < 0.0 {
                bail!("Invalid duration {}", v);
            }
        }
        if !v.min_duration.is_finite() || v.min_duration < 0.0 {
            bail!("Invalid min_duration {}", v.min_duration);
        }
        if v.max_duration.is_nan() || v.max_duration < 0.0 {
            bail!("Invalid max_duration {}", v.max_duration);
        }
        Ok(())
    }

    pub fn build(&self) -> Result<ElementCommon> {
        self.validate()?;
        Ok(self.0.clone())
    }
}

#[enum_dispatch(Schedule)]
#[derive(Debug, Clone)]
pub enum ElementVariant {
    Play,
    ShiftPhase,
    SetPhase,
    ShiftFreq,
    SetFreq,
    SwapPhase,
    Barrier,
    Repeat,
    Stack,
    Absolute,
    Grid,
}
