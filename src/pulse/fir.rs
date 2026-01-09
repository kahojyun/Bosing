use ndarray::{ArrayView1, ArrayViewMut2, Axis};
use pulp::{Arch, Simd, WithSimd};

struct ApplyFirInplace<'a, 'b> {
    waveform: ArrayViewMut2<'a, f64>,
    taps: ArrayView1<'b, f64>,
}

impl WithSimd for ApplyFirInplace<'_, '_> {
    type Output = ();

    #[inline(always)]
    #[expect(clippy::inline_always, reason = "Follow pulp's suggestion")]
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
                let (taps_simd, _) = S::as_simd_f64s(&taps_buffer[tap_buffer_index..]);
                let (w_simd, _) = S::as_simd_f64s(&w_buffer);
                let sum = taps_simd
                    .iter()
                    .zip(w_simd.iter())
                    .fold(simd.splat_f64s(0.0), |acc, (taps, w)| {
                        simd.mul_add_e_f64s(*taps, *w, acc)
                    });
                let sum = simd.reduce_sum_f64s(sum);
                *w = sum;
            }
        }
    }
}

pub fn filter_inplace(waveform: ArrayViewMut2<'_, f64>, taps: ArrayView1<'_, f64>) {
    let arch = Arch::new();
    arch.dispatch(ApplyFirInplace { waveform, taps });
}

#[inline]
const fn align_ceil(x: usize, n: usize) -> usize {
    let r = x % n;
    if r == 0 { x } else { x + n - r }
}

#[cfg(test)]
mod tests {
    use ndarray::{Array2, array, stack};

    use super::*;

    #[test]
    fn test_fir_filter_inplace() {
        let mut signal = Array2::ones((2, 10));
        let taps = array![1.0, 0.1, 0.01];
        let expected = {
            let arr1 = array![1.0, 1.1, 1.11, 1.11, 1.11, 1.11, 1.11, 1.11, 1.11, 1.11];
            stack![Axis(0), arr1, arr1]
        };

        filter_inplace(signal.view_mut(), taps.view());

        assert_eq!(signal, expected);
    }
}
