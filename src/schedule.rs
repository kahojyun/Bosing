mod absolute;
mod grid;
mod play;
mod repeat;
mod simple;
mod stack;

use std::sync::Arc;

use anyhow::{Result, bail};
use hashbrown::HashSet;
#[cfg(test)]
use mockall::automock;

use crate::quant::{ChannelId, Label, Time};

pub use self::{
    absolute::{Absolute, Entry as AbsoluteEntry},
    grid::{Entry as GridEntry, Grid, Length as GridLength},
    play::Play,
    repeat::Repeat,
    simple::{Barrier, SetFreq, SetPhase, ShiftFreq, ShiftPhase, SwapPhase},
    stack::{Direction, Stack},
};

pub type ElementRef = Arc<Element>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    End,
    Start,
    Center,
    Stretch,
}

#[derive(Debug, Clone)]
pub struct Element {
    pub common: ElementCommon,
    pub variant: ElementVariant,
}

#[derive(Debug, Clone)]
pub struct ElementCommon {
    margin: (Time, Time),
    alignment: Alignment,
    phantom: bool,
    duration: Option<Time>,
    max_duration: Time,
    min_duration: Time,
    label: Option<Label>,
}

#[derive(Debug, Clone)]
pub struct ElementCommonBuilder(ElementCommon);

#[derive(Debug, Clone, Copy)]
pub struct TimeRange {
    pub start: Time,
    pub span: Time,
}

#[derive(Debug, Clone, Copy)]
pub struct Arranged<T> {
    pub item: T,
    pub time_range: TimeRange,
}

#[cfg_attr(test, automock)]
pub trait Measure {
    fn measure(&self) -> Time;
    fn channels(&self) -> &[ChannelId];
}

pub trait Arrange {
    fn arrange(&self, time_range: TimeRange) -> impl Iterator<Item = Arranged<&ElementRef>>;
}

#[derive(Debug)]
struct MinMax {
    min: Time,
    max: Time,
}

macro_rules! impl_variant {
    ($($variant:ident),*$(,)?) => {
        #[derive(Debug, Clone)]
        pub enum ElementVariant {
            $($variant($variant),)*
        }

        $(
        impl From<$variant> for ElementVariant {
            fn from(v: $variant) -> Self {
                Self::$variant(v)
            }
        }

        impl TryFrom<ElementVariant> for $variant {
            type Error = anyhow::Error;

            fn try_from(value: ElementVariant) -> Result<Self, Self::Error> {
                match value {
                    ElementVariant::$variant(v) => Ok(v),
                    _ => bail!("Expected {} variant", stringify!($variant)),
                }
            }
        }

        impl<'a> TryFrom<&'a ElementVariant> for &'a $variant {
            type Error = anyhow::Error;

            fn try_from(value: &'a ElementVariant) -> Result<Self, Self::Error> {
                match value {
                    ElementVariant::$variant(v) => Ok(v),
                    _ => bail!("Expected {} variant", stringify!($variant)),
                }
            }
        }
        )*

        impl Measure for ElementVariant {
            fn measure(&self) -> Time {
                match self {
                    $(ElementVariant::$variant(v) => v.measure(),)*
                }
            }

            fn channels(&self) -> &[ChannelId] {
                match self {
                    $(ElementVariant::$variant(v) => v.channels(),)*
                }
            }
        }
    };
}

impl_variant!(
    Play, ShiftPhase, SetPhase, ShiftFreq, SetFreq, SwapPhase, Barrier, Repeat, Stack, Absolute,
    Grid,
);

impl Element {
    pub fn new(common: ElementCommon, variant: impl Into<ElementVariant>) -> Self {
        Self {
            common,
            variant: variant.into(),
        }
    }

    pub fn inner_time_range(&self, time_range: TimeRange) -> TimeRange {
        let min_max = self.common.min_max_duration();
        let inner_start = time_range.start + self.common.margin.0;
        let inner_span = min_max.clamp(time_range.span - self.common.total_margin());
        TimeRange {
            start: inner_start,
            span: inner_span,
        }
    }
}

impl ElementCommon {
    #[must_use]
    pub const fn margin(&self) -> (Time, Time) {
        self.margin
    }

    #[must_use]
    pub const fn alignment(&self) -> Alignment {
        self.alignment
    }

    #[must_use]
    pub const fn phantom(&self) -> bool {
        self.phantom
    }

