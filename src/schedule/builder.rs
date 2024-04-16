use std::rc::Rc;

use super::{
    grid::{Grid, GridEntry, GridLength},
    stack::{Direction, Stack},
    Absolute, AbsoluteEntry, Alignment, Barrier, Element, ElementCommon, ElementVariant, Play,
    Repeat, Schedule, SetFreq, SetPhase, ShiftFreq, ShiftPhase, SwapPhase,
};
use anyhow::{bail, Result};
use itertools::Itertools;

macro_rules! impl_setter {
    ($path:tt, $($name:ident: $type:ty), *$(,)?) => {
        $(
            pub fn $name(&mut self, $name: $type) -> &mut Self {
                self.$path.$name = $name;
                self
            }
        )*
    };
}

macro_rules! delegate_setter {
    ($path:tt, $($name:ident: $type:ty), *$(,)?) => {
        $(
            pub fn $name(&mut self, $name: $type) -> &mut Self {
                self.$path.$name($name);
                self
            }
        )*
    };
}

macro_rules! delegate_common_builder {
    ($path:tt) => {
        delegate_setter!(
            $path,
            margin: (f64, f64),
            alignment: Alignment,
            phantom: bool,
            duration: Option<f64>,
            max_duration: f64,
            min_duration: f64,
        );
    };
}

#[derive(Debug, Clone)]
struct ElementCommonBuilder(ElementCommon);

impl Default for ElementCommonBuilder {
    fn default() -> Self {
        Self(ElementCommon {
            margin: (0.0, 0.0),
            alignment: Alignment::End,
            phantom: false,
            duration: None,
            max_duration: f64::INFINITY,
            min_duration: 0.0,
        })
    }
}

impl ElementCommonBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    impl_setter!(
        0,
        margin: (f64, f64),
        alignment: Alignment,
        phantom: bool,
        duration: Option<f64>,
        max_duration: f64,
        min_duration: f64,
    );

    pub fn validate(&self) -> Result<()> {
        let v = &self.0;
        if !v.margin.0.is_finite() || !v.margin.1.is_finite() {
            bail!("Invalid margin {:?}", v.margin);
        }
        if let Some(v) = v.duration {
            if !(v >= 0.0) {
                bail!("Invalid duration {}", v);
            }
        }
        if !(v.min_duration >= 0.0) {
            bail!("Invalid min_duration {}", v.min_duration);
        }
        if !(v.max_duration >= 0.0) {
            bail!("Invalid max_duration {}", v.max_duration);
        }
        Ok(())
    }

    pub fn build(&self) -> Result<ElementCommon> {
        self.validate()?;
        Ok(self.0.to_owned())
    }
}

#[derive(Debug, Clone)]
pub struct PlayBuilder {
    common: ElementCommonBuilder,
    variant: Play,
}

impl PlayBuilder {
    pub fn new(channel_id: usize, shape_id: Option<usize>, amplitude: f64, width: f64) -> Self {
        Self {
            common: ElementCommonBuilder::new(),
            variant: Play {
                channel_id: [channel_id],
                shape_id,
                amplitude,
                width,
                plateau: 0.0,
                drag_coef: 0.0,
                frequency: 0.0,
                phase: 0.0,
                flexible: false,
            },
        }
    }

    delegate_common_builder!(common);

    impl_setter!(
        variant,
        plateau: f64,
        drag_coef: f64,
        frequency: f64,
        phase: f64,
        flexible: bool,
    );

    pub fn validate(&self) -> Result<()> {
        let play = &self.variant;
        if !(play.width >= 0.0) {
            bail!("Invalid width {}", play.width);
        }
        if !(play.plateau >= 0.0) {
            bail!("Invalid plateau {}", play.plateau);
        }
        if !play.drag_coef.is_finite() {
            bail!("Invalid drag_coef {}", play.drag_coef);
        }
        if !play.frequency.is_finite() {
            bail!("Invalid frequency {}", play.frequency);
        }
        if !play.phase.is_finite() {
            bail!("Invalid phase {}", play.phase);
        }
        Ok(())
    }

    pub fn build(&self) -> Result<Element> {
        let common = self.common.build()?;
        self.validate()?;
        Ok(Element {
            common,
            variant: ElementVariant::Play(self.variant.clone()),
        })
    }
}

macro_rules! simple_builder {
    ($builder:ident, $inst:ident, $param:ident) => {
        #[derive(Debug, Clone)]
        pub struct $builder {
            common: ElementCommonBuilder,
            variant: $inst,
        }

        impl $builder {
            pub fn new(channel_id: usize, $param: f64) -> Self {
                Self {
                    common: ElementCommonBuilder::new(),
                    variant: $inst {
                        channel_id: [channel_id],
                        $param,
                    },
                }
            }

            delegate_common_builder!(common);

            pub fn validate(&self) -> Result<()> {
                let v = &self.variant;
                if !v.$param.is_finite() {
                    bail!(concat!("Invalid ", stringify!($param), " {}"), v.$param);
                }
                Ok(())
            }

            pub fn build(&self) -> Result<Element> {
                let common = self.common.build()?;
                self.validate()?;
                Ok(Element {
                    common,
                    variant: ElementVariant::$inst(self.variant.clone()),
                })
            }
        }
    };
}

