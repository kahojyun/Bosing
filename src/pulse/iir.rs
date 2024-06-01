use std::{
    array,
    ops::{Add, Mul, Sub},
};

use ndarray::{ArrayView1, ArrayView2, ArrayViewMut2};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error("Invalid SOS format")]
    InvalidSosFormat,
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy)]
struct BiquadCoefficients<T> {
    b0: T,
    b1: T,
    b2: T,
    a1: T,
    a2: T,
}

#[derive(Debug)]
struct Biquad<T> {
    coefficients: BiquadCoefficients<T>,
    s1: T,
    s2: T,
}

#[derive(Debug)]
struct Iir<T> {
    biquads: Vec<Biquad<T>>,
}

#[derive(Debug)]
struct IirPipeline<T, const N: usize> {
    b0: [T; N],
    b1: [T; N],
    b2: [T; N],
    a1: [T; N],
    a2: [T; N],
    s1: [T; N],
    s2: [T; N],
    y: [T; N],
}

impl<T: Default> Biquad<T> {
    fn new(coefficients: BiquadCoefficients<T>) -> Self {
        Self {
            coefficients,
            s1: Default::default(),
            s2: Default::default(),
        }
    }

    fn reset(&mut self) {
        self.s1 = Default::default();
        self.s2 = Default::default();
    }
}

impl<T> Biquad<T>
where
    T: Add<Output = T> + Mul<Output = T> + Sub<Output = T> + Copy,
{
    fn run(&mut self, x: T) -> T {
        let y = self.coefficients.b0 * x + self.s1;
        self.s1 = self.coefficients.b1 * x - self.coefficients.a1 * y + self.s2;
        self.s2 = self.coefficients.b2 * x - self.coefficients.a2 * y;
        y
    }
}

impl<T: Default> Iir<T> {
    fn reset(&mut self) {
        for biquad in &mut self.biquads {
            biquad.reset();
        }
    }
}

impl<T> Iir<T>
where
    T: Add<Output = T> + Mul<Output = T> + Sub<Output = T> + Copy,
{
    fn run(&mut self, x: T) -> T {
        let mut y = x;
        for biquad in &mut self.biquads {
            y = biquad.run(y);
        }
        y
    }

    fn filter_inplace(&mut self, x: &mut [T]) {
        for x in x.iter_mut() {
            *x = self.run(*x);
        }
    }
}

impl<T, const N: usize> IirPipeline<T, N>
where
    [T; N]: Default,
{
    fn reset(&mut self) {
        self.s1 = Default::default();
        self.s2 = Default::default();
        self.y = Default::default();
    }
}

impl<T, const N: usize> IirPipeline<T, N>
where
    T: Add<Output = T> + Mul<Output = T> + Sub<Output = T> + Copy + Default,
{
    fn run(&mut self, x: T) -> T {
        let res = self.y[N - 1];
        for i in (0..N).rev() {
            let x = if i == 0 { x } else { self.y[i - 1] };
            let y = self.b0[i] * x + self.s1[i];
            self.s1[i] = self.b1[i] * x - self.a1[i] * y + self.s2[i];
            self.s2[i] = self.b2[i] * x - self.a2[i] * y;
            self.y[i] = y;
        }
        res
    }

    fn filter_inplace(&mut self, signal: &mut [T]) {
        for i in 0..signal.len() + N {
            let x = if i < signal.len() {
                signal[i]
            } else {
                Default::default()
            };
            let y = self.run(x);
            if i >= N {
                signal[i - N] = y;
            }
        }
    }
}

impl<'a, T: Copy> TryFrom<ArrayView1<'a, T>> for BiquadCoefficients<T> {
    type Error = Error;

    fn try_from(value: ArrayView1<'a, T>) -> Result<Self> {
        if value.dim() != 6 {
            return Err(Error::InvalidSosFormat);
        }
        Ok(Self {
            b0: value[0],
            b1: value[1],
            b2: value[2],
            a1: value[4],
            a2: value[5],
        })
    }
}

impl<'a, T: Copy + Default> TryFrom<ArrayView1<'a, T>> for Biquad<T> {
    type Error = Error;

    fn try_from(value: ArrayView1<'a, T>) -> Result<Self> {
        Ok(Self::new(value.try_into()?))
    }
}

impl<'a, T: Copy + Default> TryFrom<ArrayView2<'a, T>> for Iir<T> {
    type Error = Error;

