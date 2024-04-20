use bspline::BSpline;
use enum_dispatch::enum_dispatch;

#[derive(Debug, Clone)]
pub struct Shape(ShapeVariant);

impl Shape {
    pub fn new_hann() -> Self {
        Self(ShapeVariant::Hann(Hann))
    }

    pub fn new_interp(knots: Vec<f64>, controls: Vec<f64>, degree: usize) -> Self {
        Self(ShapeVariant::Interp(Interp::new(knots, controls, degree)))
    }

    pub fn sample_array(&self, x0: f64, dx: f64, array: &mut [f64]) {
        self.0.sample_array(x0, dx, array);
    }
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
}
