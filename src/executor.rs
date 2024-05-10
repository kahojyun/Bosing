use hashbrown::HashMap;

use crate::{
    pulse::{Envelope, PulseList, PulseListBuilder},
    quant::{Amplitude, ChannelId, Frequency, Phase, ShapeId, Time},
    schedule::{self, Visitor},
    shape::Shape,
};

#[derive(Debug, Clone)]
pub(crate) struct Executor {
    channels: HashMap<ChannelId, Channel>,
    shapes: HashMap<ShapeId, Shape>,
    amp_tolerance: Amplitude,
    time_tolerance: Time,
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
}

impl Visitor for Executor {
    fn visit_play(&mut self, variant: &schedule::Play, time: Time, duration: Time) {
        let shape = variant.shape_id().map(|id| self.shapes[id].clone());
        let width = variant.width();
        let plateau = if variant.flexible() {
            duration - width
        } else {
            variant.plateau()
        };
        let amplitude = variant.amplitude();
        let drag_coef = variant.drag_coef();
        let freq = variant.frequency();
        let phase = variant.phase();
        let channel = self.channels.get_mut(variant.channel_id()).unwrap();
        channel.add_pulse(
            shape, time, width, plateau, amplitude, drag_coef, freq, phase,
        );
    }

    fn visit_shift_phase(&mut self, variant: &schedule::ShiftPhase, _time: Time, _durationn: Time) {
        let delta_phase = variant.phase();
        let channel = self.channels.get_mut(variant.channel_id()).unwrap();
        channel.shift_phase(delta_phase);
    }

    fn visit_set_phase(&mut self, variant: &schedule::SetPhase, time: Time, _duration: Time) {
        let phase = variant.phase();
        let channel = self.channels.get_mut(variant.channel_id()).unwrap();
        channel.set_phase(phase, time);
    }

    fn visit_shift_freq(&mut self, variant: &schedule::ShiftFreq, time: Time, _duration: Time) {
        let delta_freq = variant.frequency();
        let channel = self.channels.get_mut(variant.channel_id()).unwrap();
        channel.shift_freq(delta_freq, time);
    }

    fn visit_set_freq(&mut self, variant: &schedule::SetFreq, time: Time, _duration: Time) {
        let freq = variant.frequency();
        let channel = self.channels.get_mut(variant.channel_id()).unwrap();
        channel.set_freq(freq, time);
    }

    fn visit_swap_phase(&mut self, variant: &schedule::SwapPhase, time: Time, _duration: Time) {
        let ch1 = variant.channel_id1();
        let ch2 = variant.channel_id2();
        if ch1 == ch2 {
            return;
        }
        let [channel, other] = self.channels.get_many_mut([ch1, ch2]).unwrap();
        channel.swap_phase(other, time);
    }

    fn visit_barrier(&mut self, _variant: &schedule::Barrier, _time: Time, _duration: Time) {}

    fn visit_repeat(&mut self, _variant: &schedule::Repeat, _time: Time, _duration: Time) {}

    fn visit_stack(&mut self, _variant: &schedule::Stack, _time: Time, _duration: Time) {}

    fn visit_absolute(&mut self, _variant: &schedule::Absolute, _time: Time, _duration: Time) {}

    fn visit_grid(&mut self, _variant: &schedule::Grid, _time: Time, _duration: Time) {}

    fn visit_common(&mut self, _common: &schedule::ElementCommon, _time: Time, _duration: Time) {}
}

#[derive(Debug, Clone)]
struct Channel {
    base_freq: Frequency,
    delta_freq: Frequency,
    phase: Phase,
    pulses: PulseListBuilder,
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
        shape: Option<Shape>,
        time: Time,
        width: Time,
        plateau: Time,
        amplitude: Amplitude,
        drag_coef: f64,
        freq: Frequency,
        phase: Phase,
    ) {
        let envelope = Envelope::new(shape, width, plateau);
        let global_freq = self.total_freq();
        let local_freq = freq;
        self.pulses.push(
            envelope,
            global_freq,
            local_freq,
            time,
            amplitude,
            drag_coef,
            phase,
        )
    }
}
