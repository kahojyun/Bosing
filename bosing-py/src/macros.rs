#[macro_export]
macro_rules! push_repr {
    ($vec:expr, $py:expr, $value:expr) => {
        $vec.push($crate::repr::Arg::positional($value, $py));
    };
    ($vec:expr, $py:expr, $key:expr, $value:expr) => {
        $vec.push($crate::repr::Arg::keyword(
            pyo3::intern!($py, $key).clone().unbind(),
            $value,
            $py,
        ));
    };
    ($vec:expr, $py:expr, $key:expr, $value:expr, $default:expr) => {
        $vec.push($crate::repr::Arg::key_with_default(
            pyo3::intern!($py, $key).clone().unbind(),
            $value,
            $default,
            $py,
        ));
    };
}
