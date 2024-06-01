use ndarray::{ArrayView1, ArrayViewMut2, Axis};
use pulp::{Arch, Simd, WithSimd};

struct ApplyFirInplace<'a, 'b> {
    waveform: ArrayViewMut2<'a, f64>,
    taps: ArrayView1<'b, f64>,
}

impl<'a, 'b> WithSimd for ApplyFirInplace<'a, 'b> {
    type Output = ();

    #[inline(always)]
    fn with_simd<S: Simd>(mut self, simd: S) -> Self::Output {
        let lanes = std::mem::size_of::<S::f64s>() / std::mem::size_of::<f64>();
        let buffer_len = align_ceil(self.taps.len(), lanes);
        assert!(buffer_len % lanes == 0);
        let taps_buffer = {
            let mut buffer = vec![0.0; buffer_len * 2];
            for (&t, b) in self.taps.iter().zip(buffer[..buffer_len].iter_mut().rev()) {
                *b = t;
            }
            for (&t, b) in self.taps.iter().zip(buffer[buffer_len..].iter_mut().rev()) {
                *b = t;
            }
            buffer
        };
        for mut row in self.waveform.axis_iter_mut(Axis(0)) {
            let mut w_buffer = vec![0.0; buffer_len];
            for (i, w) in row.iter_mut().enumerate() {
                let w_buffer_index = i % buffer_len;
                w_buffer[w_buffer_index] = *w;
                let tap_buffer_index = buffer_len - w_buffer_index - 1;
                let (taps_simd, _) = S::f64s_as_simd(&taps_buffer[tap_buffer_index..]);
                let (w_simd, _) = S::f64s_as_simd(&w_buffer);
                let sum = taps_simd
                    .iter()
                    .zip(w_simd.iter())
                    .fold(simd.f64s_splat(0.0), |acc, (taps, w)| {
                        simd.f64s_mul_add_e(*taps, *w, acc)
                    });
                let sum = simd.f64s_reduce_sum(sum);
                *w = sum;
            }
        }
    }
}

pub(crate) fn fir_filter_inplace(waveform: ArrayViewMut2<f64>, taps: ArrayView1<f64>) {
    let arch = Arch::new();
    arch.dispatch(ApplyFirInplace { waveform, taps });
}

#[inline]
fn align_ceil(x: usize, n: usize) -> usize {
    let r = x % n;
    if r == 0 {
        x
    } else {
        x + n - r
    }
}

#[cfg(test)]
mod tests {
    use ndarray::{array, stack, Array2};

    use super::*;

    #[test]
    fn test_fir_filter_inplace() {
        let mut signal = Array2::ones((2, 10));
        let taps = array![1.0, 0.1, 0.01];
        let expected = {
            let arr1 = array![1.0, 1.1, 1.11, 1.11, 1.11, 1.11, 1.11, 1.11, 1.11, 1.11];
            stack![Axis(0), arr1, arr1]
        };

        fir_filter_inplace(signal.view_mut(), taps.view());

        assert_eq!(signal, expected);
    }
}
