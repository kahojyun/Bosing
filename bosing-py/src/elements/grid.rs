use std::sync::Arc;

use bosing::schedule;
use pyo3::{Borrowed, exceptions::PyValueError, prelude::*, pybacked::PyBackedStr};

use crate::{push_repr, types::Time};

use super::{Arg, Element, ElementSubclass, Label, Rich};

/// A grid layout element.
///
/// A grid layout has multiple columns and each child element occupies some
/// columns. The width of each column can be specified by :class:`GridLength`,
/// which can be:
///
/// - Fixed length in seconds.
/// - Auto length:
///     The width is determined by the child element.
///
/// - Star length:
///     The width id determined by remaining duration. For example, if there
///     are two columns with 1* and 2* and the remaining duration is 300 ns,
///     the width of the columns will be 100 ns and 200 ns.
///
/// Columns length can be specified with a simplified syntax:
///
/// - 'auto': Auto length.
/// - 'x*': x stars.
/// - 'x': Fixed length in seconds.
/// - '*': 1 star.
///
/// If no columns are provided, the grid layout will have one column with '*'.
///
/// Children can be provided as:
///
/// - GridEntry
/// - Element: The column index is 0 and the span is 1.
/// - tuple[Element, int]: Element and column. The span is 1.
/// - tuple[Element, int, int]: Element, column, and span.
///
/// Args:
///     *children (GridEntry | Element | tuple[Element, int] | tuple[Element, int, int]): Child elements.
///     columns (Iterable[GridLength | float | str]): Column lengths. Defaults to ['*'].
///
/// Example:
///     .. code-block:: python
///
///         grid = Grid(
///             GridEntry(element1, 0, 1),
///             (element2, 1),
///             (element3, 2, 2),
///             element4,
///             columns=['auto', '1*', '2'],
///         )
#[pyclass(module="bosing._bosing", extends=Element, frozen)]
#[derive(Debug)]
pub struct Grid {
    children: Vec<Entry>,
}

impl ElementSubclass for Grid {
    type Variant = schedule::Grid;

    fn repr(slf: &Bound<'_, Self>) -> Vec<Arg> {
        let py = slf.py();
        let mut res: Vec<_> = slf
            .get()
            .children
            .iter()
            .map(|x| Arg::positional(x, py))
            .collect();
        push_repr!(res, py, "columns", Self::columns(slf));
        res
    }
}

