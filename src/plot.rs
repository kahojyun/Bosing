use itertools::Itertools as _;

use crate::{
    quant::Time,
    schedule::{Arrange as _, Arranged, ElementRef, ElementVariant, Measure as _, TimeRange},
    util::{pre_order_iter, IterVariant},
    ItemKind, PlotItem,
};

pub(crate) fn arrange_to_plot(root: ElementRef) -> impl Iterator<Item = PlotItem> {
    let time_range = TimeRange {
        start: Time::ZERO,
        span: root.measure(),
    };
    arrange_tree(root, time_range).map(
        |ArrangedItem {
             item,
             time_range: TimeRange { start, span },
             depth,
         }| {
            let kind = ItemKind::from_variant(&item.variant);
            let channels = item.channels().to_vec();
            PlotItem {
                channels,
                start,
                span,
                depth,
                kind,
            }
        },
    )
}

#[derive(Debug, Clone)]
struct ArrangedItem {
    item: ElementRef,
    time_range: TimeRange,
    depth: usize,
}

fn arrange_tree(root: ElementRef, time_range: TimeRange) -> impl Iterator<Item = ArrangedItem> {
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
        x.map(move |Arranged { item, time_range }| ArrangedItem {
            item: item.clone(),
            time_range,
            depth: depth + 1,
        })
        .collect_vec()
        .into_iter()
    })
}
