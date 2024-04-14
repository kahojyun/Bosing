use anyhow::Result;
use stack::Stack;
use std::rc::Rc;

use enum_dispatch::enum_dispatch;

mod stack;

#[derive(Debug, Clone)]
struct Element {
    common: ElementCommon,
    variant: ElementVariant,
}

#[derive(Debug, Clone)]
struct MeasuredElement {
    element: Rc<Element>,
    /// Desired duration without clipping. Doesn't include margin.
    unclipped_duration: f64,
    /// Clipped desired duration. Used by scheduling system.
    duration: f64,
    children: Vec<MeasuredElement>,
}

#[derive(Debug, Clone)]
struct ArrangedElement {
    element: Rc<Element>,
    /// Start time of the inner block without margin relative to its parent.
    inner_time: f64,
    /// Duration of the inner block without margin.
    inner_duration: f64,
    children: Vec<ArrangedElement>,
}

#[derive(Debug, Clone)]
struct ScheduleOptions {
    time_tolerance: f64,
    allow_oversize: bool,
}

type MeasureResult = (f64, Vec<MeasuredElement>);

type ArrangeResult = (f64, Vec<ArrangedElement>);

#[enum_dispatch]
trait Schedule {
    /// Measure the element and return desired inner size and measured children.
    fn measure(&self, common: &ElementCommon, max_duration: f64) -> MeasureResult;
    /// Arrange the element and return final inner size and arranged children.
    fn arrange(
        &self,
        common: &ElementCommon,
        measured_children: &[MeasuredElement],
        time: f64,
        final_duration: f64,
        options: &ScheduleOptions,
    ) -> Result<ArrangeResult>;
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
    fn measure(&self, common: &ElementCommon, max_duration: f64) -> MeasureResult {
        (0.0, vec![])
    }

    fn arrange(
        &self,
        common: &ElementCommon,
        measured_children: &[MeasuredElement],
        time: f64,
        final_duration: f64,
        options: &ScheduleOptions,
    ) -> Result<ArrangeResult> {
        Ok((0.0, vec![]))
    }

    fn channels(&self) -> &[usize] {
        self.channels()
    }
}

fn clamp_duration(duration: f64, min_duration: f64, max_duration: f64) -> f64 {
    duration.min(max_duration).max(min_duration)
}

fn measure(element: Rc<Element>, available_duration: f64) -> MeasuredElement {
    debug_assert!(available_duration >= 0.0 || available_duration.is_infinite());
    let common = &element.common;
    let total_margin = common.margin.0 + common.margin.1;
    debug_assert!(total_margin.is_finite());
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
    let (measured, children) = element.variant.measure(common, inner_duration);
    let unclipped_duration = (measured + total_margin).max(0.0);
    let duration = clamp_duration(unclipped_duration, min_duration, max_duration) + total_margin;
    let duration = clamp_duration(duration, 0.0, available_duration);
    MeasuredElement {
        element,
        unclipped_duration,
        duration,
        children,
    }
}

fn arrange(
    element: &MeasuredElement,
    time: f64,
    duration: f64,
    options: &ScheduleOptions,
) -> Result<ArrangedElement> {
    let MeasuredElement {
        element,
        unclipped_duration,
        children: measured_children,
        ..
    } = element;
    let common = &element.common;
    if duration < unclipped_duration - options.time_tolerance && !options.allow_oversize {
        anyhow::bail!(
            "Oversizing is configured to be disallowed: available duration {} < measured duration {}",
            duration,
            unclipped_duration
        );
    }
    let inner_time = time + common.margin.0;
    debug_assert!(inner_time.is_finite());
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
        anyhow::bail!(
            "Oversizing is configured to be disallowed: user requested duration {} < measured duration {}",
            inner_duration + total_margin,
            unclipped_duration
        );
    }
    let (arranged, children) = element.variant.arrange(
        common,
        measured_children,
        inner_time,
        inner_duration,
        options,
    )?;
    debug_assert!(arranged.is_finite());
    Ok(ArrangedElement {
        element: element.clone(),
        inner_time,
        inner_duration: arranged,
        children,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Alignment {
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
    amplitude: f64,
    shape_id: usize,
    width: f64,
    plateau: f64,
    drag_coef: f64,
    frequency: f64,
    phase: f64,
    flexible: bool,
}

impl Schedule for Play {
    fn measure(&self, common: &ElementCommon, max_duration: f64) -> MeasureResult {
        let wanted_duration = if self.flexible {
            self.width
        } else {
            self.width + self.plateau
        };
        (wanted_duration, vec![])
    }

    fn arrange(
        &self,
        common: &ElementCommon,
        measured_children: &[MeasuredElement],
        time: f64,
        final_duration: f64,
        options: &ScheduleOptions,
    ) -> Result<ArrangeResult> {
        let arranged = if self.flexible {
            final_duration
        } else {
            self.width + self.plateau
        };
        Ok((arranged, vec![]))
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
    fn measure(&self, common: &ElementCommon, max_duration: f64) -> MeasureResult {
        if self.count == 0 {
            return (0.0, vec![]);
        }
        let n = self.count as f64;
        let duration_per_repeat = (max_duration - self.spacing * (n - 1.0)) / n;
        let measured_child = measure(self.child.clone(), duration_per_repeat);
        let wanted_duration = measured_child.duration * n + self.spacing * (n - 1.0);
        (wanted_duration, vec![measured_child])
    }

    fn arrange(
        &self,
        common: &ElementCommon,
        measured_children: &[MeasuredElement],
        time: f64,
        final_duration: f64,
        options: &ScheduleOptions,
    ) -> Result<ArrangeResult> {
        if self.count == 0 {
            return Ok((0.0, vec![]));
        }
        let n = self.count as f64;
        let duration_per_repeat = (final_duration - self.spacing * (n - 1.0)) / n;
        let arranged_child = arrange(&measured_children[0], 0.0, duration_per_repeat, options)?;
        let arranged = arranged_child.inner_duration * n + self.spacing * (n - 1.0);
        Ok((arranged, vec![arranged_child]))
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
    fn measure(&self, common: &ElementCommon, max_duration: f64) -> MeasureResult {
        todo!()
    }

    fn arrange(
        &self,
        common: &ElementCommon,
        measured_children: &[MeasuredElement],
        time: f64,
        final_duration: f64,
        options: &ScheduleOptions,
    ) -> Result<ArrangeResult> {
        todo!()
    }

    fn channels(&self) -> &[usize] {
        &self.channel_ids
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GridLengthUnit {
    Seconds,
    Auto,
    Star,
}

#[derive(Debug, Clone)]
struct GridLength {
    value: f64,
    unit: GridLengthUnit,
}

#[derive(Debug, Clone)]
struct GridEntry {
    element: Rc<Element>,
    column: usize,
    span: usize,
}

#[derive(Debug, Clone)]
struct Grid {
    children: Vec<GridEntry>,
    columns: Vec<GridLength>,
    channel_ids: Vec<usize>,
}

impl Schedule for Grid {
    fn measure(&self, common: &ElementCommon, max_duration: f64) -> MeasureResult {
        todo!()
    }

    fn arrange(
        &self,
        common: &ElementCommon,
        measured_children: &[MeasuredElement],
        time: f64,
        final_duration: f64,
        options: &ScheduleOptions,
    ) -> Result<ArrangeResult> {
        todo!()
    }

    fn channels(&self) -> &[usize] {
        &self.channel_ids
    }
}
