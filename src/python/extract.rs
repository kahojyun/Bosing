use ndarray::ArrayView;
use numpy::{prelude::*, Ix1, Ix2, PyArray};
use pyo3::{exceptions::PyTypeError, intern, prelude::*, sync::GILOnceCell, types::PyDict};

macro_rules! define_wrapper {
    ($name:ident, $t:ty, $d:ty, $err_msg:expr, $check:expr) => {
        /// Readonly wrapper around a numpy array.
        #[derive(Debug)]
        pub(crate) struct $name(Py<PyArray<$t, $d>>);

        impl $name {
            pub(crate) fn clone_ref(&self, py: Python<'_>) -> Self {
                Self(self.0.clone_ref(py))
            }

            pub(crate) fn as_array<'a, 'py: 'a>(
                &'a self,
                py: Python<'py>,
            ) -> ArrayView<'a, $t, $d> {
                let arr = self.0.bind(py);
                // SAFETY: self.0 is private and no methods provide mutable access to it.
                unsafe { arr.as_array() }
            }
        }

        impl<'py> FromPyObject<'py> for $name {
            fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
                let err_msg = $err_msg;
                let arr =
                    np_as_readonly_array(ob).map_err(|e| PyTypeError::new_err((err_msg, e)))?;
                let arr = arr
                    .downcast_into::<PyArray<$t, $d>>()
                    .map_err(|_| PyTypeError::new_err(err_msg))?;
                if !$check(&arr) {
                    return Err(PyTypeError::new_err(err_msg));
                }

                Ok(Self(arr.unbind()))
            }
        }

        impl ToPyObject for $name {
            fn to_object(&self, py: Python<'_>) -> PyObject {
                self.0.to_object(py)
            }
        }
    };
}

define_wrapper!(
    IqMatrix,
    f64,
    Ix2,
    "IQ matrix should be convertible to a 2x2 f64 numpy array.",
    |arr: &Bound<'_, PyArray<f64, Ix2>>| arr.dims() == [2, 2]
);

define_wrapper!(
    OffsetArray,
    f64,
    Ix1,
    "offset should be convertible to a 1d f64 numpy array.",
    |_arr: &Bound<'_, PyArray<f64, Ix1>>| true
);

define_wrapper!(
    IirArray,
    f64,
    Ix2,
    "iir should be convertible to a Nx6 f64 numpy array. Usually this is generated by scipy.signal routines.",
    |arr: &Bound<'_, PyArray<f64, Ix2>>| arr.dims()[1] == 6
);

define_wrapper!(
    FirArray,
    f64,
    Ix1,
    "fir should be convertible to a 1d f64 numpy array.",
    |_arr: &Bound<'_, PyArray<f64, Ix1>>| true
);

/// Convert a Python object to a read-only numpy array.
fn np_as_readonly_array<'py>(ob: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
    static AS_ARRAY: GILOnceCell<PyObject> = GILOnceCell::new();
    let py = ob.py();
    let as_array = AS_ARRAY
        .get_or_try_init(py, || -> PyResult<PyObject> {
            Ok(py.import_bound("numpy")?.getattr("asarray")?.into())
        })?
        .bind(py);
    let arr = as_array.call1((ob, <f64 as numpy::Element>::get_dtype_bound(py)))?;
    let kwargs = PyDict::new_bound(py);
    kwargs.set_item(intern!(py, "write"), false)?;
    arr.getattr(intern!(py, "setflags"))?
        .call((), Some(&kwargs))?;
    Ok(arr)
}