    fn try_from(value: ArrayView2<'a, T>) -> Result<Self> {
        let biquads = value
            .outer_iter()
            .map(Biquad::try_from)
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { biquads })
    }
}

impl<'a, T, const N: usize> TryFrom<ArrayView2<'a, T>> for IirPipeline<T, N>
where
    T: Copy + Default,
    [T; N]: Default,
{
    type Error = Error;

    fn try_from(value: ArrayView2<'a, T>) -> Result<Self> {
        if value.dim().0 != N {
            panic!("N should be equal to the number of biquads in the pipeline");
        }
        if value.dim().1 != 6 {
            return Err(Error::InvalidSosFormat);
        }
        let b0 = array::from_fn(|i| value[(i, 0)]);
        let b1 = array::from_fn(|i| value[(i, 1)]);
        let b2 = array::from_fn(|i| value[(i, 2)]);
        let a1 = array::from_fn(|i| value[(i, 4)]);
        let a2 = array::from_fn(|i| value[(i, 5)]);
        Ok(Self {
            b0,
            b1,
            b2,
            a1,
            a2,
            s1: Default::default(),
            s2: Default::default(),
            y: Default::default(),
        })
    }
}

pub(crate) fn iir_filter_inplace<T>(signal: ArrayViewMut2<T>, sos: ArrayView2<T>) -> Result<()>
where
    T: Add<Output = T> + Mul<Output = T> + Sub<Output = T> + Copy + Default,
{
    match sos.dim().0 {
        0 => Ok(()),
        1 => specialized_filter::<T, 1>(signal, sos),
        2 => specialized_filter::<T, 2>(signal, sos),
        3 => specialized_filter::<T, 3>(signal, sos),
        4 => specialized_filter::<T, 4>(signal, sos),
        _ => fallback_filter(signal, sos),
    }
}

fn specialized_filter<T, const N: usize>(
    mut signal: ArrayViewMut2<T>,
    sos: ArrayView2<T>,
) -> Result<()>
where
    T: Add<Output = T> + Mul<Output = T> + Sub<Output = T> + Copy + Default,
    [T; N]: Default,
{
    let mut iir: IirPipeline<T, N> = sos.try_into()?;
    for mut row in signal.outer_iter_mut() {
        let row = row.as_slice_mut().expect("Row should be contiguous");
        iir.reset();
        iir.filter_inplace(row);
    }
    Ok(())
}

fn fallback_filter<T>(mut signal: ArrayViewMut2<T>, sos: ArrayView2<T>) -> Result<()>
where
    T: Add<Output = T> + Mul<Output = T> + Sub<Output = T> + Copy + Default,
{
    let mut iir: Iir<T> = sos.try_into()?;
    for mut row in signal.outer_iter_mut() {
        let row = row.as_slice_mut().expect("Row should be contiguous");
        iir.reset();
        iir.filter_inplace(row);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use ndarray::{array, stack, Array2, Axis};

    use super::*;

    fn get_test_case() -> (Array2<f64>, Array2<f64>, Array2<f64>) {
        // Generated using scipy.signal.sosfilt
        let signal = Array2::ones((2, 10));
        let sos = array![
            [
                0.41745636685930126,
                -0.8124669318632639,
                0.39511212971824017,
                1.0,
                -1.9517167839624654,
                0.9518873960332694
            ],
            [
                1.0,
                -1.998900504749875,
                0.9989006046949026,
                1.0,
                -1.9990955877237742,
                0.9990956472206675
            ],
        ];
        let expected = {
            let arr1d = array![
                0.41745636685930126,
                0.419827471396848,
                0.4221187550258305,
                0.4243336840250656,
                0.4264755709173414,
                0.42854758130187326,
                0.43055274038309926,
                0.4324939392093115,
                0.4343739406340189,
                0.4361953850123652
            ];
            stack![Axis(0), arr1d, arr1d]
        };
        (signal, sos, expected)
    }

    #[test]
    fn test_specialized_filter() {
        let (mut signal, sos, expected) = get_test_case();
        specialized_filter::<_, 2>(signal.view_mut(), sos.view()).unwrap();
        assert_eq!(signal, expected);
    }

    #[test]
    fn test_fallback_filter() {
        let (mut signal, sos, expected) = get_test_case();
        fallback_filter(signal.view_mut(), sos.view()).unwrap();
        assert_eq!(signal, expected);
    }
}
