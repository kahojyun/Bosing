use std::iter;

use hashbrown::HashMap;
use thiserror::Error;

use crate::{
    pulse::{Envelope, PulseList, PulseListBuilder, PushArgs},
    quant::{Amplitude, ChannelId, Frequency, Phase, ShapeId, Time},
    schedule::{
        Arrange as _, Arranged, ElementRef, ElementVariant, Measure, Play, SetFreq, SetPhase,
        ShiftFreq, ShiftPhase, SwapPhase, TimeRange,
    },
    shape::Shape,
};

#[derive(Debug, Clone)]
pub(crate) struct Executor {
    channels: HashMap<ChannelId, Channel>,
    shapes: HashMap<ShapeId, Shape>,
    amp_tolerance: Amplitude,
    time_tolerance: Time,
    allow_oversize: bool,
}

#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("Channel not found: {0:?}")]
    ChannelNotFound(Vec<ChannelId>),
    #[error("Shape not found: {0:?}")]
    ShapeNotFound(ShapeId),
    #[error("Invalid plateau: {0:?}")]
    NegativePlateau(Time),
    #[error("Not enough duration: required {required:?}, available {available:?}")]
    NotEnoughDuration { required: Time, available: Time },
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct OscState {
    freq: Frequency,
    phase: Phase,
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
struct Channel {
    base_freq: Frequency,
    osc: OscState,
    pulses: PulseListBuilder,
}

struct AddPulseArgs {
    shape: Option<Shape>,
    time: Time,
    width: Time,
    plateau: Time,
    amplitude: Amplitude,
    drag_coef: f64,
    freq: Frequency,
    phase: Phase,
}

#[derive(Debug)]
enum IterVariant<S, A, G, R> {
    Stack(S),
    Absolute(A),
    Grid(G),
    Repeat(R),
}

impl Executor {
    pub(crate) fn new(
        amp_tolerance: Amplitude,
        time_tolerance: Time,
        allow_oversize: bool,
    ) -> Self {
        Self {
            channels: HashMap::new(),
            shapes: HashMap::new(),
            amp_tolerance,
            time_tolerance,
            allow_oversize,
        }
    }

    pub(crate) fn add_channel(&mut self, name: ChannelId, base_freq: Frequency) {
        self.channels.insert(
            name,
            Channel::new(base_freq, self.amp_tolerance, self.time_tolerance),
        );
    }

    pub(crate) fn add_shape(&mut self, name: ShapeId, shape: Shape) {
        self.shapes.insert(name, shape);
    }

    pub(crate) fn into_result(self) -> HashMap<ChannelId, PulseList> {
        self.channels
            .into_iter()
            .map(|(n, b)| (n, b.pulses.build()))
            .collect()
    }

    pub(crate) fn execute(&mut self, root: &ElementRef) -> Result<()> {
        let time_range = TimeRange {
            start: Time::ZERO,
            span: root.measure(),
        };
        for Arranged { item, time_range } in arrange_tree(root, time_range) {
            let time_range = item.inner_time_range(time_range);
            if !self.allow_oversize {
                let required = item.variant.measure();
                check_duration(required, time_range.span, self.time_tolerance)?;
            }
            match &item.variant {
                ElementVariant::Play(variant) => self.execute_play(variant, time_range),
                ElementVariant::ShiftPhase(variant) => self.execute_shift_phase(variant),
                ElementVariant::SetPhase(variant) => {
                    self.execute_set_phase(variant, time_range.start)
                }
                ElementVariant::ShiftFreq(variant) => {
                    self.execute_shift_freq(variant, time_range.start)
                }
                ElementVariant::SetFreq(variant) => {
                    self.execute_set_freq(variant, time_range.start)
                }
                ElementVariant::SwapPhase(variant) => {
                    self.execute_swap_phase(variant, time_range.start)
                }
                _ => Ok(()),
            }?;
        }
        Ok(())
    }

    fn execute_play(&mut self, variant: &Play, time_range: TimeRange) -> Result<()> {
        let shape = match variant.shape_id() {
            Some(id) => Some(
                self.shapes
                    .get(id)
                    .ok_or(Error::ShapeNotFound(id.clone()))?
                    .clone(),
            ),
            None => None,
        };
        let width = variant.width();
        let plateau = if variant.flexible() {
            time_range.span - width
        } else {
            variant.plateau()
        };
        if plateau < Time::ZERO {
            return Err(Error::NegativePlateau(plateau));
        }
        let amplitude = variant.amplitude();
        let drag_coef = variant.drag_coef();
        let freq = variant.frequency();
        let phase = variant.phase();
        let channel = self.get_mut_channel(variant.channel_id())?;
        channel.add_pulse(AddPulseArgs {
            shape,
            time: time_range.start,
            width,
            plateau,
            amplitude,
            drag_coef,
            freq,
            phase,
        });
        Ok(())
    }

    fn execute_shift_phase(&mut self, variant: &ShiftPhase) -> Result<()> {
        let delta_phase = variant.phase();
        let channel = self.get_mut_channel(variant.channel_id())?;
        channel.shift_phase(delta_phase);
        Ok(())
    }

    fn execute_set_phase(&mut self, variant: &SetPhase, time: Time) -> Result<()> {
        let phase = variant.phase();
        let channel = self.get_mut_channel(variant.channel_id())?;
        channel.set_phase(phase, time);
        Ok(())
    }

    fn execute_shift_freq(&mut self, variant: &ShiftFreq, time: Time) -> Result<()> {
        let delta_freq = variant.frequency();
        let channel = self.get_mut_channel(variant.channel_id())?;
        channel.shift_freq(delta_freq, time);
        Ok(())
    }

    fn execute_set_freq(&mut self, variant: &SetFreq, time: Time) -> Result<()> {
        let freq = variant.frequency();
        let channel = self.get_mut_channel(variant.channel_id())?;
        channel.set_freq(freq, time);
        Ok(())
    }

    fn execute_swap_phase(&mut self, variant: &SwapPhase, time: Time) -> Result<()> {
        let ch1 = variant.channel_id1();
        let ch2 = variant.channel_id2();
        if ch1 == ch2 {
            return Ok(());
        }
        let [channel, other] = self
            .channels
            .get_many_mut([ch1, ch2])
            .ok_or(Error::ChannelNotFound(vec![ch1.clone(), ch2.clone()]))?;
        channel.swap_phase(other, time);
        Ok(())
    }

    fn get_mut_channel(&mut self, id: &ChannelId) -> Result<&mut Channel> {
        self.channels
            .get_mut(id)
            .ok_or(Error::ChannelNotFound(vec![id.clone()]))
    }
}

impl OscState {
    fn new() -> Self {
        Self::default()
    }

    fn phase_at(&self, time: Time) -> Phase {
        self.phase + self.freq * time
    }

    fn with_time_shift(&self, time: Time) -> Self {
        Self {
            freq: self.freq,
            phase: self.phase_at(time),
        }
    }
}

impl Channel {
    fn new(base_freq: Frequency, amp_tolerance: Amplitude, time_tolerance: Time) -> Self {
        Self {
            base_freq,
            osc: OscState::new(),
            pulses: PulseListBuilder::new(amp_tolerance, time_tolerance),
        }
    }

    fn shift_freq(&mut self, delta_freq: Frequency, time: Time) {
        let mut osc = self.osc.with_time_shift(time);
        osc.freq += delta_freq;
        self.osc = osc.with_time_shift(-time);
    }

    fn set_freq(&mut self, freq: Frequency, time: Time) {
        let mut osc = self.osc.with_time_shift(time);
        osc.freq = freq;
        self.osc = osc.with_time_shift(-time);
    }

    fn shift_phase(&mut self, delta_phase: Phase) {
        self.osc.phase += delta_phase;
    }

    fn set_phase(&mut self, phase: Phase, time: Time) {
        let mut osc = self.osc.with_time_shift(time);
        osc.phase = phase;
        self.osc = osc.with_time_shift(-time);
    }

    fn total_freq(&self) -> Frequency {
        self.base_freq + self.osc.freq
    }

    fn swap_phase(&mut self, other: &mut Self, time: Time) {
        let mut osc1 = self.osc;
        let mut osc2 = other.osc;
        osc1.freq += self.base_freq;
        osc2.freq += other.base_freq;
        let mut osc1 = osc1.with_time_shift(time);
        let mut osc2 = osc2.with_time_shift(time);
        std::mem::swap(&mut osc1.phase, &mut osc2.phase);
        let mut osc1 = osc1.with_time_shift(-time);
        let mut osc2 = osc2.with_time_shift(-time);
        osc1.freq -= self.base_freq;
        osc2.freq -= other.base_freq;
        self.osc = osc1;
        other.osc = osc2;
    }

    fn add_pulse(
        &mut self,
        AddPulseArgs {
            shape,
            time,
            width,
            plateau,
            amplitude,
            drag_coef,
            freq,
            phase,
        }: AddPulseArgs,
    ) {
        let envelope = Envelope::new(shape, width, plateau);
        let global_freq = self.total_freq();
        let local_freq = freq;
        self.pulses.push(PushArgs {
            envelope,
            global_freq,
            local_freq,
            time,
            amplitude,
            drag_coef,
            phase,
        })
    }
}

impl<S, A, G, R, T> Iterator for IterVariant<S, A, G, R>
where
    S: Iterator<Item = T>,
    A: Iterator<Item = T>,
    G: Iterator<Item = T>,
    R: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IterVariant::Stack(s) => s.next(),
            IterVariant::Absolute(a) => a.next(),
            IterVariant::Grid(g) => g.next(),
            IterVariant::Repeat(r) => r.next(),
        }
    }
}

