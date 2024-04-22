use std::{f64::consts::TAU, sync::Arc};

use cached::proc_macro::cached;
use itertools::izip;
use numpy::Complex64;
use ordered_float::NotNan;

use crate::{
    schedule::{self, ArrangedElement, ElementVariant},
    shape::Shape,
    time::{AlignedIndex, Time},
};

#[derive(Debug, Clone, Default)]
pub struct Sampler {
    channels: Vec<Channel>,
    shapes: Vec<Shape>,
}

impl Sampler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_channel(
        &mut self,
        base_freq: f64,
        sample_rate: f64,
        length: usize,
        delay: f64,
        align_level: i32,
    ) {
        self.channels.push(Channel::new(
            base_freq,
            sample_rate,
            length,
            delay,
            align_level,
        ));
    }

    pub fn add_shape(&mut self, shape: Shape) {
        self.shapes.push(shape);
    }

    pub fn execute(&mut self, element: &ArrangedElement) {
        self.execute_dispatch(element, 0.0);
    }

    pub fn into_result(self) -> Vec<Vec<Complex64>> {
        self.channels.into_iter().map(|c| c.waveform).collect()
    }

    fn execute_dispatch(&mut self, element: &ArrangedElement, time: f64) {
        if element.element().common().phantom() {
            return;
        }
        let time = time + element.inner_time();
        let duration = element.inner_duration();
        match element.element().variant() {
            ElementVariant::Play(e) => self.execute_play(e, time, duration),
            ElementVariant::ShiftPhase(e) => self.execute_shift_phase(e),
            ElementVariant::SetPhase(e) => self.execute_set_phase(e, time),
            ElementVariant::ShiftFreq(e) => self.execute_shift_freq(e, time),
            ElementVariant::SetFreq(e) => self.execute_set_freq(e, time),
            ElementVariant::SwapPhase(e) => self.execute_swap_phase(e, time),
            ElementVariant::Barrier(_) => (),
            ElementVariant::Repeat(e) => {
                let child = &element.try_get_children().expect("Invalid arrange data")[0];
                self.execute_repeat(e, child, time, duration);
            }
            ElementVariant::Stack(_) | ElementVariant::Absolute(_) | ElementVariant::Grid(_) => {
                let children = element.try_get_children().expect("Invalid arrange data");
                self.execute_container(children, time);
            }
        }
    }

    fn execute_play(&mut self, element: &schedule::Play, time: f64, duration: f64) {
        let shape = element.shape_id().map(|id| self.shapes[id].clone());
        let width = element.width();
        let plateau = if element.flexible() {
            duration - width
        } else {
            element.plateau()
        };
        let amplitude = element.amplitude();
        let drag_coef = element.drag_coef();
        let freq = element.frequency();
        let phase = element.phase();
        let channel = &mut self.channels[element.channel_id()];
        let time = Time::new(time).unwrap();
        let width = Time::new(width).unwrap();
        let plateau = Time::new(plateau).unwrap();
        channel.sample(
            shape, time, width, plateau, amplitude, drag_coef, freq, phase,
        );
    }

    fn execute_shift_phase(&mut self, element: &schedule::ShiftPhase) {
        let delta_phase = element.phase();
        let channel = &mut self.channels[element.channel_id()];
        channel.shift_phase(delta_phase);
    }

    fn execute_set_phase(&mut self, element: &schedule::SetPhase, time: f64) {
        let phase = element.phase();
        let channel = &mut self.channels[element.channel_id()];
        channel.set_phase(phase, time);
    }

    fn execute_shift_freq(&mut self, element: &schedule::ShiftFreq, time: f64) {
        let delta_freq = element.frequency();
        let channel = &mut self.channels[element.channel_id()];
        channel.shift_freq(delta_freq, time);
    }

    fn execute_set_freq(&mut self, element: &schedule::SetFreq, time: f64) {
        let freq = element.frequency();
        let channel = &mut self.channels[element.channel_id()];
        channel.set_freq(freq, time);
    }

    fn execute_swap_phase(&mut self, element: &schedule::SwapPhase, time: f64) {
        let ch1 = element.channel_id1();
        let ch2 = element.channel_id2();
        // Workaround for unstable get_many_mut
        if ch1 == ch2 {
            return;
        }
        let (ch1, ch2) = if ch1 < ch2 { (ch1, ch2) } else { (ch2, ch1) };
        let (a, b) = self.channels.split_at_mut(ch2);
        let channel = &mut a[ch1];
        let other = &mut b[0];
        channel.swap_phase(other, time);
    }

    fn execute_repeat(
        &mut self,
        element: &schedule::Repeat,
        child: &ArrangedElement,
        time: f64,
        duration: f64,
    ) {
        let count = element.count();
        if count == 0 {
            return;
        }
        let spacing = element.spacing();
        let time_step = (duration + spacing) / count as f64;
        for i in 0..count {
            let child_time = time + i as f64 * time_step;
            self.execute_dispatch(child, child_time);
        }
    }

    fn execute_container(&mut self, children: &[ArrangedElement], time: f64) {
        for child in children {
            self.execute_dispatch(child, time);
        }
    }
}

#[derive(Debug, Clone)]
struct Channel {
    base_freq: f64,
    delta_freq: f64,
    phase: f64,
    sample_rate: f64,
    waveform: Vec<Complex64>,
    delay: Time,
    align_level: i32,
}

