use bevy::math::{IVec2, UVec2};

use crate::math::{USize2, ZCoord};
// UL, UM, ML, UR, MR, LL, LM, LR
pub const UL: IVec2 = IVec2::new(1, -1);
pub const UM: IVec2 = IVec2::new(1, 0);
pub const ML: IVec2 = IVec2::new(0, -1);
pub const UR: IVec2 = IVec2::new(1, 1);
pub const MR: IVec2 = IVec2::new(0, 1);
pub const LL: IVec2 = IVec2::new(-1, -1);
pub const LM: IVec2 = IVec2::new(-1, 0);
pub const LR: IVec2 = IVec2::new(-1, 1);

/// copied from rust source for unstable feature `get_many_mut`
///
/// adapted to take isize indices, where negative indices will set the output at that index to None
///
/// # Safety
///
/// Calling this method with overlapping or out-of-bounds indices is *[undefined behavior]*
/// even if the resulting references are not used. Negative indices are allowed.
#[inline]
pub unsafe fn slice_get_many_unchecked_mut<T, const N: usize>(
    slice: &mut [T],
    indices: [isize; N],
) -> [Option<&mut T>; N] {
    // NB: This implementation is written as it is because any variation of
    // `indices.map(|i| self.get_unchecked_mut(i))` would make miri unhappy,
    // or generate worse code otherwise. This is also why we need to go
    // through a raw pointer here.
    let slice_ptr: *mut T = slice.as_mut_ptr();
    let mut arr: std::mem::MaybeUninit<[Option<&mut T>; N]> = std::mem::MaybeUninit::uninit();
    let arr_ptr = arr.as_mut_ptr();

    // SAFETY: We expect `indices` to contain disjunct values that are
    // in bounds of `self`.
    unsafe {
        for i in 0..N {
            let idx = *indices.get_unchecked(i);
            if idx >= 0 {
                *(*arr_ptr).get_unchecked_mut(i) = Some(&mut *(slice_ptr.add(idx as usize)));
            } else {
                *(*arr_ptr).get_unchecked_mut(i) = None;
            }
        }
        arr.assume_init()
    }
}

/// copied from rust source for unstable feature `get_many_mut`
///
/// adapted to take isize indices, where negative indices will set the output at that index to None
///
/// # Safety
///
/// Calling this method with overlapping or out-of-bounds indices is *[undefined behavior]*
/// even if the resulting references are not used. Negative indices are allowed.
#[inline]
pub unsafe fn slice_get_many_unchecked<T, const N: usize>(
    slice: &[T],
    indices: [isize; N],
) -> [Option<&T>; N] {
    // NB: This implementation is written as it is because any variation of
    // `indices.map(|i| self.get_unchecked_mut(i))` would make miri unhappy,
    // or generate worse code otherwise. This is also why we need to go
    // through a raw pointer here.
    let slice_ptr: *const T = slice.as_ptr();
    let mut arr: std::mem::MaybeUninit<[Option<&T>; N]> = std::mem::MaybeUninit::uninit();
    let arr_ptr = arr.as_mut_ptr();

    // SAFETY: We expect `indices` to contain disjunct values that are
    // in bounds of `self`.
    unsafe {
        for i in 0..N {
            let idx = *indices.get_unchecked(i);
            if idx >= 0 {
                *(*arr_ptr).get_unchecked_mut(i) = Some(&*(slice_ptr.add(idx as usize)));
            } else {
                *(*arr_ptr).get_unchecked_mut(i) = None;
            }
        }
        arr.assume_init()
    }
}

#[inline]
pub fn count_non_negative<const N: usize>(array: [isize; N]) -> usize {
    array.iter().filter(|&&v| v >= 0).count()
}

#[inline]
pub const fn zorder_n8_offsets() -> [IVec2; 8] {
    [UL, UM, ML, UR, MR, LL, LM, LR]
}

#[inline]
pub fn zorder_n8_indices(coord: UVec2, size: USize2) -> [isize; 8] {
    let c = |off: IVec2| -> isize {
        let new_coord = (coord.as_ivec2() + off).as_uvec2();
        if size.bounds_check(new_coord) {
            ZCoord::from(new_coord).to_index().0 as isize
        } else {
            -1
        }
    };
    [c(UL), c(UM), c(ML), c(UR), c(MR), c(LL), c(LM), c(LR)]
}
