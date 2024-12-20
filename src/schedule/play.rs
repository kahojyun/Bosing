use anyhow::{bail, Result};

use crate::quant::{Amplitude, ChannelId, Frequency, Phase, ShapeId, Time};

use super::Measure;

#[derive(Debug, Clone)]
pub struct Play {
    channel_id: [ChannelId; 1],
    shape_id: Option<ShapeId>,
    amplitude: Amplitude,
    width: Time,
    plateau: Time,
    drag_coef: f64,
    frequency: Frequency,
    phase: Phase,
    flexible: bool,
}

impl Play {
    pub fn new(
        channel_id: ChannelId,
        shape_id: Option<ShapeId>,
        amplitude: Amplitude,
        width: Time,
    ) -> Result<Self> {
        if !amplitude.value().is_finite() {
            bail!("Invalid amplitude {:?}", amplitude);
        }
        if !width.value().is_finite() || width.value() < 0.0 {
            bail!("Invalid width {:?}", width);
        }
        Ok(Self {
            channel_id: [channel_id],
            shape_id,
            amplitude,
            width,
            plateau: Time::ZERO,
            drag_coef: 0.0,
            frequency: Frequency::ZERO,
            phase: Phase::ZERO,
            flexible: false,
        })
    }

    pub fn with_plateau(mut self, plateau: Time) -> Result<Self> {
        if !plateau.value().is_finite() || plateau.value() < 0.0 {
            bail!("Invalid plateau {:?}", plateau);
        }
        self.plateau = plateau;
        Ok(self)
    }

    pub fn with_drag_coef(mut self, drag_coef: f64) -> Result<Self> {
        if !drag_coef.is_finite() {
            bail!("Invalid drag_coef {}", drag_coef);
        }
        self.drag_coef = drag_coef;
        Ok(self)
    }

    pub fn with_frequency(mut self, frequency: Frequency) -> Result<Self> {
        if !frequency.value().is_finite() {
            bail!("Invalid frequency {:?}", frequency);
        }
        self.frequency = frequency;
        Ok(self)
    }

    pub fn with_phase(mut self, phase: Phase) -> Result<Self> {
        if !phase.value().is_finite() {
            bail!("Invalid phase {:?}", phase);
        }
        self.phase = phase;
        Ok(self)
    }

    #[must_use]
    pub const fn with_flexible(mut self, flexible: bool) -> Self {
        self.flexible = flexible;
        self
    }

    #[must_use]
    pub const fn channel_id(&self) -> &ChannelId {
        &self.channel_id[0]
    }

    #[must_use]
    pub const fn shape_id(&self) -> Option<&ShapeId> {
        self.shape_id.as_ref()
    }

    #[must_use]
    pub const fn amplitude(&self) -> Amplitude {
        self.amplitude
    }

    #[must_use]
    pub const fn width(&self) -> Time {
        self.width
    }

    #[must_use]
    pub const fn plateau(&self) -> Time {
        self.plateau
    }

    #[must_use]
    pub const fn drag_coef(&self) -> f64 {
        self.drag_coef
    }

    #[must_use]
    pub const fn frequency(&self) -> Frequency {
        self.frequency
    }

    #[must_use]
    pub const fn phase(&self) -> Phase {
        self.phase
    }

    #[must_use]
    pub const fn flexible(&self) -> bool {
        self.flexible
    }
}

impl Measure for Play {
    fn channels(&self) -> &[ChannelId] {
        &self.channel_id
    }

    fn measure(&self) -> Time {
        if self.flexible {
            self.width
        } else {
            self.width + self.plateau
        }
    }
}