fn check_duration(required: Time, available: Time, time_tolerance: Time) -> Result<()> {
    if required > available + time_tolerance {
        return Err(Error::NotEnoughDuration {
            required,
            available,
        });
    }
    Ok(())
}

fn arrange_tree(
    root: &ElementRef,
    time_range: TimeRange,
) -> impl Iterator<Item = Arranged<&ElementRef>> {
    pre_order_iter(
        Arranged {
            item: root,
            time_range,
        },
        arrange_children,
    )
    .filter(|Arranged { item, .. }| !item.common.phantom())
}

fn arrange_children(
    Arranged { item, time_range }: Arranged<&ElementRef>,
) -> Option<impl Iterator<Item = Arranged<&ElementRef>>> {
    if item.common.phantom() {
        return None;
    }
    let time_range = item.inner_time_range(time_range);
    match &item.variant {
        ElementVariant::Repeat(r) => Some(IterVariant::Repeat(r.arrange(time_range))),
        ElementVariant::Stack(s) => Some(IterVariant::Stack(s.arrange(time_range))),
        ElementVariant::Absolute(a) => Some(IterVariant::Absolute(a.arrange(time_range))),
        ElementVariant::Grid(g) => Some(IterVariant::Grid(g.arrange(time_range))),
        _ => None,
    }
}

fn pre_order_iter<T, F, I>(root: T, mut children: F) -> impl Iterator<Item = T>
where
    F: FnMut(T) -> Option<I>,
    I: Iterator<Item = T>,
    T: Clone + Copy,
{
    let mut stack = Vec::with_capacity(16);
    stack.extend(children(root));
    iter::once(root).chain(iter::from_fn(move || loop {
        let current_iter = stack.last_mut()?;
        match current_iter.next() {
            Some(i) => {
                stack.extend(children(i));
                return Some(i);
            }
            None => {
                stack.pop();
            }
        }
    }))
}

#[cfg(test)]
mod tests {
    #[test]
    fn pre_order() {
        let node_children = vec![
            vec![1, 2, 3],
            vec![4, 5],
            vec![6, 7],
            vec![],
            vec![8, 9],
            vec![],
            vec![10, 11],
            vec![],
            vec![],
            vec![],
            vec![],
        ];
        let expected = vec![0, 1, 4, 8, 9, 5, 2, 6, 10, 11, 7, 3];

        let result = super::pre_order_iter(0, |i| node_children.get(i).map(|c| c.iter().copied()))
            .collect::<Vec<_>>();

        assert_eq!(result, expected);
    }
}
