use anyhow::{bail, Result};
use enum_dispatch::enum_dispatch;
use std::rc::Rc;

mod builder;
mod grid;
mod stack;

use grid::Grid;
use stack::Stack;

#[derive(Debug, Clone)]
pub struct Element {
    common: ElementCommon,
    variant: ElementVariant,
}

#[derive(Debug, Clone)]
pub struct MeasuredElement {
    element: Rc<Element>,
    /// Desired duration without clipping. Doesn't include margin.
    unclipped_duration: f64,
    /// Clipped desired duration. Used by scheduling system.
    duration: f64,
    data: MeasureResultVariant,
}

#[derive(Debug, Clone)]
pub struct ArrangedElement {
    element: Rc<Element>,
    /// Start time of the inner block without margin relative to its parent.
    inner_time: f64,
    /// Duration of the inner block without margin.
    inner_duration: f64,
    data: ArrangeResultVariant,
}

#[derive(Debug, Clone)]
pub struct ScheduleOptions {
    time_tolerance: f64,
    allow_oversize: bool,
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
enum ArrangeResultVariant {
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
    fn channels(&self) -> &[usize];
}

trait SimpleElement {
    fn channels(&self) -> &[usize];
}

impl<T> Schedule for T
where
    T: SimpleElement,
{
    fn measure(&self, _context: &MeasureContext) -> MeasureResult {
        MeasureResult(0.0, MeasureResultVariant::Simple)
    }

    fn arrange(&self, _context: &ArrangeContext) -> Result<ArrangeResult> {
        Ok(ArrangeResult(0.0, ArrangeResultVariant::Simple))
    }

    fn channels(&self) -> &[usize] {
        self.channels()
    }
}

fn clamp_duration(duration: f64, min_duration: f64, max_duration: f64) -> f64 {
    duration.min(max_duration).max(min_duration)
}

pub fn measure(element: Rc<Element>, available_duration: f64) -> MeasuredElement {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    End,
    Start,
    Center,
    Stretch,
}

#[derive(Debug, Clone)]
struct ElementCommon {
    margin: (f64, f64),
    alignment: Alignment,
    phantom: bool,
    duration: Option<f64>,
    max_duration: f64,
    min_duration: f64,
}

#[enum_dispatch(Schedule)]
#[derive(Debug, Clone)]
enum ElementVariant {
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

#[derive(Debug, Clone)]
struct Play {
    channel_id: [usize; 1],
    shape_id: Option<usize>,
    amplitude: f64,
    width: f64,
    plateau: f64,
    drag_coef: f64,
    frequency: f64,
    phase: f64,
    flexible: bool,
}

impl Schedule for Play {
    fn measure(&self, _context: &MeasureContext) -> MeasureResult {
        let wanted_duration = if self.flexible {
            self.width
        } else {
            self.width + self.plateau
        };
        MeasureResult(wanted_duration, MeasureResultVariant::Simple)
    }

    fn arrange(&self, context: &ArrangeContext) -> Result<ArrangeResult> {
        let arranged = if self.flexible {
            context.final_duration
        } else {
            self.width + self.plateau
        };
        Ok(ArrangeResult(arranged, ArrangeResultVariant::Simple))
    }

    fn channels(&self) -> &[usize] {
        &self.channel_id
    }
}

#[derive(Debug, Clone)]
struct ShiftPhase {
    channel_id: [usize; 1],
    phase: f64,
}

impl SimpleElement for ShiftPhase {
    fn channels(&self) -> &[usize] {
        &self.channel_id
    }
}

#[derive(Debug, Clone)]
struct SetPhase {
    channel_id: [usize; 1],
    phase: f64,
}

impl SimpleElement for SetPhase {
    fn channels(&self) -> &[usize] {
        &self.channel_id
    }
}

#[derive(Debug, Clone)]
struct ShiftFreq {
    channel_id: [usize; 1],
    frequency: f64,
}

impl SimpleElement for ShiftFreq {
    fn channels(&self) -> &[usize] {
        &self.channel_id
    }
}

#[derive(Debug, Clone)]
struct SetFreq {
    channel_id: [usize; 1],
    frequency: f64,
}

impl SimpleElement for SetFreq {
    fn channels(&self) -> &[usize] {
        &self.channel_id
    }
}

#[derive(Debug, Clone)]
struct SwapPhase {
    channel_ids: [usize; 2],
}

impl SimpleElement for SwapPhase {
    fn channels(&self) -> &[usize] {
        &self.channel_ids
    }
}

#[derive(Debug, Clone)]
struct Barrier {
    channel_ids: Vec<usize>,
}

impl SimpleElement for Barrier {
    fn channels(&self) -> &[usize] {
        &self.channel_ids
    }
}

#[derive(Debug, Clone)]
struct Repeat {
    child: Rc<Element>,
    count: usize,
    spacing: f64,
}

impl Schedule for Repeat {
    fn measure(&self, context: &MeasureContext) -> MeasureResult {
        if self.count == 0 {
            return MeasureResult(0.0, MeasureResultVariant::Simple);
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
            return Ok(ArrangeResult(0.0, ArrangeResultVariant::Simple));
        }
        let n = self.count as f64;
        let duration_per_repeat = (context.final_duration - self.spacing * (n - 1.0)) / n;
        let measured_child = match &context.measured_self.data {
            MeasureResultVariant::Multiple(c) if c.len() == 1 => &c[0],
            _ => bail!("Invalid measure data"),
        };
        let arranged_child = arrange(measured_child, 0.0, duration_per_repeat, context.options)?;
        let arranged = arranged_child.inner_duration * n + self.spacing * (n - 1.0);
        Ok(ArrangeResult(
            arranged,
            ArrangeResultVariant::Multiple(vec![arranged_child]),
        ))
    }

    fn channels(&self) -> &[usize] {
        self.child.variant.channels()
    }
}

#[derive(Debug, Clone)]
struct AbsoluteEntry {
    time: f64,
    element: Rc<Element>,
}

#[derive(Debug, Clone)]
struct Absolute {
    children: Vec<AbsoluteEntry>,
    channel_ids: Vec<usize>,
}

impl Schedule for Absolute {
    fn measure(&self, context: &MeasureContext) -> MeasureResult {
        let mut max_time: f64 = 0.0;
        let mut measured_children = vec![];
        for e in &self.children {
            let measured_child = measure(e.element.clone(), context.max_duration);
            max_time = max_time.max(e.time + measured_child.duration);
            measured_children.push(measured_child);
        }
        MeasureResult(max_time, MeasureResultVariant::Multiple(measured_children))
    }

    fn arrange(&self, context: &ArrangeContext) -> Result<ArrangeResult> {
        let measured_children = match &context.measured_self.data {
            MeasureResultVariant::Multiple(v) => v,
            _ => bail!("Invalid measure data"),
        };
        let arranged_children = self
            .children
            .iter()
            .map(|e| e.time)
            .zip(measured_children.iter())
            .map(|(t, mc)| arrange(mc, t, mc.duration, context.options))
            .collect::<Result<_>>()?;
        Ok(ArrangeResult(
            context.final_duration,
            ArrangeResultVariant::Multiple(arranged_children),
        ))
    }

    fn channels(&self) -> &[usize] {
        &self.channel_ids
    }
}
