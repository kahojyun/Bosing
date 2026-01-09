use itertools::Itertools as _;
use pyo3::{
    IntoPyObjectExt,
    prelude::*,
    types::{DerefToPyAny, PyString},
};

// TODO: check if derive IntoPyObject works
#[derive(Debug, IntoPyObject)]
pub enum Arg {
    Positional(Py<PyAny>),
    Keyword(Py<PyString>, Py<PyAny>),
    KeyWithDefault(Py<PyString>, Py<PyAny>, Py<PyAny>),
}

fn into_pyobject<'py, T: IntoPyObjectExt<'py>>(value: T, py: Python<'py>) -> Py<PyAny> {
    value
        .into_py_any(py)
        .expect("failed to convert to PyObject")
}

impl Arg {
    pub(crate) fn positional<'py, T: IntoPyObjectExt<'py>>(value: T, py: Python<'py>) -> Self {
        Self::Positional(into_pyobject(value, py))
    }

    pub(crate) fn keyword<'py, T: IntoPyObjectExt<'py>>(
        key: Py<PyString>,
        value: T,
        py: Python<'py>,
    ) -> Self {
        Self::Keyword(key, into_pyobject(value, py))
    }

    pub(crate) fn key_with_default<'py, T: IntoPyObjectExt<'py>>(
        key: Py<PyString>,
        value: T,
        default: T,
        py: Python<'py>,
    ) -> Self {
        Self::KeyWithDefault(key, into_pyobject(value, py), into_pyobject(default, py))
    }

    pub(crate) fn fmt(&self, py: Python<'_>) -> PyResult<Option<String>> {
        let result = match self {
            Self::Positional(v) => Some(v.bind(py).repr()?.to_string()),
            Self::Keyword(n, v) => Some(format!("{}={}", n, v.bind(py).repr()?)),
            Self::KeyWithDefault(n, v, d) => {
                if matches!(v.bind(py).eq(d), Ok(true)) {
                    None
                } else {
                    Some(format!("{}={}", n, v.bind(py).repr()?))
                }
            }
        };
        Ok(result)
    }
}

pub trait Rich: Sized + DerefToPyAny {
    fn repr(slf: &Bound<'_, Self>) -> impl Iterator<Item = Arg>;

    fn to_rich_repr(slf: &Bound<'_, Self>) -> Vec<Arg> {
        Self::repr(slf).collect()
    }

    fn to_repr(slf: &Bound<'_, Self>) -> PyResult<String> {
        let py = slf.py();
        let cls_name = slf.get_type().qualname()?;
        Ok(format!(
            "{}({})",
            cls_name,
            Self::repr(slf)
                .map(|x| x.fmt(py))
                .flatten_ok()
                .collect::<PyResult<Vec<_>>>()?
                .join(", ")
        ))
    }
}
