use std::{
    f64::consts::TAU,
    ops::{Add, Mul},
    sync::Arc,
};

use cached::proc_macro::cached;
use float_cmp::approx_eq;
use hashbrown::HashMap;
use itertools::{izip, Itertools};
use ndarray::{s, ArrayView2, ArrayViewMut2};
use numpy::Complex64;
use rayon::prelude::*;

use crate::{
    quant::{AlignedIndex, Frequency, Time},
    shape::Shape,
};

/// A pulse envelope
///
/// If `shape` is `None`, constructor will set `plateau` to `width + plateau`
/// and `width` to `0`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Envelope {
    shape: Option<Shape>,
    width: Time,
    plateau: Time,
}

impl Envelope {
    pub fn new(mut shape: Option<Shape>, mut width: Time, mut plateau: Time) -> Self {
        if shape.is_none() {
            plateau += width;
            width = Time::new(0.0).unwrap();
        }
        if width.value() == 0.0 {
            shape = None
        }
        Self {
            shape,
            width,
            plateau,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ListBin {
    envelope: Envelope,
    global_freq: Frequency,
    local_freq: Frequency,
}

#[derive(Debug, Clone, Copy)]
struct PulseAmplitude {
    // Amplitude of the pulse
    amp: Complex64,
    // Drag amplitude of the pulse (but not multiplied by sample rate yet)
    drag: Complex64,
}

impl Add for PulseAmplitude {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            amp: self.amp + other.amp,
            drag: self.drag + other.drag,
        }
    }
}

impl Mul<f64> for PulseAmplitude {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self {
        Self {
            amp: self.amp * rhs,
            drag: self.drag * rhs,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PulseList {
    items: HashMap<ListBin, Vec<(Time, PulseAmplitude)>>,
}

#[derive(Debug, Clone)]
pub struct Crosstalk<'a> {
    matrix: ArrayView2<'a, f64>,
    names: Vec<String>,
}

impl<'a> Crosstalk<'a> {
    pub fn new(matrix: ArrayView2<'a, f64>, names: Vec<String>) -> Self {
        Self { matrix, names }
    }
}

#[derive(Debug)]
pub struct Sampler<'a> {
    channels: HashMap<String, Channel<'a>>,
    pulse_lists: HashMap<String, PulseList>,
    crosstalk: Option<Crosstalk<'a>>,
}

impl<'a> Sampler<'a> {
    pub fn new(pulse_lists: HashMap<String, PulseList>) -> Self {
        Self {
            channels: HashMap::new(),
            pulse_lists,
            crosstalk: None,
        }
    }

    pub fn add_channel(
        &mut self,
        name: String,
        waveform: ArrayViewMut2<'a, f64>,
        sample_rate: Frequency,
        delay: Time,
        align_level: i32,
    ) {
        self.channels.insert(
            name.clone(),
            Channel {
                waveform,
                sample_rate,
                align_level,
                delay,
            },
        );
    }

    pub fn set_crosstalk(&mut self, crosstalk: ArrayView2<'a, f64>, names: Vec<String>) {
        self.crosstalk = Some(Crosstalk::new(crosstalk, names));
    }

    pub fn sample(self, time_tolerance: f64) {
        if let Some(crosstalk) = self.crosstalk {
            let ct_lookup = crosstalk
                .names
                .iter()
                .enumerate()
                .map(|(i, name)| (name.as_str(), i))
                .collect::<HashMap<_, _>>();
            self.channels.into_par_iter().for_each(|(n, c)| {
                let row_index = ct_lookup.get(n.as_str()).copied();
                if let Some(row_index) = row_index {
                    let row = crosstalk.matrix.slice(s![row_index, ..]);
                    let lists = row
                        .iter()
                        .copied()
                        .zip(&crosstalk.names)
                        .map(|(multiplier, in_name)| (multiplier, &self.pulse_lists[in_name]));
                    merge_and_sample(
                        lists,
                        c.waveform,
                        c.sample_rate,
                        c.delay,
                        c.align_level,
                        time_tolerance,
                    )
                } else {
                    let list = self.pulse_lists[&n]
                        .items
                        .iter()
                        .map(|(bin, items)| (bin.clone(), items.iter().copied()));
                    sample_pulse_list(list, c.waveform, c.sample_rate, c.delay, c.align_level)
                }
            });
        } else {
            self.channels.into_par_iter().for_each(|(n, c)| {
                let list = self.pulse_lists[&n]
                    .items
                    .iter()
                    .map(|(bin, items)| (bin.clone(), items.iter().copied()));
                sample_pulse_list(list, c.waveform, c.sample_rate, c.delay, c.align_level)
            })
        }
    }
}

#[derive(Debug)]
struct Channel<'a> {
    waveform: ArrayViewMut2<'a, f64>,
    sample_rate: Frequency,
    align_level: i32,
    delay: Time,
}

#[derive(Debug, Clone)]
pub struct PulseListBuilder {
    items: HashMap<ListBin, Vec<(Time, PulseAmplitude)>>,
    amp_tolerance: f64,
    time_tolerance: f64,
}

impl PulseListBuilder {
    pub fn new(amp_tolerance: f64, time_tolerance: f64) -> Self {
        Self {
            items: HashMap::new(),
            amp_tolerance,
            time_tolerance,
        }
    }

