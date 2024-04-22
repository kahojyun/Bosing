use std::ops::Add;

use anyhow::{anyhow, Result};
use ordered_float::NotNan;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Time(NotNan<f64>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AlignedIndex(NotNan<f64>);

impl Time {
    pub fn new(value: f64) -> Result<Self> {
        Ok(Self(
            NotNan::new(value).map_err(|_| anyhow!("NaN in Time value"))?,
        ))
    }

    pub fn value(&self) -> f64 {
        self.0.into_inner()
    }
}

impl AlignedIndex {
    pub fn new(time: Time, sample_rate: f64, align_level: i32) -> Result<Self> {
        let scaled_sr = scaleb(sample_rate, -align_level);
        let i = (time.value() * scaled_sr).ceil();
        let aligned_index = scaleb(i, align_level);
        Ok(Self(
            NotNan::new(aligned_index).map_err(|_| anyhow!("NaN in AlignedIndex value"))?,
        ))
    }

    pub fn value(&self) -> f64 {
        self.0.into_inner()
    }

    pub fn ceil(&self) -> Self {
        Self(NotNan::new(self.0.ceil()).unwrap())
    }

    pub fn index_offset(&self) -> Self {
        Self(NotNan::new(self.0.ceil() - self.0.into_inner()).unwrap())
    }
}

fn scaleb(x: f64, s: i32) -> f64 {
    x * (s as f64).exp2()
}

impl Add for Time {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
