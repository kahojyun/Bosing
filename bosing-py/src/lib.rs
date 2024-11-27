use pyo3::prelude::*;

#[pymodule]
pub mod _bosing {
    use pyo3::prelude::*;

    #[must_use]
    #[pyfunction]
    pub const fn add(a: usize, b: usize) -> usize {
        a + b
    }
}
