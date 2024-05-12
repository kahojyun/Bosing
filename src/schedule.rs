mod absolute;
mod grid;
mod play;
mod repeat;
mod simple;
mod stack;

use std::{iter, sync::Arc};

use anyhow::{bail, Result};
use hashbrown::HashSet;
#[cfg(test)]
use mockall::automock;

use crate::{
    quant::{ChannelId, Time},
    Alignment,
};

pub(crate) use absolute::{Absolute, AbsoluteEntry};
pub(crate) use grid::{Grid, GridEntry};
pub(crate) use play::Play;
pub(crate) use repeat::Repeat;
pub(crate) use simple::{Barrier, SetFreq, SetPhase, ShiftFreq, ShiftPhase, SwapPhase};
pub(crate) use stack::Stack;

pub(crate) type ElementRef = Arc<Element>;

#[derive(Debug, Clone)]
pub(crate) struct Element {
    pub(crate) common: ElementCommon,
    pub(crate) variant: ElementVariant,
}

#[derive(Debug, Clone)]
pub(crate) struct ElementCommon {
    margin: (Time, Time),
    alignment: Alignment,
    phantom: bool,
    duration: Option<Time>,
    max_duration: Time,
    min_duration: Time,
}

#[derive(Debug, Clone)]
pub(crate) struct ElementCommonBuilder(ElementCommon);

#[derive(Debug, Clone, Copy)]
pub(crate) struct Arranged<T> {
    pub(crate) item: T,
    pub(crate) offset: Time,
    pub(crate) duration: Time,
}

pub(crate) trait Visitor {
    fn visit_play(&mut self, variant: &Play, time: Time, duration: Time) -> Result<()>;
    fn visit_shift_phase(&mut self, variant: &ShiftPhase, time: Time, duration: Time)
        -> Result<()>;
    fn visit_set_phase(&mut self, variant: &SetPhase, time: Time, duration: Time) -> Result<()>;
    fn visit_shift_freq(&mut self, variant: &ShiftFreq, time: Time, duration: Time) -> Result<()>;
    fn visit_set_freq(&mut self, variant: &SetFreq, time: Time, duration: Time) -> Result<()>;
    fn visit_swap_phase(&mut self, variant: &SwapPhase, time: Time, duration: Time) -> Result<()>;
    fn visit_barrier(&mut self, variant: &Barrier, time: Time, duration: Time) -> Result<()>;
    fn visit_repeat(&mut self, variant: &Repeat, time: Time, duration: Time) -> Result<()>;
    fn visit_stack(&mut self, variant: &Stack, time: Time, duration: Time) -> Result<()>;
    fn visit_absolute(&mut self, variant: &Absolute, time: Time, duration: Time) -> Result<()>;
    fn visit_grid(&mut self, variant: &Grid, time: Time, duration: Time) -> Result<()>;
    fn visit_common(&mut self, common: &ElementCommon, time: Time, duration: Time) -> Result<()>;
}

#[cfg_attr(test, automock)]
pub(crate) trait Measure {
    fn measure(&self) -> Time;
    fn channels(&self) -> &[ChannelId];
}

pub(crate) trait Visit {
    fn visit<V>(&self, visitor: &mut V, time: Time, duration: Time) -> Result<()>
    where
        V: Visitor;
}

pub(crate) trait Arrange<'a> {
    fn arrange(
        &'a self,
        time: Time,
        duration: Time,
    ) -> impl Iterator<Item = Arranged<&'a ElementRef>>;
}

#[derive(Debug)]
struct MinMax {
    min: Time,
    max: Time,
}

#[derive(Debug)]
enum IterVariant<S, A, G, R> {
    Stack(S),
    Absolute(A),
    Grid(G),
    Repeat(R),
}

