use bosing::{
    quant,
    schedule::{Arrange as _, Arranged, ElementRef, ElementVariant, Measure, TimeRange},
    util::{IterVariant, pre_order_iter},
};
use itertools::Itertools as _;
use pyo3::{prelude::*, sync::PyOnceLock, types::PyList};

use crate::types::{ChannelId, Label, Time};

const BOSING_PLOT_MODULE: &str = "bosing._plot";
const BOSING_PLOT_PLOT: &str = "plot";

#[pyclass(module = "bosing._bosing", frozen, eq, hash)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemKind {
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

pub fn element(
    py: Python<'_>,
    root: ElementRef,
    ax: Option<Py<PyAny>>,
    channels: Option<Vec<ChannelId>>,
    max_depth: usize,
    show_label: bool,
) -> PyResult<Py<PyAny>> {
    let channels = channels.map_or_else(
        || PyList::new(py, root.channels().iter().cloned().map(ChannelId::from)),
        |channels| PyList::new(py, channels),
    )?;
    let plot_items = Box::new(arrange_to_plot(root));
    let blocks = PlotIter { inner: plot_items };
    call_plot(py, ax, blocks, channels, max_depth, show_label)
}

impl ItemKind {
    const fn from_variant(variant: &ElementVariant) -> Self {
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

#[pyclass(module = "bosing._bosing", name = "PlotArgs", frozen, get_all)]
#[derive(Debug)]
pub struct Args {
    ax: Option<Py<PyAny>>,
    blocks: Py<PlotIter>,
    channels: Py<PyList>,
    max_depth: usize,
    show_label: bool,
}

#[pyclass(module = "bosing._bosing")]
struct PlotIter {
    inner: Box<dyn Iterator<Item = Item> + Send + Sync>,
}

#[pymethods]
impl PlotIter {
    const fn __iter__(slf: Bound<'_, Self>) -> Bound<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<Item> {
        slf.inner.next()
    }
}

#[pyclass(module = "bosing._bosing", name = "PlotItem", frozen, get_all)]
#[derive(Debug)]
pub struct Item {
    channels: Vec<ChannelId>,
    start: Time,
    span: Time,
    depth: usize,
    kind: ItemKind,
    label: Option<Label>,
}

fn call_plot(
    py: Python<'_>,
    ax: Option<Py<PyAny>>,
    blocks: PlotIter,
    channels: Bound<'_, PyList>,
    max_depth: usize,
    show_label: bool,
) -> PyResult<Py<PyAny>> {
    static PLOT: PyOnceLock<Py<PyAny>> = PyOnceLock::new();
    let plot = PLOT.get_or_try_init(py, || {
        py.import(BOSING_PLOT_MODULE)?
            .getattr(BOSING_PLOT_PLOT)
            .map(Into::into)
    })?;
    let args = Args {
        ax,
        blocks: Py::new(py, blocks)?,
        channels: channels.unbind(),
        max_depth,
        show_label,
    };
    plot.call1(py, (args,))
}

fn arrange_to_plot(root: ElementRef) -> impl Iterator<Item = Item> {
    let time_range = TimeRange {
        start: quant::Time::ZERO,
        span: root.measure(),
    };
    arrange_tree(root, time_range).map(
        |ArrangedItem {
             item,
             time_range: TimeRange { start, span },
             depth,
         }| {
            let kind = ItemKind::from_variant(&item.variant);
            let channels = item.channels().iter().cloned().map(Into::into).collect();
            let label = item.common.label().cloned().map(Into::into);
            Item {
                channels,
                start: start.into(),
                span: span.into(),
                depth,
                kind,
                label,
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
