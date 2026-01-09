use bosing::shape;
use pyo3::{exceptions::PyTypeError, prelude::*};

/// Base class for shapes.
///
/// Shapes are used to define the envelope of a pulse. Internally, the shape is
/// represented as a function :math:`f(t)` defined on the interval :math:`t \in
/// [-0.5, 0.5]`. The shape should be normalized such that :math:`f(\pm 0.5) = 0`
/// and :math:`f(0) = 1`.
///
/// Following shapes are supported:
///
/// - :class:`Hann`: Hann window.
/// - :class:`Interp`: Interpolated shape.
#[pyclass(module = "bosing", subclass, frozen)]
#[derive(Debug, Clone)]
pub struct Shape;

impl Shape {
    pub(super) fn get_rust_shape(slf: &Bound<'_, Self>) -> PyResult<shape::Shape> {
        if slf.cast::<Hann>().is_ok() {
            return Ok(shape::Shape::new_hann());
        }
        if let Ok(interp) = slf.cast::<Interp>() {
            let interp = interp.get();
            return Ok(shape::Shape::new_interp(
                interp.knots.clone(),
                interp.controls.clone(),
                interp.degree,
            )?);
        }
        Err(PyTypeError::new_err("Invalid shape type."))
    }
}

/// A Hann shape.
#[pyclass(module="bosing._bosing",extends=Shape, frozen)]
#[derive(Debug, Clone)]
pub struct Hann;

#[pymethods]
impl Hann {
    #[new]
    const fn new() -> (Self, Shape) {
        (Self, Shape)
    }
}

/// An interpolated shape.
///
/// The interpolated shape use a B-spline. :func:`scipy.interpolate.make_interp_spline`
/// can be used to calculate the parameters.
///
/// .. caution::
///
///     It's user's responsibility to ensure the b-spline parameters are valid and
///     the shape is normalized such that :math:`f(\pm 0.5) = 0` and :math:`f(0) = 1`.
///
/// Args:
///     knots (Sequence[float]): Knots of the B-spline.
///     controls (Sequence[float]): Control points of the B-spline.
///     degree (int): Degree of the B-spline.
///
/// Example:
///     .. code-block:: python
///
///         import numpy as np
///         from scipy.interpolate import make_interp_spline
///         from bosing import Interp
///         x = np.linspace(0, np.pi, 10)
///         y = np.sin(x)
///         x = (x - x[0]) / (x[-1] - x[0]) - 0.5 # Normalize x to [-0.5, 0.5]
///         spline = make_interp_spline(x, y, k=3)
///         interp = Interp(spline.t, spline.c, spline.k)
#[pyclass(module="bosing._bosing",extends=Shape, get_all, frozen)]
#[derive(Debug, Clone)]
pub struct Interp {
    knots: Vec<f64>,
    controls: Vec<f64>,
    degree: usize,
}

#[pymethods]
impl Interp {
    #[new]
    const fn new(knots: Vec<f64>, controls: Vec<f64>, degree: usize) -> (Self, Shape) {
        (
            Self {
                knots,
                controls,
                degree,
            },
            Shape,
        )
    }
}
