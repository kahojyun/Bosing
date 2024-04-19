use anyhow::{bail, Result};

use super::{
    ArrangeContext, ArrangeResult, ArrangeResultVariant, MeasureContext, MeasureResult,
    MeasureResultVariant, Schedule,
};

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

#[derive(Debug, Clone)]
pub struct ShiftPhase {
    channel_id: [usize; 1],
    phase: f64,
}

impl ShiftPhase {
    pub fn new(channel_id: usize, phase: f64) -> Result<Self> {
        if !phase.is_finite() {
            bail!("Invalid phase {}", phase);
        }
        Ok(Self {
            channel_id: [channel_id],
            phase,
        })
    }

    pub fn channel_id(&self) -> usize {
        self.channel_id[0]
    }

    pub fn phase(&self) -> f64 {
        self.phase
    }
}

impl SimpleElement for ShiftPhase {
    fn channels(&self) -> &[usize] {
        &self.channel_id
    }
}

#[derive(Debug, Clone)]
pub struct SetPhase {
    channel_id: [usize; 1],
    phase: f64,
}

impl SetPhase {
    pub fn new(channel_id: usize, phase: f64) -> Result<Self> {
        if !phase.is_finite() {
            bail!("Invalid phase {}", phase);
        }
        Ok(Self {
            channel_id: [channel_id],
            phase,
        })
    }

    pub fn channel_id(&self) -> usize {
        self.channel_id[0]
    }

    pub fn phase(&self) -> f64 {
        self.phase
    }
}

impl SimpleElement for SetPhase {
    fn channels(&self) -> &[usize] {
        &self.channel_id
    }
}

#[derive(Debug, Clone)]
pub struct ShiftFreq {
    channel_id: [usize; 1],
    frequency: f64,
}

impl ShiftFreq {
    pub fn new(channel_id: usize, frequency: f64) -> Result<Self> {
        if !frequency.is_finite() {
            bail!("Invalid frequency {}", frequency);
        }
        Ok(Self {
            channel_id: [channel_id],
            frequency,
        })
    }

    pub fn channel_id(&self) -> usize {
        self.channel_id[0]
    }

    pub fn frequency(&self) -> f64 {
        self.frequency
    }
}

impl SimpleElement for ShiftFreq {
    fn channels(&self) -> &[usize] {
        &self.channel_id
    }
}

#[derive(Debug, Clone)]
pub struct SetFreq {
    channel_id: [usize; 1],
    frequency: f64,
}

impl SetFreq {
    pub fn new(channel_id: usize, frequency: f64) -> Result<Self> {
        if !frequency.is_finite() {
            bail!("Invalid frequency {}", frequency);
        }
        Ok(Self {
            channel_id: [channel_id],
            frequency,
        })
    }

    pub fn channel_id(&self) -> usize {
        self.channel_id[0]
    }

    pub fn frequency(&self) -> f64 {
        self.frequency
    }
}

impl SimpleElement for SetFreq {
    fn channels(&self) -> &[usize] {
        &self.channel_id
    }
}

#[derive(Debug, Clone)]
pub struct SwapPhase {
    channel_ids: [usize; 2],
}

impl SwapPhase {
    pub fn new(channel_id1: usize, channel_id2: usize) -> Self {
        Self {
            channel_ids: [channel_id1, channel_id2],
        }
    }

    pub fn channel_id1(&self) -> usize {
        self.channel_ids[0]
    }

    pub fn channel_id2(&self) -> usize {
        self.channel_ids[1]
    }
}

impl SimpleElement for SwapPhase {
    fn channels(&self) -> &[usize] {
        &self.channel_ids
    }
}

#[derive(Debug, Clone)]
pub struct Barrier {
    channel_ids: Vec<usize>,
}

impl Barrier {
    pub fn new(channel_ids: Vec<usize>) -> Self {
        Self { channel_ids }
    }

    pub fn channel_ids(&self) -> &[usize] {
        &self.channel_ids
    }
}

impl SimpleElement for Barrier {
    fn channels(&self) -> &[usize] {
        &self.channel_ids
    }
}
