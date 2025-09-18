// (Value, Original Index, Previous Value)
#[derive(Default)]
pub struct Eytzinger<T>(pub Vec<(T, usize, Option<T>)>);

fn eytzinger<T: Copy>(a: &[(T, usize, Option<T>)], b: &mut [(T, usize, Option<T>)], mut i: usize, k: usize) -> usize {
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
        let new_input = input.iter().copied()
            .enumerate()
            .zip(input.iter().enumerate().map(|x| {
                if x.0 > 0 {
                    input.get(x.0 - 1).cloned()
                } else {
                    None
                }
            }))
            .map(|((idx, v), previous)| (v, idx, previous))
            .collect::<Vec<_>>();

        let mut result = vec![(T::default(), input.len(), input.last().copied()); input.len() + 1];
        eytzinger(&new_input[..], &mut result[..], 0, 1);

        Self(result)
    }
}

impl<T: Copy + Ord> Eytzinger<T> {
    pub fn get(&self, idx: usize) -> Option<&T> {
        self.0.get(idx).map(|x| &x.0)
    }

    pub fn get_original_index(&self, idx: usize) -> Option<usize> {
        self.0.get(idx).map(|x| x.1)
    }

    pub fn get_previous_value(&self, idx: usize) -> Option<&T> {
        self.0.get(idx).and_then(|x| x.2.as_ref())
    }

    #[inline]
    pub fn binary_search(&self, target: T) -> Result<usize, usize> {
        if self.0.len() == 1 {
            return Err(0);
        }

        let mut idx = 1;

        while idx < self.0.len() {
            #[cfg(target_arch = "x86_64")]
            unsafe {
                use std::arch::x86_64::*;
                let prefetch = self.0.as_ptr().wrapping_offset(2 * idx as isize);
                _mm_prefetch::<_MM_HINT_T0>(std::ptr::addr_of!(prefetch) as *const i8);
            }
            let current = &self.0[idx];
            idx = 2 * idx + usize::from(current.0 < target);
        }

        idx >>= idx.trailing_ones() + 1;

        if self.0[idx].0 == target {
            Ok(idx)
        } else {
            Err(idx)
        }
    }
}

#[test]
fn ok_binary_search() {
    let v = Eytzinger::from(vec![].as_slice());
    assert_eq!(v.binary_search(0).unwrap_err(), 0);

    let v = Eytzinger::from(vec![1].as_slice());
    assert_eq!(v.get_original_index(v.binary_search(0).unwrap_err()).unwrap(), 0);
    assert_eq!(v.get_original_index(v.binary_search(1).unwrap()).unwrap(), 0);
    assert_eq!(v.get_original_index(v.binary_search(2).unwrap_err()).unwrap(), 1);

    let v = Eytzinger::from(vec![1, 5, 10].as_slice());
    assert_eq!(v.get_original_index(v.binary_search(0).unwrap_err()).unwrap(), 0);
    assert_eq!(v.get_original_index(v.binary_search(1).unwrap()).unwrap(), 0);

    assert_eq!(v.get_original_index(v.binary_search(2).unwrap_err()).unwrap(), 1);
    assert_eq!(*v.get_previous_value(v.binary_search(2).unwrap_err()).unwrap(), 1);

    assert_eq!(v.get_original_index(v.binary_search(3).unwrap_err()).unwrap(), 1);
    assert_eq!(v.get_original_index(v.binary_search(4).unwrap_err()).unwrap(), 1);
    assert_eq!(v.get_original_index(v.binary_search(5).unwrap()).unwrap(), 1);
    assert_eq!(v.get_original_index(v.binary_search(6).unwrap_err()).unwrap(), 2);

    assert_eq!(v.get_original_index(v.binary_search(7).unwrap_err()).unwrap(), 2);
    assert_eq!(*v.get_previous_value(v.binary_search(7).unwrap_err()).unwrap(), 5);

    assert_eq!(v.get_original_index(v.binary_search(8).unwrap_err()).unwrap(), 2);
    assert_eq!(v.get_original_index(v.binary_search(9).unwrap_err()).unwrap(), 2);
    assert_eq!(v.get_original_index(v.binary_search(10).unwrap()).unwrap(), 2);
    assert_eq!(v.get_original_index(v.binary_search(11).unwrap_err()).unwrap(), 3);
    assert_eq!(v.get_original_index(v.binary_search(12).unwrap_err()).unwrap(), 3);

    let v = Eytzinger::from(vec![1, 5, 10, 13].as_slice());
    assert_eq!(v.get_original_index(v.binary_search(0).unwrap_err()).unwrap(), 0);
    assert_eq!(v.get_original_index(v.binary_search(1).unwrap()).unwrap(), 0);
    assert_eq!(v.get_original_index(v.binary_search(2).unwrap_err()).unwrap(), 1);
    assert_eq!(v.get_original_index(v.binary_search(3).unwrap_err()).unwrap(), 1);
    assert_eq!(v.get_original_index(v.binary_search(4).unwrap_err()).unwrap(), 1);
    assert_eq!(v.get_original_index(v.binary_search(5).unwrap()).unwrap(), 1);
    assert_eq!(v.get_original_index(v.binary_search(6).unwrap_err()).unwrap(), 2);
    assert_eq!(v.get_original_index(v.binary_search(7).unwrap_err()).unwrap(), 2);
    assert_eq!(v.get_original_index(v.binary_search(8).unwrap_err()).unwrap(), 2);
    assert_eq!(v.get_original_index(v.binary_search(9).unwrap_err()).unwrap(), 2);
    assert_eq!(v.get_original_index(v.binary_search(10).unwrap()).unwrap(), 2);
    assert_eq!(v.get_original_index(v.binary_search(11).unwrap_err()).unwrap(), 3);
    assert_eq!(v.get_original_index(v.binary_search(12).unwrap_err()).unwrap(), 3);
    assert_eq!(v.get_original_index(v.binary_search(13).unwrap()).unwrap(), 3);
    assert_eq!(v.get_original_index(v.binary_search(14).unwrap_err()).unwrap(), 4);
}