    pub fn push(
        &mut self,
        envelope: Envelope,
        global_freq: Frequency,
        local_freq: Frequency,
        time: Time,
        amplitude: f64,
        drag_coef: f64,
        phase: f64,
    ) {
        if approx_eq!(f64, amplitude, 0.0, epsilon = self.amp_tolerance) {
            return;
        }
        let bin = ListBin {
            envelope,
            global_freq,
            local_freq,
        };
        let amp = Complex64::from_polar(amplitude, TAU * phase);
        let drag = amp * Complex64::i() * drag_coef;
        let amplitude = PulseAmplitude { amp, drag };
        self.items.entry(bin).or_default().push((time, amplitude));
    }

    pub fn build(mut self) -> PulseList {
        for pulses in self.items.values_mut() {
            pulses.sort_unstable_by_key(|(time, _)| *time);
            let mut i = 0;
            for j in 1..pulses.len() {
                if approx_eq!(
                    f64,
                    pulses[i].0.value(),
                    pulses[j].0.value(),
                    epsilon = self.time_tolerance
                ) {
                    pulses[i].1 = pulses[i].1 + pulses[j].1;
                } else {
                    i += 1;
                    pulses[i] = pulses[j];
                }
            }
            pulses.truncate(i + 1);
        }
        PulseList { items: self.items }
    }
}

fn mix_add_envelope(
    mut waveform: ArrayViewMut2<f64>,
    envelope: &[f64],
    amplitude: Complex64,
    drag_amp: Complex64,
    phase0: f64,
    dphase: f64,
) {
    let mut carrier = Complex64::from_polar(1.0, phase0);
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
    for (mut y, env, slope) in izip!(waveform.columns_mut(), envelope.iter().copied(), slope_iter) {
        let w = carrier * (amplitude * env + drag_amp * slope);
        y[0] += w.re;
        y[1] += w.im;
        carrier *= dcarrier;
    }
}

fn mix_add_plateau(
    mut waveform: ArrayViewMut2<f64>,
    amplitude: Complex64,
    phase: f64,
    dphase: f64,
) {
    let mut carrier = Complex64::from_polar(1.0, phase) * amplitude;
    let dcarrier = Complex64::from_polar(1.0, dphase);
    for mut y in waveform.columns_mut() {
        y[0] += carrier.re;
        y[1] += carrier.im;
        carrier *= dcarrier;
    }
}

#[cached(size = 1024)]
fn get_envelope(
    shape: Shape,
    width: Time,
    plateau: Time,
    index_offset: AlignedIndex,
    sample_rate: Frequency,
) -> Arc<Vec<f64>> {
    let width = width.value();
    let plateau = plateau.value();
    let index_offset = index_offset.value();
    let sample_rate = sample_rate.value();
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

fn merge_and_sample<'a>(
    lists: impl IntoIterator<Item = (f64, &'a PulseList)>,
    waveform: ArrayViewMut2<f64>,
    sample_rate: Frequency,
    delay: Time,
    align_level: i32,
    time_tolerance: f64,
) {
    let mut merged: HashMap<ListBin, Vec<_>> = HashMap::new();
    for (multiplier, list) in lists {
        if multiplier == 0.0 {
            continue;
        }
        for (bin, items) in &list.items {
            merged.entry(bin.clone()).or_default().push(
                items
                    .iter()
                    .map(move |&(time, amp)| (time, amp * multiplier)),
            )
        }
    }
    let merged = merged.into_iter().map(|(bin, items)| {
        (
            bin,
            items
                .into_iter()
                .kmerge_by(|a, b| a.0 < b.0)
                .coalesce(|a, b| {
                    if approx_eq!(f64, a.0.value(), b.0.value(), epsilon = time_tolerance) {
                        Ok((a.0, a.1 + b.1))
                    } else {
                        Err((a, b))
                    }
                }),
        )
    });
    sample_pulse_list(merged, waveform, sample_rate, delay, align_level)
}

fn sample_pulse_list<PL, L>(
    list: PL,
    mut waveform: ArrayViewMut2<f64>,
    sample_rate: Frequency,
    delay: Time,
    align_level: i32,
) where
    PL: IntoIterator<Item = (ListBin, L)>,
    L: IntoIterator<Item = (Time, PulseAmplitude)>,
{
    for (bin, items) in list {
        let ListBin {
            envelope,
            global_freq,
            local_freq,
        } = bin;
        for (time, PulseAmplitude { amp, drag }) in items {
            let t_start = time + delay;
            let i_frac_start = AlignedIndex::new(t_start, sample_rate, align_level).unwrap();
            let i_start = i_frac_start.ceil();
            let index_offset = i_frac_start.index_offset();
            let global_freq = global_freq.value();
            let local_freq = local_freq.value();
            let total_freq = global_freq + local_freq;
            let dt = 1.0 / sample_rate.value();
            let phase0 = global_freq * (i_start.value() * dt - delay.value())
                + local_freq * index_offset.value() * dt;
            let dphase = total_freq * dt;
            let phase0 = phase0 * TAU;
            let dphase = dphase * TAU;
            let mut waveform = waveform.slice_mut(s![.., i_start.value() as usize..]);
            if let Some(shape) = &envelope.shape {
                let envelope = get_envelope(
                    shape.clone(),
                    envelope.width,
                    envelope.plateau,
                    index_offset,
                    sample_rate,
                );
                let drag = drag * sample_rate.value();
                mix_add_envelope(waveform, &envelope, amp, drag, phase0, dphase);
            } else {
                let plateau = envelope.plateau;
                let i_plateau = (plateau.value() * sample_rate.value()).ceil() as usize;
                let waveform = waveform.slice_mut(s![.., ..i_plateau]);
                mix_add_plateau(waveform, amp, phase0, dphase);
            }
        }
    }
}
