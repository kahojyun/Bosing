use hashbrown::HashMap;
use thiserror::Error;

use crate::{
    pulse::{Envelope, PulseList, PulseListBuilder, PushArgs},
    quant::{Amplitude, ChannelId, Frequency, Phase, ShapeId, Time},
    schedule::{
        walk_element, Element, Play, SetFreq, SetPhase, ShiftFreq, ShiftPhase, SwapPhase, Visitor,
    },
    shape::Shape,
};

#[derive(Debug, Clone)]
pub(crate) struct Executor {
    channels: HashMap<ChannelId, Channel>,
    shapes: HashMap<ShapeId, Shape>,
    amp_tolerance: Amplitude,
    time_tolerance: Time,
}

#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("Channel not found: {0:?}")]
    ChannelNotFound(Vec<ChannelId>),
    #[error("Shape not found: {0:?}")]
    ShapeNotFound(ShapeId),
    #[error("Invalid plateau: {0:?}")]
    NegativePlateau(Time),
}

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Clone)]
struct Channel {
    base_freq: Frequency,
    delta_freq: Frequency,
    phase: Phase,
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

impl Executor {
    pub(crate) fn new(amp_tolerance: Amplitude, time_tolerance: Time) -> Self {
        Self {
            channels: HashMap::new(),
            shapes: HashMap::new(),
            amp_tolerance,
            time_tolerance,
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

    fn get_mut_channel(&mut self, id: &ChannelId) -> Result<&mut Channel> {
        self.channels
            .get_mut(id)
            .ok_or(Error::ChannelNotFound(vec![id.clone()]))
    }
}

impl Channel {
    fn new(base_freq: Frequency, amp_tolerance: Amplitude, time_tolerance: Time) -> Self {
        Self {
            base_freq,
            delta_freq: Frequency::ZERO,
            phase: Phase::ZERO,
            pulses: PulseListBuilder::new(amp_tolerance, time_tolerance),
        }
    }

    fn shift_freq(&mut self, delta_freq: Frequency, time: Time) {
        let delta_phase = -delta_freq * time;
        self.delta_freq += delta_freq;
        self.phase += delta_phase;
    }

    fn set_freq(&mut self, freq: Frequency, time: Time) {
        let delta_freq = freq - self.delta_freq;
        let delta_phase = -delta_freq * time;
        self.delta_freq = freq;
        self.phase += delta_phase;
    }

    fn shift_phase(&mut self, delta_phase: Phase) {
        self.phase += delta_phase;
    }

    fn set_phase(&mut self, phase: Phase, time: Time) {
        self.phase = phase - self.delta_freq * time;
    }

    fn total_freq(&self) -> Frequency {
        self.base_freq + self.delta_freq
    }

    fn swap_phase(&mut self, other: &mut Self, time: Time) {
        let delta_freq = self.total_freq() - other.total_freq();
        let phase1 = self.phase;
        let phase2 = other.phase;
        self.phase = phase2 - delta_freq * time;
        other.phase = phase1 + delta_freq * time;
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

impl Visitor for Executor {
    type Error = Error;
    fn visit_play(&mut self, variant: &Play, time: Time, duration: Time) -> Result<()> {
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
            duration - width
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
            time,
            width,
            plateau,
            amplitude,
            drag_coef,
            freq,
            phase,
        });
        Ok(())
    }

    fn visit_shift_phase(
        &mut self,
        variant: &ShiftPhase,
        _time: Time,
        _durationn: Time,
    ) -> Result<()> {
        let delta_phase = variant.phase();
        let channel = self.get_mut_channel(variant.channel_id())?;
        channel.shift_phase(delta_phase);
        Ok(())
    }

    fn visit_set_phase(&mut self, variant: &SetPhase, time: Time, _duration: Time) -> Result<()> {
        let phase = variant.phase();
        let channel = self.get_mut_channel(variant.channel_id())?;
        channel.set_phase(phase, time);
        Ok(())
    }

    fn visit_shift_freq(&mut self, variant: &ShiftFreq, time: Time, _duration: Time) -> Result<()> {
        let delta_freq = variant.frequency();
        let channel = self.get_mut_channel(variant.channel_id())?;
        channel.shift_freq(delta_freq, time);
        Ok(())
    }

    fn visit_set_freq(&mut self, variant: &SetFreq, time: Time, _duration: Time) -> Result<()> {
        let freq = variant.frequency();
        let channel = self.get_mut_channel(variant.channel_id())?;
        channel.set_freq(freq, time);
        Ok(())
    }

    fn visit_swap_phase(&mut self, variant: &SwapPhase, time: Time, _duration: Time) -> Result<()> {
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

    fn visit_element(&mut self, element: &Element, time: Time, duration: Time) -> Result<()> {
        if element.common.phantom() {
            return Ok(());
        }
        walk_element(element, self, time, duration)
    }
}