simple_builder!(ShiftPhaseBuilder, ShiftPhase, phase);
simple_builder!(SetPhaseBuilder, SetPhase, phase);
simple_builder!(ShiftFreqBuilder, ShiftFreq, frequency);
simple_builder!(SetFreqBuilder, SetFreq, frequency);

#[derive(Debug, Clone)]
pub struct SwapPhaseBuilder {
    common: ElementCommonBuilder,
    variant: SwapPhase,
}

impl SwapPhaseBuilder {
    pub fn new(channel_id1: usize, channel_id2: usize) -> Self {
        Self {
            common: ElementCommonBuilder::new(),
            variant: SwapPhase {
                channel_ids: [channel_id1, channel_id2],
            },
        }
    }

    delegate_common_builder!(common);

    pub fn build(&self) -> Result<Element> {
        let common = self.common.build()?;
        Ok(Element {
            common,
            variant: ElementVariant::SwapPhase(self.variant.clone()),
        })
    }
}

#[derive(Debug, Clone)]
pub struct BarrierBuilder {
    common: ElementCommonBuilder,
    variant: Barrier,
}

impl BarrierBuilder {
    pub fn new(channel_ids: Vec<usize>) -> Self {
        Self {
            common: ElementCommonBuilder::new(),
            variant: Barrier { channel_ids },
        }
    }

    delegate_common_builder!(common);

    pub fn build(&self) -> Result<Element> {
        let common = self.common.build()?;
        Ok(Element {
            common,
            variant: ElementVariant::Barrier(self.variant.clone()),
        })
    }
}

#[derive(Debug, Clone)]
pub struct RepeatBuilder {
    common: ElementCommonBuilder,
    variant: Repeat,
}

impl RepeatBuilder {
    pub fn new(child: Rc<Element>, count: usize) -> Self {
        Self {
            common: ElementCommonBuilder::new(),
            variant: Repeat {
                child,
                count,
                spacing: 0.0,
            },
        }
    }

    delegate_common_builder!(common);

    impl_setter!(variant, spacing: f64);

    pub fn build(&self) -> Result<Element> {
        let common = self.common.build()?;
        Ok(Element {
            common,
            variant: ElementVariant::Repeat(self.variant.clone()),
        })
    }
}

#[derive(Debug, Clone)]
pub struct StackBuilder {
    common: ElementCommonBuilder,
    variant: Stack,
}

impl StackBuilder {
    pub fn new(children: Vec<Rc<Element>>) -> Self {
        let channel_ids = children
            .iter()
            .flat_map(|e| e.variant.channels())
            .copied()
            .unique()
            .collect();
        Self {
            common: ElementCommonBuilder::new(),
            variant: Stack {
                children,
                direction: Direction::Backward,
                channel_ids,
            },
        }
    }

    delegate_common_builder!(common);

    impl_setter!(variant, direction: Direction);

    pub fn build(&self) -> Result<Element> {
        let common = self.common.build()?;
        Ok(Element {
            common,
            variant: ElementVariant::Stack(self.variant.clone()),
        })
    }
}

#[derive(Debug, Clone)]
pub struct AbsoluteBuilder {
    common: ElementCommonBuilder,
    variant: Absolute,
}

impl AbsoluteBuilder {
    pub fn new(children: Vec<AbsoluteEntry>) -> Self {
        let channel_ids = children
            .iter()
            .flat_map(|e| e.element.variant.channels())
            .copied()
            .unique()
            .collect();
        Self {
            common: ElementCommonBuilder::new(),
            variant: Absolute {
                children,
                channel_ids,
            },
        }
    }

    delegate_common_builder!(common);

    pub fn build(&self) -> Result<Element> {
        let common = self.common.build()?;
        Ok(Element {
            common,
            variant: ElementVariant::Absolute(self.variant.clone()),
        })
    }
}

#[derive(Debug, Clone)]
pub struct GridBuilder {
    common: ElementCommonBuilder,
    variant: Grid,
}

impl GridBuilder {
    pub fn new(children: Vec<GridEntry>) -> Self {
        let channel_ids = children
            .iter()
            .flat_map(|e| e.element.variant.channels())
            .copied()
            .unique()
            .collect();
        Self {
            common: ElementCommonBuilder::new(),
            variant: Grid {
                children,
                columns: vec![],
                channel_ids,
            },
        }
    }

    delegate_common_builder!(common);

    impl_setter!(variant, columns: Vec<GridLength>);

    pub fn build(&self) -> Result<Element> {
        let common = self.common.build()?;
        let mut grid = self.variant.clone();
        if grid.columns.is_empty() {
            grid.columns = vec![GridLength::star(1.0)]
        }
        Ok(Element {
            common,
            variant: ElementVariant::Grid(grid),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder() {
        let ec = ElementCommonBuilder::new()
            .alignment(Alignment::Center)
            .build();
        println!("{:?}", ec);
    }

    #[test]
    fn test_build_play() {
        let play = PlayBuilder::new(0, None, 0.0, 1.0)
            .plateau(0.5)
            .drag_coef(0.1)
            .frequency(0.2)
            .phase(0.3)
            .flexible(true)
            .build();
        println!("{:?}", play);
    }
}