#[pymethods]
impl Grid {
    #[new]
    #[pyo3(signature = (
        *children,
        columns=vec![],
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
        columns: Vec<Py<PyAny>>,
        margin: Option<&Bound<'_, PyAny>>,
        alignment: Option<&Bound<'_, PyAny>>,
        phantom: bool,
        duration: Option<Time>,
        max_duration: Time,
        min_duration: Time,
        label: Option<Label>,
    ) -> PyResult<(Self, Element)> {
        let children: Vec<_> = children
            .into_iter()
            .map(|x| extract_grid_entry(&x.into_bound(py)))
            .collect::<PyResult<_>>()?;
        let columns: Vec<_> = columns
            .into_iter()
            .map(|x| extract_grid_length(&x.into_bound(py)).map(Into::into))
            .collect::<PyResult<_>>()?;
        let rust_children = children
            .iter()
            .map(|x| {
                let element = x.element.get().0.clone();
                schedule::GridEntry::new(element)
                    .with_column(x.column)
                    .with_span(x.span)
                    .expect("Should be checked in GridEntry::new")
            })
            .collect();
        let variant = schedule::Grid::new()
            .with_children(rust_children)
            .with_columns(columns);
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

    /// Create a new grid schedule with different children.
    ///
    /// Using this method may be more readable than specifying children in the
    /// constructor.
    ///
    /// .. code-block:: python
    ///
    ///     grid = Grid(columns=['auto', '*', 'auto']).with_children(
    ///         element1,
    ///         (element2, 2),
    ///         (element3, 0, 3),
    ///     )
    ///
    /// Args:
    ///     *children (GridEntry | Element | tuple[Element, int] | tuple[Element, int, int]): New child elements.
    ///
    /// Returns:
    ///     Grid: New grid schedule.
    #[pyo3(signature=(*children))]
    fn with_children(slf: &Bound<'_, Self>, children: Vec<Py<PyAny>>) -> PyResult<Py<Self>> {
        let py = slf.py();
        let children: Vec<_> = children
            .into_iter()
            .map(|x| extract_grid_entry(&x.into_bound(py)))
            .collect::<PyResult<_>>()?;
        let rust_children = children
            .iter()
            .map(|x| {
                let element = x.element.get().0.clone();
                schedule::GridEntry::new(element)
                    .with_column(x.column)
                    .with_span(x.span)
                    .expect("Should be checked in GridEntry::new")
            })
            .collect();
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
    fn columns(slf: &Bound<'_, Self>) -> Vec<Length> {
        Self::variant(slf)
            .columns()
            .iter()
            .copied()
            .map(Into::into)
            .collect()
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

/// Unit of grid length.
///
/// The unit can be:
///
/// - Seconds: Fixed length in seconds.
/// - Auto: Auto length.
/// - Star: Ratio of the remaining duration.
#[pyclass(module = "bosing", name = "GridLengthUnit", frozen, eq)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LengthUnit {
    Seconds,
    Auto,
    Star,
}

/// Length of a grid column.
///
/// :class:`GridLength` is used to specify the length of a grid column. The
/// length can be specified in seconds, as a fraction of the remaining duration,
/// or automatically.
#[pyclass(module = "bosing", name = "GridLength", frozen)]
#[derive(Debug, Clone)]
pub struct Length(schedule::GridLength);

impl Length {
    pub const STAR: Self = Self(schedule::GridLength::STAR);
}

#[pymethods]
impl Length {
    #[new]
    fn new(value: f64, unit: LengthUnit) -> PyResult<Self> {
        match unit {
            LengthUnit::Seconds => Self::fixed(value),
            LengthUnit::Auto => Ok(Self::auto()),
            LengthUnit::Star => Self::star(value),
        }
    }

    #[getter]
    const fn value(&self) -> f64 {
        match self.0 {
            schedule::GridLength::Star(v) | schedule::GridLength::Fixed(v) => v,
            schedule::GridLength::Auto => 0.0,
        }
    }

    #[getter]
    const fn unit(&self) -> LengthUnit {
        match self.0 {
            schedule::GridLength::Fixed(_) => LengthUnit::Seconds,
            schedule::GridLength::Star(_) => LengthUnit::Star,
            schedule::GridLength::Auto => LengthUnit::Auto,
        }
    }

    /// Create an automatic grid length.
    ///
    /// Returns:
    ///     GridLength: Automatic grid length.
    #[staticmethod]
    const fn auto() -> Self {
        Self(schedule::GridLength::auto())
    }

    /// Create a ratio based grid length.
    ///
    /// Args:
    ///     value (float): Ratio of the remaining duration.
    ///
    /// Returns:
    ///     GridLength: Ratio based grid length.
    #[staticmethod]
    fn star(value: f64) -> PyResult<Self> {
        schedule::GridLength::star(value)
            .map_err(|e| PyValueError::new_err(e.to_string()))
            .map(Self)
    }

    /// Create a fixed grid length.
    ///
    /// Args:
    ///     value (float): Fixed length in seconds.
    ///
    /// Returns:
    ///     GridLength: Fixed grid length.
    #[staticmethod]
    fn fixed(value: f64) -> PyResult<Self> {
        schedule::GridLength::fixed(value)
            .map_err(|e| PyValueError::new_err(e.to_string()))
            .map(Self)
    }

    /// Convert the value to GridLength.
    ///
    /// The value can be:
    ///
    /// - GridLength
    /// - float: Fixed length in seconds.
    /// - 'auto': Auto length.
    /// - 'x*': x stars.
    /// - 'x': Fixed length in seconds.
    /// - '*': 1 star.
    ///
    /// Args:
    ///     obj (GridLength | float | str): Value to convert.
    ///
    /// Returns:
    ///     GridLength: Converted value.
    ///
    /// Raises:
    ///     ValueError: If the value cannot be converted.
    #[staticmethod]
    fn convert(obj: &Bound<'_, PyAny>) -> PyResult<Py<Self>> {
        let py = obj.py();
        if let Ok(slf) = obj.extract::<Py<Self>>() {
            return Ok(slf);
        }
        if let Ok(v) = obj.extract::<f64>() {
            return Py::new(py, Self::fixed(v)?);
        }
        if let Ok(s) = obj.extract::<PyBackedStr>() {
            return Py::new(py, Self(s.parse()?));
        }
        Err(PyValueError::new_err(
            "Failed to convert the value to GridLength.",
        ))
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        Self::to_repr(slf)
    }

    fn __rich_repr__(slf: &Bound<'_, Self>) -> Vec<Arg> {
        Self::to_rich_repr(slf)
    }
}

impl Rich for Length {
    fn repr(slf: &Bound<'_, Self>) -> impl Iterator<Item = Arg> {
        let mut res = Vec::new();
        let py = slf.py();
        let slf = slf.get();
        push_repr!(res, py, slf.value());
        push_repr!(res, py, slf.unit());
        res.into_iter()
    }
}

fn extract_grid_length(obj: &Bound<'_, PyAny>) -> PyResult<Length> {
    Length::convert(obj).and_then(|x| x.extract(obj.py()).map_err(Into::into))
}

impl From<Length> for schedule::GridLength {
    fn from(value: Length) -> Self {
        value.0
    }
}

impl From<schedule::GridLength> for Length {
    fn from(value: schedule::GridLength) -> Self {
        Self(value)
    }
}

/// A child element in a grid layout.
///
/// Args:
///     element (Element): Child element.
///     column (int): Column index.
///     span (int): Column span.
#[pyclass(module = "bosing", name = "GridEntry", get_all, frozen)]
#[derive(Debug, IntoPyObjectRef)]
pub struct Entry {
    element: Py<Element>,
    column: usize,
    span: usize,
}

impl Entry {
    fn clone_ref(&self, py: Python<'_>) -> Self {
        Self {
            element: self.element.clone_ref(py),
            column: self.column,
            span: self.span,
        }
    }
}

#[pymethods]
impl Entry {
    #[new]
    #[pyo3(signature = (element, column=0, span=1))]
    fn new(element: Py<Element>, column: usize, span: usize) -> PyResult<Self> {
        if span == 0 {
            return Err(PyValueError::new_err("The span must be greater than 0."));
        }
        Ok(Self {
            element,
            column,
            span,
        })
    }

    /// Convert the value to GridEntry.
    ///
    /// The value can be:
    ///
    /// - GridEntry
    /// - Element
    /// - tuple[Element, int]: Element and column.
    /// - tuple[Element, int, int]: Element, column, and span.
    ///
    /// Args:
    ///     obj (GridEntry | Element | tuple[Element, int] | tuple[Element, int, int]): Value to convert.
    ///
    /// Returns:
    ///     GridEntry: Converted value.
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
            return Py::new(py, Self::new(element, 0, 1)?);
        }
        if let Ok((element, column)) = obj.extract() {
            return Py::new(py, Self::new(element, column, 1)?);
        }
        if let Ok((element, column, span)) = obj.extract() {
            return Py::new(py, Self::new(element, column, span)?);
        }
        Err(PyValueError::new_err(
            "Failed to convert the value to GridEntry.",
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
        push_repr!(res, py, &slf.element);
        push_repr!(res, py, "column", slf.column, 0);
        push_repr!(res, py, "span", slf.span, 0);
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

fn extract_grid_entry(obj: &Bound<'_, PyAny>) -> PyResult<Entry> {
    Entry::convert(obj).and_then(|x| x.extract(obj.py()))
}
