use std::{hash::Hash, sync::Arc};

use anyhow::{anyhow, Result};
use bspline::BSpline;
use cached::proc_macro::cached;
use enum_dispatch::enum_dispatch;
use ordered_float::NotNan;

/// A shape that can be used to modulate the amplitude of a signal.
///
/// The shape is defined in the range \[-0.5, 0.5\].
///
/// Internally, shape instances are cached such that we can compare and hash
/// by instance address.
#[derive(Debug, Clone)]
pub struct Shape(Arc<ShapeVariant>);

impl Shape {
    pub fn new_hann() -> Self {
        Self(get_shape_instance(ShapeKey::Hann))
    }

    pub fn new_interp(knots: Vec<f64>, controls: Vec<f64>, degree: usize) -> Result<Self> {
        let knots = knots
            .into_iter()
            .map(NotNan::new)
            .collect::<Result<_, _>>()
            .map_err(|_| anyhow!("Nan in knots"))?;
        let controls = controls
            .into_iter()
            .map(NotNan::new)
            .collect::<Result<_, _>>()
            .map_err(|_| anyhow!("Nan in controls"))?;
        let key = ShapeKey::Interp(knots, controls, degree);
        Ok(Self(get_shape_instance(key)))
    }

    pub fn sample_array(&self, x0: f64, dx: f64, array: &mut [f64]) {
        self.0.sample_array(x0, dx, array);
    }
}

impl Hash for Shape {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}

impl PartialEq for Shape {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for Shape {}

type HashableArray = Vec<NotNan<f64>>;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
enum ShapeKey {
    Hann,
    Interp(HashableArray, HashableArray, usize),
}

#[cached(size = 128)]
fn get_shape_instance(a: ShapeKey) -> Arc<ShapeVariant> {
    let variant = match a {
        ShapeKey::Hann => Hann.into(),
        ShapeKey::Interp(t, c, k) => {
            let t = t.into_iter().map(|v| v.into()).collect();
            let c = c.into_iter().map(|v| v.into()).collect();
            Interp::new(t, c, k).into()
        }
    };
    Arc::new(variant)
}

#[enum_dispatch(ShapeTrait)]
#[derive(Debug, Clone)]
enum ShapeVariant {
    Hann,
    Interp,
}

#[enum_dispatch]
trait ShapeTrait {
    /// Sample the shape at a given position x in the range \[-0.5, 0.5\].
    fn sample(&self, x: f64) -> f64;
    fn sample_array(&self, x0: f64, dx: f64, array: &mut [f64]) {
        for (i, y) in array.iter_mut().enumerate() {
            *y = self.sample(x0 + i as f64 * dx);
        }
    }
}

#[derive(Debug, Clone)]
struct Hann;

impl ShapeTrait for Hann {
    fn sample(&self, x: f64) -> f64 {
        0.5 * (1.0 + (2.0 * std::f64::consts::PI * x).cos())
    }
}

#[derive(Debug, Clone)]
struct Interp(BSpline<f64, f64>);

impl Interp {
    fn new(knots: Vec<f64>, controls: Vec<f64>, degree: usize) -> Self {
        Self(BSpline::new(degree, controls, knots))
    }
}

impl ShapeTrait for Interp {
    fn sample(&self, x: f64) -> f64 {
        let (min, max) = self.0.knot_domain();
        if x < min || x > max {
            return 0.0;
        }
        self.0.point(x)
    }
}

#[cfg(test)]
mod tests {
    use float_cmp::assert_approx_eq;

    use super::*;

    #[test]
    fn test_hann() {
        let hann = Hann;
        assert_approx_eq!(f64, hann.sample(-0.5), 0.0);
        assert_approx_eq!(f64, hann.sample(-0.25), 0.5);
        assert_approx_eq!(f64, hann.sample(0.0), 1.0);
        assert_approx_eq!(f64, hann.sample(0.25), 0.5);
        assert_approx_eq!(f64, hann.sample(0.5), 0.0);
    }

    #[test]
    fn test_interp() {
        // Generated with the following Python code:
        // ```
        // import numpy as np
        // from scipy.interpolate import make_interp_spline
        // x = np.linspace(-0.5, 0.5, 7)
        // y = np.cos(x * np.pi)
        // interp = make_interp_spline(x, y, k=3)
        // knots = interp.t
        // controls = interp.c
        // print("let knots = vec![{}];".format(", ".join(map(str, knots))))
        // print("let controls = vec![{}];".format(", ".join(map(str, controls))))
        // test_x = np.linspace(-0.5, 0.5, 10)
        // test_y = interp(test_x)
        // print("let test_x = vec![{}];".format(", ".join(map(str, test_x))))
        // print("let test_y = vec![{}];".format(", ".join(map(str, test_y))))
        // ```
        let knots = vec![
            -0.5,
            -0.5,
            -0.5,
            -0.5,
            -0.16666666666666669,
            0.0,
            0.16666666666666663,
            0.5,
            0.5,
            0.5,
            0.5,
        ];
        let controls = vec![
            6.123233995736766e-17,
            0.35338865119588236,
            0.8602099957160162,
            1.0465966680946615,
            0.8602099957160163,
            0.35338865119588264,
            6.123233995736766e-17,
        ];
        let test_x = vec![
            -0.5,
            -0.3888888888888889,
            -0.2777777777777778,
            -0.16666666666666669,
            -0.05555555555555558,
            0.05555555555555558,
            0.16666666666666663,
            0.2777777777777777,
            0.38888888888888884,
            0.5,
        ];
        let test_y = vec![
            6.123233995736766e-17,
            0.34275209271817986,
            0.6423618410356466,
            0.8660254037844386,
            0.9846831627857952,
            0.9846831627857954,
            0.8660254037844388,
            0.6423618410356471,
            0.3427520927181801,
            6.123233995736766e-17,
        ];
        let interp = Interp::new(knots, controls, 3);
        for (&x, &y) in test_x.iter().zip(test_y.iter()) {
            assert_approx_eq!(f64, interp.sample(x), y);
        }
    }

    #[test]
    fn test_shape_eq() {
        let h1 = Shape::new_hann();
        let h2 = Shape::new_hann();
        assert_eq!(h1, h2);
        let knots = vec![
            -0.5,
            -0.5,
            -0.5,
            -0.5,
            -0.16666666666666669,
            0.0,
            0.16666666666666663,
            0.5,
            0.5,
            0.5,
            0.5,
        ];
        let controls = vec![
            6.123233995736766e-17,
            0.35338865119588236,
            0.8602099957160162,
            1.0465966680946615,
            0.8602099957160163,
            0.35338865119588264,
            6.123233995736766e-17,
        ];
        let i1 = Shape::new_interp(knots.clone(), controls.clone(), 3).unwrap();
        let i2 = Shape::new_interp(knots.clone(), controls.clone(), 3).unwrap();
        assert_eq!(i1, i2);
        assert_ne!(h1, i1);
    }
}
