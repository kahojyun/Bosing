use anyhow::{Result, bail};

use crate::quant::{ChannelId, Frequency, Phase, Time};

use super::Measure;

#[derive(Debug, Clone)]
pub struct ShiftPhase {
    channel_ids: [ChannelId; 1],
    phase: Phase,
}

#[derive(Debug, Clone)]
pub struct SetPhase {
    channel_ids: [ChannelId; 1],
    phase: Phase,
}

#[derive(Debug, Clone)]
pub struct ShiftFreq {
    channel_ids: [ChannelId; 1],
    frequency: Frequency,
}

#[derive(Debug, Clone)]
pub struct SetFreq {
    channel_ids: [ChannelId; 1],
    frequency: Frequency,
}

#[derive(Debug, Clone)]
pub struct SwapPhase {
    channel_ids: [ChannelId; 2],
}

#[derive(Debug, Clone)]
pub struct Barrier {
    channel_ids: Vec<ChannelId>,
}

impl ShiftPhase {
    pub fn new(channel_id: ChannelId, phase: Phase) -> Result<Self> {
        if !phase.value().is_finite() {
            bail!("Invalid phase {phase:?}");
        }
        Ok(Self {
            channel_ids: [channel_id],
            phase,
        })
    }

    #[must_use]
    pub const fn channel_id(&self) -> &ChannelId {
        &self.channel_ids[0]
    }

    #[must_use]
    pub const fn phase(&self) -> Phase {
        self.phase
    }
}

impl SetPhase {
    pub fn new(channel_id: ChannelId, phase: Phase) -> Result<Self> {
        if !phase.value().is_finite() {
            bail!("Invalid phase {phase:?}");
        }
        Ok(Self {
            channel_ids: [channel_id],
            phase,
        })
    }

    #[must_use]
    pub const fn channel_id(&self) -> &ChannelId {
        &self.channel_ids[0]
    }

    #[must_use]
    pub const fn phase(&self) -> Phase {
        self.phase
    }
}

impl ShiftFreq {
    pub fn new(channel_id: ChannelId, frequency: Frequency) -> Result<Self> {
        if !frequency.value().is_finite() {
            bail!("Invalid frequency {frequency:?}");
        }
        Ok(Self {
            channel_ids: [channel_id],
            frequency,
        })
    }

    #[must_use]
    pub const fn channel_id(&self) -> &ChannelId {
        &self.channel_ids[0]
    }

    #[must_use]
    pub const fn frequency(&self) -> Frequency {
        self.frequency
    }
}

impl SetFreq {
    pub fn new(channel_id: ChannelId, frequency: Frequency) -> Result<Self> {
        if !frequency.value().is_finite() {
            bail!("Invalid frequency {frequency:?}");
        }
        Ok(Self {
            channel_ids: [channel_id],
            frequency,
        })
    }

    #[must_use]
    pub const fn channel_id(&self) -> &ChannelId {
        &self.channel_ids[0]
    }

    #[must_use]
    pub const fn frequency(&self) -> Frequency {
        self.frequency
    }
}

impl SwapPhase {
    #[must_use]
    pub const fn new(channel_id1: ChannelId, channel_id2: ChannelId) -> Self {
        Self {
            channel_ids: [channel_id1, channel_id2],
        }
    }

    #[must_use]
    pub const fn channel_id1(&self) -> &ChannelId {
        &self.channel_ids[0]
    }

    #[must_use]
    pub const fn channel_id2(&self) -> &ChannelId {
        &self.channel_ids[1]
    }
}

impl Barrier {
    #[must_use]
    pub const fn new(channel_ids: Vec<ChannelId>) -> Self {
        Self { channel_ids }
    }

    #[must_use]
    pub fn channel_ids(&self) -> &[ChannelId] {
        &self.channel_ids
    }
}

macro_rules! impl_measure {
    ($t:ty) => {
        impl Measure for $t {
            fn measure(&self) -> Time {
                Time::ZERO
            }

            fn channels(&self) -> &[ChannelId] {
                &self.channel_ids
            }
        }
    };
}

impl_measure!(ShiftPhase);
impl_measure!(SetPhase);
impl_measure!(ShiftFreq);
impl_measure!(SetFreq);
impl_measure!(SwapPhase);
impl_measure!(Barrier);
