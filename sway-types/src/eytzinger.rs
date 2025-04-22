#[derive(Default)]
pub struct Eytzinger<T>(Vec<T>, Vec<usize>);

#[derive(Debug)]
pub struct Index {
    pub index: usize,
    pub original: usize,
}

fn eytzinger<T: Copy>(a: &[T], b: &mut [T], mut i: usize, k: usize) -> usize {
    if k <= a.len() {
        i = eytzinger(a, b, i, 2 * k);
        b[k] = a[i];
        i += 1;
        i = eytzinger(a, b, i, 2 * k + 1);
    }
    i
}

impl<T: Copy + Default> From<&[T]> for Eytzinger<T> {
    fn from(input: &[T]) -> Self {
        let mut result = vec![T::default(); input.len() + 1];
        eytzinger(&input[..], &mut result[..], 0, 1);
        let original = (0..input.len()).into_iter().collect::<Vec<_>>();
        let mut order = vec![input.len(); input.len() + 1];
        eytzinger(&original[..], &mut order[..], 0, 1);
        Self(result, order)
    }
}

impl<T: Copy + Ord> Eytzinger<T> {
    pub fn get(&self, idx: usize) -> Option<&T> {
        self.0.get(idx)
    }

    #[inline]
    pub fn binary_search(&self, target: T) -> Result<Index, Index> {
        let mut idx = 1;

        while idx < self.0.len() {
            #[cfg(target_arch = "x86_64")]
            unsafe {
                use std::arch::x86_64::*;
                let prefetch = self.0.as_ptr().wrapping_offset(2 * idx as isize);
                _mm_prefetch::<_MM_HINT_T0>(std::ptr::addr_of!(prefetch) as *const i8);
            }
            let current = self.0[idx];
            idx = 2 * idx + usize::from(current < target);
        }

        idx >>= idx.trailing_ones() + 1;

        let r = Index {
            index: idx,
            original: self.1[idx],
        };

        if self.0[idx] == target {
            Ok(r)
        } else {
            Err(r)
        }
    }
}

#[test]
fn ok_binary_search() {
    let v = Eytzinger::from(vec![1, 5, 10].as_slice());
    assert_eq!(v.binary_search(1).unwrap().original, 0);
    assert_eq!(v.binary_search(5).unwrap().original, 1);
    assert_eq!(v.binary_search(10).unwrap().original, 2);

    assert_eq!(v.binary_search(0).unwrap_err().original, 0);
    assert_eq!(v.binary_search(2).unwrap_err().original, 1);

    assert_eq!(v.binary_search(4).unwrap_err().original, 1);
    assert_eq!(v.binary_search(6).unwrap_err().original, 2);

    assert_eq!(v.binary_search(9).unwrap_err().original, 2);
    assert_eq!(v.binary_search(11).unwrap_err().original, 3);
}