macro_rules! impl_variant {
    ($($variant:ident),*$(,)?) => {
        #[derive(Debug, Clone)]
        pub(crate) enum ElementVariant {
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

        impl Visit for ElementVariant {
            fn visit<V>(&self, visitor: &mut V, time: Time, duration: Time) -> Result<()>
            where
                V: Visitor,
            {
                match self {
                    $(ElementVariant::$variant(v) => v.visit(visitor, time, duration),)*
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
    pub(crate) fn new(common: ElementCommon, variant: impl Into<ElementVariant>) -> Self {
        Self {
            common,
            variant: variant.into(),
        }
    }

    pub(crate) fn arrange_inner(&self, time: Time, duration: Time) -> Arranged<&ElementVariant> {
        let min_max = self.common.min_max_duration();
        let inner_time = time + self.common.margin.0;
        let inner_duration = (duration - self.common.total_margin()).max(Time::ZERO);
        let inner_duration = min_max.clamp(inner_duration);
        Arranged {
            item: &self.variant,
            offset: inner_time,
            duration: inner_duration,
        }
    }
}

pub(crate) fn arrange_tree(
    element: &ElementRef,
    time: Time,
    duration: Time,
) -> impl Iterator<Item = Arranged<&ElementRef>> {
    pre_order(
        Arranged {
            item: element,
            offset: time,
            duration,
        },
        |e| {
            let inner = e.item.arrange_inner(e.offset, e.duration);
            arrange_variant(inner.item, inner.offset, inner.duration)
        },
    )
}

fn arrange_variant(
    variant: &ElementVariant,
    time: Time,
    duration: Time,
) -> Option<impl Iterator<Item = Arranged<&ElementRef>>> {
    match variant {
        ElementVariant::Repeat(r) => Some(IterVariant::Repeat(r.arrange(time, duration))),
        ElementVariant::Stack(s) => Some(IterVariant::Stack(s.arrange(time, duration))),
        ElementVariant::Absolute(a) => Some(IterVariant::Absolute(a.arrange(time, duration))),
        ElementVariant::Grid(g) => Some(IterVariant::Grid(g.arrange(time, duration))),
        _ => None,
    }
}

impl ElementCommon {
    pub(crate) fn margin(&self) -> (Time, Time) {
        self.margin
    }

    pub(crate) fn alignment(&self) -> Alignment {
        self.alignment
    }

    pub(crate) fn phantom(&self) -> bool {
        self.phantom
    }

    pub(crate) fn duration(&self) -> Option<Time> {
        self.duration
    }

    pub(crate) fn max_duration(&self) -> Time {
        self.max_duration
    }

    pub(crate) fn min_duration(&self) -> Time {
        self.min_duration
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
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn margin(&mut self, margin: (Time, Time)) -> &mut Self {
        self.0.margin = margin;
        self
    }

    pub(crate) fn alignment(&mut self, alignment: Alignment) -> &mut Self {
        self.0.alignment = alignment;
        self
    }

    pub(crate) fn phantom(&mut self, phantom: bool) -> &mut Self {
        self.0.phantom = phantom;
        self
    }

    pub(crate) fn duration(&mut self, duration: Option<Time>) -> &mut Self {
        self.0.duration = duration;
        self
    }

    pub(crate) fn max_duration(&mut self, max_duration: Time) -> &mut Self {
        self.0.max_duration = max_duration;
        self
    }

    pub(crate) fn min_duration(&mut self, min_duration: Time) -> &mut Self {
        self.0.min_duration = min_duration;
        self
    }

    pub(crate) fn validate(&self) -> Result<()> {
        let v = &self.0;
        if !(v.margin.0.value().is_finite() && v.margin.1.value().is_finite()) {
            bail!("Invalid margin {:?}", v.margin);
        }
        if let Some(v) = v.duration {
            if !(v.value().is_finite() && v >= Time::ZERO) {
                bail!("Invalid duration {:?}", v);
            }
        }
        if !(v.min_duration.value().is_finite() && v.min_duration >= Time::ZERO) {
            bail!("Invalid min_duration {:?}", v.min_duration);
        }
        if v.max_duration < Time::ZERO {
            bail!("Invalid max_duration {:?}", v.max_duration);
        }
        Ok(())
    }

    pub(crate) fn build(&self) -> Result<ElementCommon> {
        self.validate()?;
        Ok(self.0.clone())
    }
}

impl MinMax {
    fn new(min: Time, max: Time) -> Self {
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
            min_duration: Default::default(),
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

impl Visit for Element {
    fn visit<V>(&self, visitor: &mut V, time: Time, duration: Time) -> Result<()>
    where
        V: Visitor,
    {
        visitor.visit_common(&self.common, time, duration)?;
        let min_max = self.common.min_max_duration();
        let inner_time = time + self.common.margin.0;
        let inner_duration = (duration - self.common.total_margin()).max(Time::ZERO);
        let inner_duration = min_max.clamp(inner_duration);
        self.variant.visit(visitor, inner_time, inner_duration)
    }
}

impl Visit for ElementRef {
    fn visit<V>(&self, visitor: &mut V, time: Time, duration: Time) -> Result<()>
    where
        V: Visitor,
    {
        (**self).visit(visitor, time, duration)
    }
}

impl<T> Visit for &T
where
    T: Visit + ?Sized,
{
    fn visit<V>(&self, visitor: &mut V, time: Time, duration: Time) -> Result<()>
    where
        V: Visitor,
    {
        (*self).visit(visitor, time, duration)
    }
}

impl<S, A, G, R, T> Iterator for IterVariant<S, A, G, R>
where
    S: Iterator<Item = T>,
    A: Iterator<Item = T>,
    G: Iterator<Item = T>,
    R: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IterVariant::Stack(s) => s.next(),
            IterVariant::Absolute(a) => a.next(),
            IterVariant::Grid(g) => g.next(),
            IterVariant::Repeat(r) => r.next(),
        }
    }
}

fn merge_channel_ids<'a, I>(ids: I) -> Vec<ChannelId>
where
    I: IntoIterator,
    I::Item: IntoIterator<Item = &'a ChannelId>,
{
    let set = ids.into_iter().flatten().collect::<HashSet<_>>();
    set.into_iter().cloned().collect()
}

fn pre_order<T, F, I>(root: T, mut children: F) -> impl Iterator<Item = T>
where
    F: FnMut(T) -> Option<I>,
    I: Iterator<Item = T>,
    T: Clone + Copy,
{
    let mut stack = Vec::with_capacity(16);
    stack.extend(children(root));
    iter::once(root).chain(iter::from_fn(move || loop {
        let current_iter = stack.last_mut()?;
        match current_iter.next() {
            Some(i) => {
                stack.extend(children(i));
                return Some(i);
            }
            None => {
                stack.pop();
            }
        }
    }))
}

#[cfg(test)]
mod tests {
    #[test]
    fn pre_order() {
        let node_children = vec![
            vec![1, 2, 3],
            vec![4, 5],
            vec![6, 7],
            vec![],
            vec![8, 9],
            vec![],
            vec![10, 11],
            vec![],
            vec![],
            vec![],
            vec![],
        ];
        let expected = vec![0, 1, 4, 8, 9, 5, 2, 6, 10, 11, 7, 3];

        let result = super::pre_order(0, |i| node_children.get(i).map(|c| c.iter().copied()))
            .collect::<Vec<_>>();

        assert_eq!(result, expected);
    }
}