impl Channel {
    fn new(base_freq: f64, sample_rate: f64, length: usize, delay: f64, align_level: i32) -> Self {
        Self {
            base_freq,
            delta_freq: 0.0,
            phase: 0.0,
            sample_rate,
            waveform: vec![Complex64::default(); length],
            delay: Time::new(delay).unwrap(),
            align_level,
        }
    }

    fn shift_freq(&mut self, delta_freq: f64, time: f64) {
        let delta_phase = -delta_freq * time;
        self.delta_freq += delta_freq;
        self.phase += delta_phase;
    }

    fn set_freq(&mut self, freq: f64, time: f64) {
        let delta_freq = freq - self.delta_freq;
        let delta_phase = -delta_freq * time;
        self.delta_freq = freq;
        self.phase += delta_phase;
    }

    fn shift_phase(&mut self, delta_phase: f64) {
        self.phase += delta_phase;
    }

    fn set_phase(&mut self, phase: f64, time: f64) {
        self.phase = phase - self.delta_freq * time;
    }

    fn total_freq(&self) -> f64 {
        self.base_freq + self.delta_freq
    }

    fn swap_phase(&mut self, other: &mut Self, time: f64) {
        let delta_freq = self.total_freq() - other.total_freq();
        let phase1 = self.phase;
        let phase2 = other.phase;
        self.phase = phase2 - delta_freq * time;
        other.phase = phase1 + delta_freq * time;
    }

    fn sample(
        &mut self,
        shape: Option<Shape>,
        time: Time,
        width: Time,
        plateau: Time,
        amplitude: f64,
        drag_coef: f64,
        freq: f64,
        phase: f64,
    ) {
        let t_start = time + self.delay;
        let i_frac_start = AlignedIndex::new(t_start, self.sample_rate, self.align_level).unwrap();
        let i_start = i_frac_start.ceil();
        let index_offset = i_frac_start.index_offset();
        let global_freq = self.total_freq();
        let local_freq = freq;
        let total_freq = global_freq + local_freq;
        let dt = 1.0 / self.sample_rate;
        let phase0 = phase
            + self.phase
            + global_freq * (i_start.value() * dt - self.delay.value())
            + local_freq * index_offset.value() * dt;
        let dphase = total_freq * dt;
        let phase0 = phase0 * TAU;
        let dphase = dphase * TAU;
        let waveform = &mut self.waveform[i_start.value() as usize..];
        if let Some(shape) = shape {
            let sample_rate = NotNan::new(self.sample_rate).unwrap();
            let envelope = get_envelope(shape, width, plateau, index_offset, sample_rate);
            let drag_coef = drag_coef * self.sample_rate;
            mix_add_envelope(waveform, &envelope, amplitude, drag_coef, phase0, dphase);
        } else {
            let i_plateau = ((width + plateau).value() * self.sample_rate).ceil() as usize;
            mix_add_plateau(&mut waveform[..i_plateau], amplitude, phase0, dphase);
        }
    }
}

#[cached(size = 1024)]
fn get_envelope(
    shape: Shape,
    width: Time,
    plateau: Time,
    index_offset: AlignedIndex,
    sample_rate: NotNan<f64>,
) -> Arc<Vec<f64>> {
    let width = width.value();
    let plateau = plateau.value();
    let index_offset = index_offset.value();
    let sample_rate = sample_rate.into_inner();
    let dt = 1.0 / sample_rate;
    let t_offset = index_offset * dt;
    let t1 = width / 2.0 - t_offset;
    let t2 = width / 2.0 + plateau - t_offset;
    let t3 = width + plateau - t_offset;
    let length = (t3 * sample_rate).ceil() as usize;
    let plateau_start_index = (t1 * sample_rate).ceil() as usize;
    let plateau_end_index = (t2 * sample_rate).ceil() as usize;
    let mut envelope = vec![0.0; length];
    let x0 = -t1 / width;
    let dx = dt / width;
    if plateau == 0.0 {
        shape.sample_array(x0, dx, &mut envelope);
    } else {
        shape.sample_array(x0, dx, &mut envelope[..plateau_start_index]);
        envelope[plateau_start_index..plateau_end_index].fill(1.0);
        let x2 = (plateau_end_index as f64 * dt - t2) / width;
        shape.sample_array(x2, dx, &mut envelope[plateau_end_index..]);
    }
    Arc::new(envelope)
}

fn mix_add_envelope(
    waveform: &mut [Complex64],
    envelope: &[f64],
    amplitude: f64,
    drag_coef: f64,
    phase: f64,
    dphase: f64,
) {
    let mut carrier = Complex64::from_polar(1.0, phase);
    let dcarrier = Complex64::from_polar(1.0, dphase);
    let slope_iter = (0..envelope.len()).map(|i| {
        let left = if i > 0 { envelope[i - 1] } else { 0.0 };
        let right = if i < envelope.len() - 1 {
            envelope[i + 1]
        } else {
            0.0
        };
        (right - left) / 2.0
    });
    for (y, env, slope) in izip!(waveform.iter_mut(), envelope.iter().copied(), slope_iter) {
        *y += carrier * (amplitude * env + Complex64::i() * drag_coef * slope);
        carrier *= dcarrier;
    }
}

pub fn mix_add_plateau(waveform: &mut [Complex64], amplitude: f64, phase: f64, dphase: f64) {
    let mut carrier = Complex64::from_polar(amplitude, phase);
    let dcarrier = Complex64::from_polar(1.0, dphase);
    for y in waveform.iter_mut() {
        *y += carrier;
        carrier *= dcarrier;
    }
}
