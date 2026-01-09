mod fir;
mod iir;

use std::{
    ops::{Add, Mul},
    sync::Arc,
};

use anyhow::{Context, Result, bail};
use cached::proc_macro::cached;
use float_cmp::approx_eq;
use hashbrown::HashMap;
use itertools::{Itertools, izip};
use ndarray::{ArrayView1, ArrayView2, ArrayViewMut2, Axis, azip, s};
use num::Complex;
use rayon::prelude::*;

type Complex64 = Complex<f64>;

use crate::{
    quant::{AlignedIndex, Amplitude, ChannelId, Frequency, Phase, Time},
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
    #[must_use]
    pub fn new(mut shape: Option<Shape>, mut width: Time, mut plateau: Time) -> Self {
        if shape.is_none() {
            plateau += width;
            width = Time::ZERO;
        }
        if width == Time::ZERO {
            shape = None;
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
pub struct List {
    items: HashMap<ListBin, Vec<(Time, PulseAmplitude)>>,
}

#[derive(Debug, Clone)]
pub struct Crosstalk<'a> {
    matrix: ArrayView2<'a, f64>,
    names: Vec<ChannelId>,
}

impl<'a> Crosstalk<'a> {
    #[must_use]
    pub const fn new(matrix: ArrayView2<'a, f64>, names: Vec<ChannelId>) -> Self {
        Self { matrix, names }
    }
}

#[derive(Debug)]
pub struct Sampler<'a> {
    channels: HashMap<ChannelId, Channel<'a>>,
    pulse_lists: HashMap<ChannelId, List>,
    crosstalk: Option<Crosstalk<'a>>,
}

impl<'a> Sampler<'a> {
    #[must_use]
    pub fn new(pulse_lists: HashMap<ChannelId, List>) -> Self {
        Self {
            channels: HashMap::new(),
            pulse_lists,
            crosstalk: None,
        }
    }

    pub fn add_channel(
        &mut self,
        name: ChannelId,
        waveform: ArrayViewMut2<'a, f64>,
        sample_rate: Frequency,
        delay: Time,
        align_level: i32,
    ) {
        self.channels.insert(
            name,
            Channel {
                waveform,
                sample_rate,
                align_level,
                delay,
            },
        );
    }

    pub fn set_crosstalk(&mut self, crosstalk: ArrayView2<'a, f64>, names: Vec<ChannelId>) {
        self.crosstalk = Some(Crosstalk::new(crosstalk, names));
    }

    pub fn sample(self, time_tolerance: Time) -> Result<()> {
        if let Some(crosstalk) = self.crosstalk {
            let ct_lookup = crosstalk
                .names
                .iter()
                .enumerate()
                .map(|(i, name)| (name, i))
                .collect::<HashMap<_, _>>();
            self.channels.into_par_iter().try_for_each(|(n, c)| {
                let row_index = ct_lookup.get(&n).copied();
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
                    .with_context(|| format!("Failed to sample channel '{n}'"))
                } else {
                    let list = self.pulse_lists[&n]
                        .items
                        .iter()
                        .map(|(bin, items)| (bin.clone(), items.iter().copied()));
                    sample_pulse_list(list, c.waveform, c.sample_rate, c.delay, c.align_level)
                        .with_context(|| format!("Failed to sample channel '{n}'"))
                }
            })
        } else {
            self.channels.into_par_iter().try_for_each(|(n, c)| {
                let list = self.pulse_lists[&n]
                    .items
                    .iter()
                    .map(|(bin, items)| (bin.clone(), items.iter().copied()));
                sample_pulse_list(list, c.waveform, c.sample_rate, c.delay, c.align_level)
                    .with_context(|| format!("Failed to sample channel '{n}'"))
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
pub struct ListBuilder {
    items: HashMap<ListBin, Vec<(Time, PulseAmplitude)>>,
    amp_tolerance: Amplitude,
    time_tolerance: Time,
}

pub struct PushArgs {
    pub(crate) envelope: Envelope,
    pub(crate) global_freq: Frequency,
    pub(crate) local_freq: Frequency,
    pub(crate) time: Time,
    pub(crate) amplitude: Amplitude,
    pub(crate) drag_coef: f64,
    pub(crate) phase: Phase,
}

impl ListBuilder {
    #[must_use]
    pub fn new(amp_tolerance: Amplitude, time_tolerance: Time) -> Self {
        Self {
            items: HashMap::new(),
            amp_tolerance,
            time_tolerance,
        }
    }

    pub fn push(
        &mut self,
        PushArgs {
            envelope,
            global_freq,
            local_freq,
            time,
            amplitude,
            drag_coef,
            phase,
        }: PushArgs,
    ) {
        if approx_eq!(
            f64,
            amplitude.value(),
            0.0,
            epsilon = self.amp_tolerance.value()
        ) {
            return;
        }
        let bin = ListBin {
            envelope,
            global_freq,
            local_freq,
        };
        let amp = amplitude.value() * phase.phaser();
        let drag = amp * Complex64::i() * drag_coef;
        let amplitude = PulseAmplitude { amp, drag };
        self.items.entry(bin).or_default().push((time, amplitude));
    }

    #[must_use]
    pub fn build(mut self) -> List {
        for pulses in self.items.values_mut() {
            pulses.sort_unstable_by_key(|(time, _)| *time);
            let mut i = 0;
            for j in 1..pulses.len() {
                if approx_eq!(
                    f64,
                    pulses[i].0.value(),
                    pulses[j].0.value(),
                    epsilon = self.time_tolerance.value()
                ) {
                    pulses[i].1 = pulses[i].1 + pulses[j].1;
                } else {
                    i += 1;
                    pulses[i] = pulses[j];
                }
            }
            pulses.truncate(i + 1);
        }
        List { items: self.items }
    }
}

fn mix_add_envelope(
    mut waveform: ArrayViewMut2<'_, f64>,
    envelope: &[f64],
    amplitude: Complex64,
    drag_amp: Complex64,
    phase0: Phase,
    dphase: Phase,
) {
    let mut carrier = phase0.phaser();
    let dcarrier = dphase.phaser();
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
        if let Some(y1) = y.get_mut(1) {
            *y1 += w.im;
        }
        carrier *= dcarrier;
    }
}

fn mix_add_plateau(
    mut waveform: ArrayViewMut2<'_, f64>,
    amplitude: Complex64,
    phase: Phase,
    dphase: Phase,
) {
    let mut carrier = phase.phaser() * amplitude;
    let dcarrier = dphase.phaser();
    for mut y in waveform.columns_mut() {
        y[0] += carrier.re;
        if let Some(y1) = y.get_mut(1) {
            *y1 += carrier.im;
        }
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
    fn time_to_index(t: f64, sr: f64) -> usize {
        assert!(t >= 0.0, "Time must be non-negative.");
        assert!(sr > 0.0, "Sample rate should be positive.");
        #[expect(clippy::cast_sign_loss, reason = "Asserted non-negative.")]
        #[expect(clippy::cast_possible_truncation, reason = "Index is small.")]
        let res = (t * sr).ceil() as usize;
        res
    }

    let width = width.value();
    let plateau = plateau.value();
    let index_offset = index_offset.value();
    let sample_rate = sample_rate.value();
    let dt = 1.0 / sample_rate;
    let t_offset = index_offset * dt;
    let t1 = width / 2.0 - t_offset;
    let t2 = width / 2.0 + plateau - t_offset;
    let t3 = width + plateau - t_offset;
    let length = time_to_index(t3, sample_rate);
    let plateau_start_index = time_to_index(t1, sample_rate);
    let plateau_end_index = time_to_index(t2, sample_rate);
    let mut envelope = vec![0.0; length];
    let x0 = -t1 / width;
    let dx = dt / width;
    if plateau == 0.0 {
        shape.sample_array(x0, dx, &mut envelope);
    } else {
        shape.sample_array(x0, dx, &mut envelope[..plateau_start_index]);
        envelope[plateau_start_index..plateau_end_index].fill(1.0);
        #[expect(clippy::cast_precision_loss, reason = "Index is small.")]
        let x2 = (plateau_end_index as f64).mul_add(dt, -t2) / width;
        shape.sample_array(x2, dx, &mut envelope[plateau_end_index..]);
    }
    Arc::new(envelope)
}

fn merge_and_sample<'a>(
    lists: impl IntoIterator<Item = (f64, &'a List)>,
    waveform: ArrayViewMut2<'_, f64>,
    sample_rate: Frequency,
    delay: Time,
    align_level: i32,
    time_tolerance: Time,
) -> Result<()> {
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
            );
        }
    }
    let merged = merged.into_iter().map(|(bin, items)| {
        (
            bin,
            items
                .into_iter()
                .kmerge_by(|a, b| a.0 < b.0)
                .coalesce(|a, b| {
                    if approx_eq!(
                        f64,
                        a.0.value(),
                        b.0.value(),
                        epsilon = time_tolerance.value()
                    ) {
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
    mut waveform: ArrayViewMut2<'_, f64>,
    sample_rate: Frequency,
    delay: Time,
    align_level: i32,
) -> Result<()>
where
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
            let i_frac_start = AlignedIndex::new(t_start, sample_rate, align_level)
                .expect("Reasonable align_level should not fail.");
            if i_frac_start.value() < 0.0 {
                bail!(
                    "The start time of a pulse is negative, try adjusting channel delay or schedule. start time: {time}",
                    time = t_start.value()
                );
            }
            let i_start = i_frac_start
                .ceil_to_usize()
                .expect("Reasonable align_level should not fail.");
            let index_offset = i_frac_start.index_offset();
            let total_freq = global_freq + local_freq;
            let dt = sample_rate.dt();
            #[expect(clippy::cast_precision_loss, reason = "Index is small.")]
            let phase0 = global_freq * (i_start as f64 * dt - delay)
                + local_freq * index_offset.value() * dt;
            let dphase = total_freq * dt;
            if i_start >= waveform.shape()[1] {
                bail!(
                    "The start index of a pulse is out of bounds, try adjusting channel delay, length or schedule. start index: {i_start}, start time: {time}",
                    time = t_start.value()
                );
            }
            let mut waveform = waveform.slice_mut(s![.., i_start..]);
            if let Some(shape) = &envelope.shape {
                let envelope = get_envelope(
                    shape.clone(),
                    envelope.width,
                    envelope.plateau,
                    index_offset,
                    sample_rate,
                );
                let drag = drag * sample_rate.value();
                if waveform.shape()[1] < envelope.len() {
                    #[expect(clippy::cast_precision_loss, reason = "Index is small.")]
                    let end_time = (envelope.len() as f64).mul_add(dt.value(), t_start.value());
                    bail!(
                        "The pulse end time is out of bounds, try adjusting channel delay, length or schedule. end time: {end_time}"
                    );
                }
                mix_add_envelope(waveform, &envelope, amp, drag, phase0, dphase);
            } else {
                let plateau = envelope.plateau;
                #[expect(clippy::cast_sign_loss, reason = "Plateau is positive.")]
                #[expect(clippy::cast_possible_truncation, reason = "Index is small.")]
                let i_plateau = (plateau.value() * sample_rate.value()).ceil() as usize;
                if waveform.shape()[1] < i_plateau {
                    bail!(
                        "The pulse end time is out of bounds, try adjusting channel delay, length or schedule. end time: {end_time}",
                        end_time = t_start.value() + plateau.value()
                    );
                }
                let waveform = waveform.slice_mut(s![.., ..i_plateau]);
                mix_add_plateau(waveform, amp, phase0, dphase);
            }
        }
    }
    Ok(())
}

pub fn apply_iq_inplace(waveform: &mut ArrayViewMut2<'_, f64>, iq_matrix: ArrayView2<'_, f64>) {
    assert!(matches!(waveform.shape(), [2, _]));
    assert!(matches!(iq_matrix.shape(), [2, 2]));
    for mut col in waveform.columns_mut() {
        let y = [
            iq_matrix[(0, 0)].mul_add(col[0], iq_matrix[(0, 1)] * col[1]),
            iq_matrix[(1, 0)].mul_add(col[0], iq_matrix[(1, 1)] * col[1]),
        ];
        col[0] = y[0];
        col[1] = y[1];
    }
}

pub fn apply_offset_inplace(waveform: &mut ArrayViewMut2<'_, f64>, offset: ArrayView1<'_, f64>) {
    assert!(waveform.shape()[0] == offset.len());
    azip!((mut row in waveform.axis_iter_mut(Axis(0)), &offset in &offset) row += offset);
}

pub fn apply_iir_inplace(waveform: &mut ArrayViewMut2<'_, f64>, sos: ArrayView2<'_, f64>) {
    self::iir::filter_inplace(waveform.view_mut(), sos)
        .expect("`sos` should be checked in IirArray.");
}

pub fn apply_fir_inplace(waveform: &mut ArrayViewMut2<'_, f64>, taps: ArrayView1<'_, f64>) {
    self::fir::filter_inplace(waveform.view_mut(), taps);
}
