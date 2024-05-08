use anyhow::{bail, Result};

use super::{
    ArrangeContext, ArrangeResult, ArrangeResultVariant, MeasureResult, MeasureResultVariant,
    Schedule,
};
use crate::quant::{ChannelId, Frequency, Phase, Time};

trait SimpleElement {
    fn channels(&self) -> &[ChannelId];
}

impl<T> Schedule for T
where
    T: SimpleElement,
{
    fn measure(&self) -> MeasureResult {
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
pub(crate) struct ShiftPhase {
    channel_id: [ChannelId; 1],
    phase: Phase,
}

impl ShiftPhase {
    pub(crate) fn new(channel_id: ChannelId, phase: Phase) -> Result<Self> {
        if !phase.value().is_finite() {
            bail!("Invalid phase {:?}", phase);
        }
        Ok(Self {
            channel_id: [channel_id],
            phase,
        })
    }

    pub(crate) fn channel_id(&self) -> &ChannelId {
        &self.channel_id[0]
    }

    pub(crate) fn phase(&self) -> Phase {
        self.phase
    }
}

impl SimpleElement for ShiftPhase {
    fn channels(&self) -> &[ChannelId] {
        &self.channel_id
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SetPhase {
    channel_id: [ChannelId; 1],
    phase: Phase,
}

impl SetPhase {
    pub(crate) fn new(channel_id: ChannelId, phase: Phase) -> Result<Self> {
        if !phase.value().is_finite() {
            bail!("Invalid phase {:?}", phase);
        }
        Ok(Self {
            channel_id: [channel_id],
            phase,
        })
    }

    pub(crate) fn channel_id(&self) -> &ChannelId {
        &self.channel_id[0]
    }

    pub(crate) fn phase(&self) -> Phase {
        self.phase
    }
}

impl SimpleElement for SetPhase {
    fn channels(&self) -> &[ChannelId] {
        &self.channel_id
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ShiftFreq {
    channel_id: [ChannelId; 1],
    frequency: Frequency,
}

impl ShiftFreq {
    pub(crate) fn new(channel_id: ChannelId, frequency: Frequency) -> Result<Self> {
        if !frequency.value().is_finite() {
            bail!("Invalid frequency {:?}", frequency);
        }
        Ok(Self {
            channel_id: [channel_id],
            frequency,
        })
    }

    pub(crate) fn channel_id(&self) -> &ChannelId {
        &self.channel_id[0]
    }

    pub(crate) fn frequency(&self) -> Frequency {
        self.frequency
    }
}

impl SimpleElement for ShiftFreq {
    fn channels(&self) -> &[ChannelId] {
        &self.channel_id
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SetFreq {
    channel_id: [ChannelId; 1],
    frequency: Frequency,
}

impl SetFreq {
    pub(crate) fn new(channel_id: ChannelId, frequency: Frequency) -> Result<Self> {
        if !frequency.value().is_finite() {
            bail!("Invalid frequency {:?}", frequency);
        }
        Ok(Self {
            channel_id: [channel_id],
            frequency,
        })
    }

    pub(crate) fn channel_id(&self) -> &ChannelId {
        &self.channel_id[0]
    }

    pub(crate) fn frequency(&self) -> Frequency {
        self.frequency
    }
}

impl SimpleElement for SetFreq {
    fn channels(&self) -> &[ChannelId] {
        &self.channel_id
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SwapPhase {
    channel_ids: [ChannelId; 2],
}

impl SwapPhase {
    pub(crate) fn new(channel_id1: ChannelId, channel_id2: ChannelId) -> Self {
        Self {
            channel_ids: [channel_id1, channel_id2],
        }
    }

    pub(crate) fn channel_id1(&self) -> &ChannelId {
        &self.channel_ids[0]
    }

    pub(crate) fn channel_id2(&self) -> &ChannelId {
        &self.channel_ids[1]
    }
}

impl SimpleElement for SwapPhase {
    fn channels(&self) -> &[ChannelId] {
        &self.channel_ids
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Barrier {
    channel_ids: Vec<ChannelId>,
}

impl Barrier {
    pub(crate) fn new(channel_ids: Vec<ChannelId>) -> Self {
        Self { channel_ids }
    }

    pub(crate) fn channel_ids(&self) -> &[ChannelId] {
        &self.channel_ids
    }
}

impl SimpleElement for Barrier {
    fn channels(&self) -> &[ChannelId] {
        &self.channel_ids
    }
}
