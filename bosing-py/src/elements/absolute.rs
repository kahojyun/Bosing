use std::sync::Arc;

use bosing::schedule;
use pyo3::{Borrowed, exceptions::PyValueError, prelude::*};

use crate::{push_repr, types::Time};

use super::{Arg, Element, ElementSubclass, Label, Rich};

/// An absolute layout element.
///
/// The child elements are arranged in absolute time. The time of each child
/// element is relative to the start of the absolute schedule. The duration of
/// the absolute schedule is the maximum end time of the child elements.
///
/// The `children` argument can be:
///
/// - AbsoluteEntry
/// - Element
/// - tuple[float, Element]: Time and element.
///
/// Args:
///     *children (AbsoluteEntry | Element | tuple[float, Element]): Child elements.
///
/// Example:
///     .. code-block:: python
///
///         absolute = Absolute(
///             element1,
///             (1.0, element2),
///             AbsoluteEntry(2.0, element3),
///         )
#[pyclass(module="bosing._bosing",extends=Element, frozen)]
#[derive(Debug)]
pub struct Absolute {
    children: Vec<Entry>,
}

impl ElementSubclass for Absolute {
    type Variant = schedule::Absolute;

    fn repr(slf: &Bound<'_, Self>) -> Vec<Arg> {
        let py = slf.py();
        slf.get()
            .children
            .iter()
            .map(|x| Arg::positional(x, py))
            .collect()
    }
}

#[pymethods]
impl Absolute {
    #[new]
    #[pyo3(signature = (
        *children,
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
        py: Python<'_>,
        children: Vec<Py<PyAny>>,
        margin: Option<&Bound<'_, PyAny>>,
        alignment: Option<&Bound<'_, PyAny>>,
        phantom: bool,
        duration: Option<Time>,
        max_duration: Time,
        min_duration: Time,
        label: Option<Label>,
    ) -> PyResult<(Self, Element)> {
        let children: Vec<Entry> = children
            .into_iter()
            .map(|x| extract_absolute_entry(&x.into_bound(py)))
            .collect::<PyResult<_>>()?;
        let rust_children = children
            .iter()
            .map(|x| {
                let element = x.element.get().0.clone();
                Ok(schedule::AbsoluteEntry::new(element).with_time(x.time.into())?)
            })
            .collect::<PyResult<_>>()?;
        let variant = schedule::Absolute::new().with_children(rust_children);
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

    /// Create a new absolute schedule with different children.
    ///
    /// Using this method may be more readable than specifying children in the
    /// constructor.
    ///
    /// .. code-block:: python
    ///
    ///     absolute = Absolute(duration=50e-6).with_children(
    ///         element1,
    ///         (100e-9, element2),
    ///     )
    ///
    /// Args:
    ///     *children (AbsoluteEntry | Element | tuple[float, Element]): New
    ///         child elements.
    ///
    /// Returns:
    ///     Absolute: New absolute schedule.
    #[pyo3(signature=(*children))]
    fn with_children(slf: &Bound<'_, Self>, children: Vec<Py<PyAny>>) -> PyResult<Py<Self>> {
        let py = slf.py();
        let children: Vec<_> = children
            .into_iter()
            .map(|x| extract_absolute_entry(&x.into_bound(py)))
            .collect::<PyResult<_>>()?;
        let rust_children = children
            .iter()
            .map(|x| {
                let element = x.element.get().0.clone();
                Ok(schedule::AbsoluteEntry::new(element).with_time(x.time.into())?)
            })
            .collect::<PyResult<_>>()?;
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
    fn children<'a>(slf: &'a Bound<'_, Self>) -> &'a Vec<Entry> {
        &slf.get().children
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        Self::to_repr(slf)
    }

    fn __rich_repr__(slf: &Bound<'_, Self>) -> Vec<Arg> {
        Self::to_rich_repr(slf)
    }
}

/// A child element with an absolute time in a absolute layout.
///
/// The time of each child element is relative to the start of the absolute
/// layout.
///
/// Args:
///     time (float): Time relative to the start of the parent element.
///     element (Element): Child element.
#[pyclass(module = "bosing", name = "AbsoluteEntry", get_all, frozen)]
#[derive(Debug, IntoPyObjectRef)]
pub struct Entry {
    time: Time,
    element: Py<Element>,
}

impl Entry {
    fn clone_ref(&self, py: Python<'_>) -> Self {
        Self {
            time: self.time,
            element: self.element.clone_ref(py),
        }
    }
}

#[pymethods]
impl Entry {
    #[new]
    fn new(time: Time, element: Py<Element>) -> PyResult<Self> {
        if !time.0.value().is_finite() {
            return Err(PyValueError::new_err("Time must be finite"));
        }
        Ok(Self { time, element })
    }

    /// Convert the value to AbsoluteEntry.
    ///
    /// the value can be:
    ///
    /// - AbsoluteEntry
    /// - Element
    /// - tuple[float, Element]: Time and element.
    ///
    /// Args:
    ///     obj (AbsoluteEntry | Element | tuple[float, Element]): Value to convert.
    ///
    /// Returns:
    ///     AbsoluteEntry: Converted value.
    ///
    /// Raises:
    ///     ValueError: If the value cannot be converted.
    #[staticmethod]
    fn convert(obj: &Bound<'_, PyAny>) -> PyResult<Py<Self>> {
        let py = obj.py();
        if let Ok(slf) = obj.extract() {
            return Ok(slf);
        }
        if let Ok(element) = obj.extract() {
            return Py::new(py, Self::new(Time::ZERO, element)?);
        }
        if let Ok((time, element)) = obj.extract() {
            return Py::new(py, Self::new(time, element)?);
        }
        Err(PyValueError::new_err(
            "Failed to convert the value to AbsoluteEntry",
        ))
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        Self::to_repr(slf)
    }

    fn __rich_repr__(slf: &Bound<'_, Self>) -> Vec<Arg> {
        Self::to_rich_repr(slf)
    }
}

impl Rich for Entry {
    fn repr(slf: &Bound<'_, Self>) -> impl Iterator<Item = Arg> {
        let mut res = Vec::new();
        let py = slf.py();
        let slf = slf.get();
        push_repr!(res, py, slf.time);
        push_repr!(res, py, &slf.element);
        res.into_iter()
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for Entry {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        let py = obj.py();
        let obj = obj.cast_exact::<Self>()?;
        let obj = obj.get();
        Ok(obj.clone_ref(py))
    }
}

fn extract_absolute_entry(obj: &Bound<'_, PyAny>) -> PyResult<Entry> {
    Entry::convert(obj).and_then(|x| x.extract(obj.py()))
}
