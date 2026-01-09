use std::sync::Arc;

use bosing::schedule;
use pyo3::{exceptions::PyValueError, prelude::*, pybacked::PyBackedStr};

use crate::{push_repr, types::Time};

use super::{Arg, Element, ElementSubclass, Label, Rich as _};

/// A stack layout element.
///
/// Each child element occupies some channels and has a duration. Stack layout
/// will put children as close as possible without changing the order of
/// children with common channels. Two layout orders are available:
/// :attr:`Direction.Backward` and :attr:`Direction.Forward`. The default order
/// is :attr:`Direction.Backward`.
///
/// Args:
///     *children (Element): Child elements.
///     direction (str | Direction): Layout order. Defaults to ``'backward'``.
#[pyclass(module="bosing._bosing",extends=Element, get_all, frozen)]
#[derive(Debug)]
pub struct Stack {
    children: Vec<Py<Element>>,
}

impl ElementSubclass for Stack {
    type Variant = schedule::Stack;

    fn repr(slf: &Bound<'_, Self>) -> Vec<Arg> {
        let py = slf.py();
        let mut res: Vec<_> = slf
            .get()
            .children
            .iter()
            .map(|x| Arg::positional(x, py))
            .collect();
        push_repr!(
            res,
            py,
            "direction",
            Self::direction(slf),
            Direction::Backward
        );
        res
    }
}

#[pymethods]
impl Stack {
    #[new]
    #[pyo3(signature = (
        *children,
        direction=None,
        margin=None,
        alignment=None,
        phantom=false,
        duration=None,
        max_duration=Time::INFINITY,
        min_duration=Time::ZERO,
        label=None,
    ))]
    #[expect(clippy::too_many_arguments)]
    fn new(
        children: Vec<Py<Element>>,
        direction: Option<&Bound<'_, PyAny>>,
        margin: Option<&Bound<'_, PyAny>>,
        alignment: Option<&Bound<'_, PyAny>>,
        phantom: bool,
        duration: Option<Time>,
        max_duration: Time,
        min_duration: Time,
        label: Option<Label>,
    ) -> PyResult<(Self, Element)> {
        let rust_children = children.iter().map(|x| x.get().0.clone()).collect();
        let variant = schedule::Stack::new().with_children(rust_children);
        let variant = if let Some(obj) = direction {
            variant.with_direction(extract_direction(obj)?.into())
        } else {
            variant
        };
        Ok((
            Self { children },
            Self::build_element(
                variant,
                margin,
                alignment,
                phantom,
                duration,
                max_duration,
                min_duration,
                label,
            )?,
        ))
    }

    /// Create a new stack layout with different children.
    ///
    /// Using this method may be more readable than specifying children in the
    /// constructor.
    ///
    /// .. code-block:: python
    ///
    ///     stack = Stack(direction='forward').with_children(
    ///         element1,
    ///         element2,
    ///     )
    ///
    /// Args:
    ///     *children (Element): New child elements.
    ///
    /// Returns:
    ///     Stack: New stack layout.
    #[pyo3(signature=(*children))]
    fn with_children(slf: &Bound<'_, Self>, children: Vec<Py<Element>>) -> PyResult<Py<Self>> {
        let py = slf.py();
        let rust_children = children.iter().map(|x| x.get().0.clone()).collect();
        let rust_base = &slf.cast::<Element>()?.get().0;
        let common = rust_base.common.clone();
        let variant = Self::variant(slf).clone().with_children(rust_children);
        Py::new(
            py,
            (
                Self { children },
                Element(Arc::new(schedule::Element::new(common, variant))),
            ),
        )
    }

    #[getter]
    fn direction(slf: &Bound<'_, Self>) -> Direction {
        Self::variant(slf).direction().into()
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        Self::to_repr(slf)
    }

    fn __rich_repr__(slf: &Bound<'_, Self>) -> Vec<Arg> {
        Self::to_rich_repr(slf)
    }
}

/// Layout order in a stack layout.
///
/// A stack layout has two possible children processing orders:
///
/// - :attr:`Direction.Backward`:
///     Process children in reverse order and schedule them as late as possible.
///     This is the default order.
///
/// - :attr:`Direction.Forward`:
///     Process children in original order and schedule them as early as
///     possible.
#[pyclass(module = "bosing", frozen, eq)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Backward,
    Forward,
}

impl From<schedule::Direction> for Direction {
    fn from(direction: schedule::Direction) -> Self {
        match direction {
            schedule::Direction::Backward => Self::Backward,
            schedule::Direction::Forward => Self::Forward,
        }
    }
}

impl From<Direction> for schedule::Direction {
    fn from(direction: Direction) -> Self {
        match direction {
            Direction::Backward => Self::Backward,
            Direction::Forward => Self::Forward,
        }
    }
}

#[pymethods]
impl Direction {
    /// Convert the value to Direction.
    ///
    /// The value can be:
    ///
    /// - :class:`Direction`
    /// - str: 'backward' or 'forward'
    ///
    /// Args:
    ///     obj (str | Direction): Value to convert.
    ///
    /// Returns:
    ///     Direction: Converted value.
    ///
    /// Raises:
    ///     ValueError: If the value cannot be converted.
    #[staticmethod]
    fn convert(obj: &Bound<'_, PyAny>) -> PyResult<Py<Self>> {
        if let Ok(slf) = obj.extract() {
            return Ok(slf);
        }
        if let Ok(s) = obj.extract::<PyBackedStr>() {
            let direction = match &*s {
                "backward" => Some(Self::Backward),
                "forward" => Some(Self::Forward),
                _ => None,
            };
            if let Some(direction) = direction {
                return Py::new(obj.py(), direction);
            }
        }
        let msg = concat!(
            "Failed to convert the value to Direction. ",
            "Must be Direction or one of 'backward', 'forward'"
        );
        Err(PyValueError::new_err(msg))
    }
}

fn extract_direction(obj: &Bound<'_, PyAny>) -> PyResult<Direction> {
    Direction::convert(obj).and_then(|x| x.extract(obj.py()).map_err(Into::into))
}
