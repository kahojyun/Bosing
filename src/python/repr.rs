use itertools::Itertools as _;
use pyo3::{
    prelude::*,
    types::{DerefToPyAny, PyString},
};

#[derive(Debug)]
pub(crate) enum Arg {
    Positional(PyObject),
    Keyword(Py<PyString>, PyObject),
    KeyWithDefault(Py<PyString>, PyObject, PyObject),
}

impl Arg {
    pub(crate) fn positional<T: ToPyObject>(value: T, py: Python<'_>) -> Self {
        Self::Positional(value.to_object(py))
    }

    pub(crate) fn keyword<T: ToPyObject>(key: Py<PyString>, value: T, py: Python<'_>) -> Self {
        Self::Keyword(key, value.to_object(py))
    }

    pub(crate) fn key_with_default<T: ToPyObject>(
        key: Py<PyString>,
        value: T,
        default: T,
        py: Python<'_>,
    ) -> Self {
        Self::KeyWithDefault(key, value.to_object(py), default.to_object(py))
    }

    pub(crate) fn fmt(&self, py: Python<'_>) -> PyResult<Option<String>> {
        let result = match self {
            Arg::Positional(v) => Some(v.bind(py).repr()?.to_string()),
            Arg::Keyword(n, v) => Some(format!("{}={}", n, v.bind(py).repr()?)),
            Arg::KeyWithDefault(n, v, d) => {
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

impl IntoPy<PyObject> for Arg {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self {
            Arg::Positional(v) => (v,).into_py(py),
            Arg::Keyword(n, v) => (n, v).into_py(py),
            Arg::KeyWithDefault(n, v, d) => (n, v, d).into_py(py),
        }
    }
}

pub(crate) trait RichRepr: Sized + DerefToPyAny {
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
