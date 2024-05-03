use anyhow::{bail, Result};

use super::{
    ArrangeContext, ArrangeResult, ArrangeResultVariant, MeasureContext, MeasureResult,
    MeasureResultVariant, Schedule,
};
use crate::quant::{Amplitude, ChannelId, Frequency, Phase, ShapeId, Time};

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

    pub fn with_flexible(mut self, flexible: bool) -> Self {
        self.flexible = flexible;
        self
    }

    pub fn channel_id(&self) -> &ChannelId {
        &self.channel_id[0]
    }

    pub fn shape_id(&self) -> Option<&ShapeId> {
        self.shape_id.as_ref()
    }

    pub fn amplitude(&self) -> Amplitude {
        self.amplitude
    }

    pub fn width(&self) -> Time {
        self.width
    }

    pub fn plateau(&self) -> Time {
        self.plateau
    }

    pub fn drag_coef(&self) -> f64 {
        self.drag_coef
    }

    pub fn frequency(&self) -> Frequency {
        self.frequency
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn flexible(&self) -> bool {
        self.flexible
    }
}

impl Schedule for Play {
    fn measure(&self, _context: &MeasureContext) -> MeasureResult {
        let wanted_duration = if self.flexible {
            self.width
        } else {
            self.width + self.plateau
        };
        MeasureResult(wanted_duration, MeasureResultVariant::Simple)
    }

    fn arrange(&self, context: &ArrangeContext) -> Result<ArrangeResult> {
        let arranged = if self.flexible {
            context.final_duration
        } else {
            self.width + self.plateau
        };
        Ok(ArrangeResult(arranged, ArrangeResultVariant::Simple))
    }

    fn channels(&self) -> &[ChannelId] {
        &self.channel_id
    }
}
