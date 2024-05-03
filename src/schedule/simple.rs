use anyhow::{bail, Result};

use super::{
    ArrangeContext, ArrangeResult, ArrangeResultVariant, MeasureContext, MeasureResult,
    MeasureResultVariant, Schedule,
};
use crate::quant::{ChannelId, Frequency, Phase, Time};

trait SimpleElement {
    fn channels(&self) -> &[ChannelId];
}

impl<T> Schedule for T
where
    T: SimpleElement,
{
    fn measure(&self, _context: &MeasureContext) -> MeasureResult {
        MeasureResult(Time::ZERO, MeasureResultVariant::Simple)
    }

    fn arrange(&self, _context: &ArrangeContext) -> Result<ArrangeResult> {
        Ok(ArrangeResult(Time::ZERO, ArrangeResultVariant::Simple))
    }

    fn channels(&self) -> &[ChannelId] {
        self.channels()
    }
}

#[derive(Debug, Clone)]
pub struct ShiftPhase {
    channel_id: [ChannelId; 1],
    phase: Phase,
}

impl ShiftPhase {
    pub fn new(channel_id: ChannelId, phase: Phase) -> Result<Self> {
        if !phase.value().is_finite() {
            bail!("Invalid phase {:?}", phase);
        }
        Ok(Self {
            channel_id: [channel_id],
            phase,
        })
    }

    pub fn channel_id(&self) -> &ChannelId {
        &self.channel_id[0]
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }
}

impl SimpleElement for ShiftPhase {
    fn channels(&self) -> &[ChannelId] {
        &self.channel_id
    }
}

#[derive(Debug, Clone)]
pub struct SetPhase {
    channel_id: [ChannelId; 1],
    phase: Phase,
}

impl SetPhase {
    pub fn new(channel_id: ChannelId, phase: Phase) -> Result<Self> {
        if !phase.value().is_finite() {
            bail!("Invalid phase {:?}", phase);
        }
        Ok(Self {
            channel_id: [channel_id],
            phase,
        })
    }

    pub fn channel_id(&self) -> &ChannelId {
        &self.channel_id[0]
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }
}

impl SimpleElement for SetPhase {
    fn channels(&self) -> &[ChannelId] {
        &self.channel_id
    }
}

#[derive(Debug, Clone)]
pub struct ShiftFreq {
    channel_id: [ChannelId; 1],
    frequency: Frequency,
}

impl ShiftFreq {
    pub fn new(channel_id: ChannelId, frequency: Frequency) -> Result<Self> {
        if !frequency.value().is_finite() {
            bail!("Invalid frequency {:?}", frequency);
        }
        Ok(Self {
            channel_id: [channel_id],
            frequency,
        })
    }

    pub fn channel_id(&self) -> &ChannelId {
        &self.channel_id[0]
    }

    pub fn frequency(&self) -> Frequency {
        self.frequency
    }
}

impl SimpleElement for ShiftFreq {
    fn channels(&self) -> &[ChannelId] {
        &self.channel_id
    }
}

#[derive(Debug, Clone)]
pub struct SetFreq {
    channel_id: [ChannelId; 1],
    frequency: Frequency,
}

impl SetFreq {
    pub fn new(channel_id: ChannelId, frequency: Frequency) -> Result<Self> {
        if !frequency.value().is_finite() {
            bail!("Invalid frequency {:?}", frequency);
        }
        Ok(Self {
            channel_id: [channel_id],
            frequency,
        })
    }

    pub fn channel_id(&self) -> &ChannelId {
        &self.channel_id[0]
    }

    pub fn frequency(&self) -> Frequency {
        self.frequency
    }
}

impl SimpleElement for SetFreq {
    fn channels(&self) -> &[ChannelId] {
        &self.channel_id
    }
}

#[derive(Debug, Clone)]
pub struct SwapPhase {
    channel_ids: [ChannelId; 2],
}

impl SwapPhase {
    pub fn new(channel_id1: ChannelId, channel_id2: ChannelId) -> Self {
        Self {
            channel_ids: [channel_id1, channel_id2],
        }
    }

    pub fn channel_id1(&self) -> &ChannelId {
        &self.channel_ids[0]
    }

    pub fn channel_id2(&self) -> &ChannelId {
        &self.channel_ids[1]
    }
}

impl SimpleElement for SwapPhase {
    fn channels(&self) -> &[ChannelId] {
        &self.channel_ids
    }
}

#[derive(Debug, Clone)]
pub struct Barrier {
    channel_ids: Vec<ChannelId>,
}

impl Barrier {
    pub fn new(channel_ids: Vec<ChannelId>) -> Self {
        Self { channel_ids }
    }

    pub fn channel_ids(&self) -> &[ChannelId] {
        &self.channel_ids
    }
}

impl SimpleElement for Barrier {
    fn channels(&self) -> &[ChannelId] {
        &self.channel_ids
    }
}