    #[must_use]
    pub const fn duration(&self) -> Option<Time> {
        self.duration
    }

    #[must_use]
    pub const fn max_duration(&self) -> Time {
        self.max_duration
    }

    #[must_use]
    pub const fn min_duration(&self) -> Time {
        self.min_duration
    }

    #[must_use]
    pub const fn label(&self) -> Option<&Label> {
        self.label.as_ref()
    }

    fn min_max_duration(&self) -> MinMax {
        let min_max = MinMax::new(self.min_duration, self.max_duration);
        let max = min_max.clamp(self.duration.unwrap_or(Time::INFINITY));
        let min = min_max.clamp(self.duration.unwrap_or(Time::ZERO));
        MinMax::new(min, max)
    }

    fn total_margin(&self) -> Time {
        self.margin.0 + self.margin.1
    }
}

impl ElementCommonBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub const fn margin(&mut self, margin: (Time, Time)) -> &mut Self {
        self.0.margin = margin;
        self
    }

    pub const fn alignment(&mut self, alignment: Alignment) -> &mut Self {
        self.0.alignment = alignment;
        self
    }

    pub const fn phantom(&mut self, phantom: bool) -> &mut Self {
        self.0.phantom = phantom;
        self
    }

    pub const fn duration(&mut self, duration: Option<Time>) -> &mut Self {
        self.0.duration = duration;
        self
    }

    pub const fn max_duration(&mut self, max_duration: Time) -> &mut Self {
        self.0.max_duration = max_duration;
        self
    }

    pub const fn min_duration(&mut self, min_duration: Time) -> &mut Self {
        self.0.min_duration = min_duration;
        self
    }

    pub fn label(&mut self, label: Option<Label>) -> &mut Self {
        self.0.label = label;
        self
    }

    pub fn validate(&self) -> Result<()> {
        let v = &self.0;
        if !(v.margin.0.value().is_finite() && v.margin.1.value().is_finite()) {
            bail!("Invalid margin {margin:?}", margin = v.margin);
        }
        if let Some(v) = v.duration {
            if !(v.value().is_finite() && v >= Time::ZERO) {
                bail!("Invalid duration {v:?}");
            }
        }
        if !(v.min_duration.value().is_finite() && v.min_duration >= Time::ZERO) {
            bail!(
                "Invalid min_duration {min_duration:?}",
                min_duration = v.min_duration
            );
        }
        if v.max_duration < Time::ZERO {
            bail!(
                "Invalid max_duration {max_duration:?}",
                max_duration = v.max_duration
            );
        }
        Ok(())
    }

    pub fn build(&self) -> Result<ElementCommon> {
        self.validate()?;
        Ok(self.0.clone())
    }
}

impl MinMax {
    const fn new(min: Time, max: Time) -> Self {
        Self { min, max }
    }

    fn clamp(&self, value: Time) -> Time {
        value.min(self.max).max(self.min)
    }
}

impl Default for ElementCommonBuilder {
    fn default() -> Self {
        Self(ElementCommon {
            margin: Default::default(),
            alignment: Alignment::End,
            phantom: false,
            duration: None,
            max_duration: Time::INFINITY,
            min_duration: Time::default(),
            label: None,
        })
    }
}

impl Measure for Element {
    fn measure(&self) -> Time {
        let inner_duration = self.variant.measure();
        let min_max = self.common.min_max_duration();
        let duration = min_max.clamp(inner_duration) + self.common.total_margin();
        duration.max(Time::ZERO)
    }

    fn channels(&self) -> &[ChannelId] {
        self.variant.channels()
    }
}

impl Measure for ElementRef {
    fn measure(&self) -> Time {
        (**self).measure()
    }

    fn channels(&self) -> &[ChannelId] {
        (**self).channels()
    }
}

impl<T> Measure for &T
where
    T: Measure + ?Sized,
{
    fn measure(&self) -> Time {
        (*self).measure()
    }

    fn channels(&self) -> &[ChannelId] {
        (*self).channels()
    }
}

fn merge_channel_ids<'a, I>(ids: I) -> Vec<ChannelId>
where
    I: IntoIterator<Item: IntoIterator<Item = &'a ChannelId>>,
{
    let set = ids.into_iter().flatten().collect::<HashSet<_>>();
    set.into_iter().cloned().collect()
}
