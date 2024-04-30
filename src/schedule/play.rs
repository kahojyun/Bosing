use anyhow::{bail, Result};

use super::{
    ArrangeContext, ArrangeResult, ArrangeResultVariant, MeasureContext, MeasureResult,
    MeasureResultVariant, Schedule,
};

#[derive(Debug, Clone)]
pub struct Play {
    channel_id: [String; 1],
    shape_id: Option<String>,
    amplitude: f64,
    width: f64,
    plateau: f64,
    drag_coef: f64,
    frequency: f64,
    phase: f64,
    flexible: bool,
}

impl Play {
    pub fn new(
        channel_id: String,
        shape_id: Option<String>,
        amplitude: f64,
        width: f64,
    ) -> Result<Self> {
        if !amplitude.is_finite() {
            bail!("Invalid amplitude {}", amplitude);
        }
        if !width.is_finite() || width < 0.0 {
            bail!("Invalid width {}", width);
        }
        Ok(Self {
            channel_id: [channel_id],
            shape_id,
            amplitude,
            width,
            plateau: 0.0,
            drag_coef: 0.0,
            frequency: 0.0,
            phase: 0.0,
            flexible: false,
        })
    }

    pub fn with_plateau(mut self, plateau: f64) -> Result<Self> {
        if !plateau.is_finite() || plateau < 0.0 {
            bail!("Invalid plateau {}", plateau);
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

    pub fn with_frequency(mut self, frequency: f64) -> Result<Self> {
        if !frequency.is_finite() {
            bail!("Invalid frequency {}", frequency);
        }
        self.frequency = frequency;
        Ok(self)
    }

    pub fn with_phase(mut self, phase: f64) -> Result<Self> {
        if !phase.is_finite() {
            bail!("Invalid phase {}", phase);
        }
        self.phase = phase;
        Ok(self)
    }

    pub fn with_flexible(mut self, flexible: bool) -> Self {
        self.flexible = flexible;
        self
    }

    pub fn channel_id(&self) -> &str {
        &self.channel_id[0]
    }

    pub fn shape_id(&self) -> Option<&str> {
        self.shape_id.as_deref()
    }

    pub fn amplitude(&self) -> f64 {
        self.amplitude
    }

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn plateau(&self) -> f64 {
        self.plateau
    }

    pub fn drag_coef(&self) -> f64 {
        self.drag_coef
    }

    pub fn frequency(&self) -> f64 {
        self.frequency
    }

    pub fn phase(&self) -> f64 {
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

    fn channels(&self) -> &[String] {
        &self.channel_id
    }
}
