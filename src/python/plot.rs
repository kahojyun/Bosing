use itertools::Itertools as _;
use pyo3::{prelude::*, sync::GILOnceCell, types::PyList};

use crate::{
    quant::{ChannelId, Label, Time},
    schedule::{Arrange as _, Arranged, ElementRef, ElementVariant, Measure, TimeRange},
    util::{pre_order_iter, IterVariant},
};

const BOSING_PLOT_MODULE: &str = "bosing._plot";
const BOSING_PLOT_PLOT: &str = "plot";

#[pyclass(module = "bosing._bosing", frozen, eq, hash)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

pub(super) fn plot_element(
    py: Python<'_>,
    root: ElementRef,
    ax: Option<PyObject>,
    channels: Option<Vec<ChannelId>>,
    max_depth: usize,
    show_label: bool,
) -> PyResult<PyObject> {
    let channels = match channels {
        Some(channels) => PyList::new_bound(py, channels),
        None => PyList::new_bound(py, root.channels()),
    };
    let plot_items = Box::new(arrange_to_plot(root));
    let blocks = PlotIter { inner: plot_items };
    call_plot(py, ax, blocks, channels, max_depth, show_label)
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

#[pyclass(module = "bosing._bosing", frozen, get_all)]
#[derive(Debug)]
pub(super) struct PlotArgs {
    ax: Option<PyObject>,
    blocks: Py<PlotIter>,
    channels: Py<PyList>,
    max_depth: usize,
    show_label: bool,
}

#[pyclass(module = "bosing._bosing")]
struct PlotIter {
    inner: Box<dyn Iterator<Item = PlotItem> + Send>,
}

#[pymethods]
impl PlotIter {
    fn __iter__(slf: Bound<'_, Self>) -> Bound<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyObject> {
        slf.inner.next().map(|x| x.into_py(slf.py()))
    }
}

#[pyclass(module = "bosing._bosing", frozen, get_all)]
#[derive(Debug)]
pub(super) struct PlotItem {
    channels: Vec<ChannelId>,
    start: Time,
    span: Time,
    depth: usize,
    kind: ItemKind,
    label: Option<Label>,
}

fn call_plot(
    py: Python<'_>,
    ax: Option<PyObject>,
    blocks: PlotIter,
    channels: Bound<'_, PyList>,
    max_depth: usize,
    show_label: bool,
) -> PyResult<PyObject> {
    static PLOT: GILOnceCell<PyObject> = GILOnceCell::new();
    let plot = PLOT.get_or_try_init(py, || {
        py.import_bound(BOSING_PLOT_MODULE)?
            .getattr(BOSING_PLOT_PLOT)
            .map(Into::into)
    })?;
    let args = PlotArgs {
        ax,
        blocks: Py::new(py, blocks)?,
        channels: channels.unbind(),
        max_depth,
        show_label,
    };
    plot.call1(py, (args,))
}

fn arrange_to_plot(root: ElementRef) -> impl Iterator<Item = PlotItem> {
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
            let label = item.common.label().cloned();
            PlotItem {
                channels,
                start,
                span,
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
