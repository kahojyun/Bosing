use crate::{
    quant::{ChannelId, Time},
    schedule::{Arrange as _, ElementRef, ElementVariant, Measure as _, TimeRange},
    util::{pre_order_iter, IterVariant},
};

pub(crate) fn arrange_to_plot(root: &ElementRef) -> impl Iterator<Item = PlotItem> + use<'_> {
    let time_range = TimeRange {
        start: Time::ZERO,
        span: root.measure(),
    };
    arrange_tree(root, time_range).flat_map(|x| {
        x.item.channels().iter().map(move |c| PlotItem {
            channel: c.clone(),
            time_range: x.time_range,
            depth: x.depth,
            kind: ItemKind::from_variant(&x.item.variant),
        })
    })
}

#[derive(Debug, Clone, Copy)]
struct ArrangedItem<'a> {
    item: &'a ElementRef,
    time_range: TimeRange,
    depth: usize,
}

fn arrange_tree(root: &ElementRef, time_range: TimeRange) -> impl Iterator<Item = ArrangedItem> {
    pre_order_iter(
        ArrangedItem {
            item: root,
            time_range,
            depth: 0,
        },
        arrange_children,
    )
}

fn arrange_children(
    ArrangedItem {
        item,
        time_range,
        depth,
    }: ArrangedItem,
) -> Option<impl Iterator<Item = ArrangedItem>> {
    let time_range = item.inner_time_range(time_range);
    match &item.variant {
        ElementVariant::Repeat(r) => Some(IterVariant::Repeat(r.arrange(time_range))),
        ElementVariant::Stack(s) => Some(IterVariant::Stack(s.arrange(time_range))),
        ElementVariant::Absolute(a) => Some(IterVariant::Absolute(a.arrange(time_range))),
        ElementVariant::Grid(g) => Some(IterVariant::Grid(g.arrange(time_range))),
        _ => None,
    }
    .map(move |x| {
        x.map(move |x| ArrangedItem {
            item: x.item,
            time_range: x.time_range,
            depth: depth + 1,
        })
    })
}

#[derive(Debug)]
pub(crate) struct PlotItem {
    channel: ChannelId,
    time_range: TimeRange,
    depth: usize,
    kind: ItemKind,
}

#[derive(Debug)]
pub(crate) enum ItemKind {
    Play,
    ShiftPhase,
    SetPhase,
    ShiftFreq,
    SetFreq,
    SwapPhase,
    Barrier,
    Repeat,
    Stack,
    Absolute,
    Grid,
}

impl ItemKind {
    fn from_variant(variant: &ElementVariant) -> Self {
        match variant {
            ElementVariant::Play(_) => Self::Play,
            ElementVariant::ShiftPhase(_) => Self::ShiftPhase,
            ElementVariant::SetPhase(_) => Self::SetPhase,
            ElementVariant::ShiftFreq(_) => Self::ShiftFreq,
            ElementVariant::SetFreq(_) => Self::SetFreq,
            ElementVariant::SwapPhase(_) => Self::SwapPhase,
            ElementVariant::Barrier(_) => Self::Barrier,
            ElementVariant::Repeat(_) => Self::Repeat,
            ElementVariant::Stack(_) => Self::Stack,
            ElementVariant::Absolute(_) => Self::Absolute,
            ElementVariant::Grid(_) => Self::Grid,
        }
    }
}
