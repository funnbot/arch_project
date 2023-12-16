use std::{iter::FusedIterator, marker::PhantomData, ptr::NonNull};

use bevy::{math::UVec2, reflect::TypePath};
use rand::{
    distributions::{uniform::UniformSampler, DistIter, Uniform},
    Rng,
};

use super::cell::Cell;
use crate::{
    math::{USize2, UVec2Sample, UniformUVec2, ZCoord},
    preludes::{bevy::GlobalRng, core::*},
    sim::n8,
};

#[derive(Debug, Default, reflect::Reflect)]
pub struct CellGrid {
    #[reflect(ignore)]
    cells: Vec<Cell>,
    size: USize2,
}

impl CellGrid {
    pub fn new(size: USize2) -> Self {
        let cells = vec![default(); zorder_data_len(size.0)];
        Self { cells, size }
    }
    /// length of data array, always the nearest power of two, squared, of the grid size
    pub fn len(&self) -> usize {
        self.cells.len()
    }
    /// grid size of the cells that are used.
    ///
    /// size.width * size.height <= len()
    ///
    /// may have extra unused cells allocated if grid is not a power of two square.
    pub fn grid_size(&self) -> USize2 {
        self.size
    }
    pub fn get_cell(&self, coord: UVec2) -> Option<&Cell> {
        if self.size.bounds_check(coord) {
            let zidx = ZCoord::from(coord).to_index().0 as usize;
            unsafe { Some(self.cells.get_unchecked(zidx)) }
        } else {
            None
        }
    }
    pub fn get_cell_mut(&mut self, coord: UVec2) -> Option<&mut Cell> {
        if self.size.bounds_check(coord) {
            let zidx = ZCoord::from(coord).to_index().0 as usize;
            unsafe { Some(self.cells.get_unchecked_mut(zidx)) }
        } else {
            None
        }
    }
    pub fn get_cell_n8(&self, coord: UVec2) -> Option<WithN8> {
        if self.size.bounds_check(coord) {
            let zidx = ZCoord::from(coord).to_index().0 as usize;
            let cells = self.cells.as_ptr();
            let size = self.size;
            unsafe {
                Some((
                    &*cells.add(zidx),
                    n8::slice_ptr_get_many_unchecked(cells, n8::zorder_n8_indices(coord, size)),
                ))
            }
        } else {
            None
        }
    }
    pub fn get_cell_n8_mut(&mut self, coord: UVec2) -> Option<WithN8Mut> {
        if self.size.bounds_check(coord) {
            let zidx = ZCoord::from(coord).to_index().0 as usize;
            let cells = self.cells.as_mut_ptr();
            let size = self.size;
            unsafe {
                Some((
                    &mut *cells.add(zidx),
                    n8::slice_ptr_get_many_unchecked_mut(
                        NonNull::new(cells).unwrap(),
                        n8::zorder_n8_indices(coord, size),
                    ),
                ))
            }
        } else {
            None
        }
    }
    pub fn iter_all(&self) -> std::slice::Iter<'_, Cell> {
        self.cells.iter()
    }
    pub fn iter_all_mut(&mut self) -> std::slice::IterMut<'_, Cell> {
        self.cells.iter_mut()
    }
    pub fn iter(&self) -> impl Iterator<Item = &Cell> {
        let grid_size = self.size;
        self.cells
            .iter()
            .filter(move |cell| grid_size.bounds_check(cell.coord.into()))
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Cell> {
        let grid_size = self.size;
        self.cells
            .iter_mut()
            .filter(move |cell| grid_size.bounds_check(cell.coord.into()))
    }
    pub fn rand_iter<'a>(&'a self, rng: &'a mut impl Rng) -> impl Iterator<Item = &'a Cell> {
        let size = self.size;
        RandIter::new(self, rng, move |c| size.bounds_check(c.coord.into()))
    }
    pub fn rand_iter_mut<'a, R: Rng>(&'a mut self, rng: R) -> impl Iterator<Item = &mut Cell> {
        let size = self.size;
        RandIterMut::new(self, rng, move |c| size.bounds_check(c.coord.into()))
    }
    pub fn iter_land(&self) -> impl Iterator<Item = &Cell> {
        // is_land will only be set on bounds checked cells
        self.cells.iter().filter(|c| c.is_land)
    }
    pub fn iter_land_mut(&mut self) -> impl Iterator<Item = &mut Cell> {
        // is_land will only be set on bounds checked cells
        self.cells.iter_mut().filter(|c| c.is_land)
    }
    pub fn iter_n8(&self) -> impl Iterator<Item = WithN8> {
        let iter = self.iter();
        unsafe { IterWithN8::new(iter, self.cells.as_ptr(), self.size) }
    }
    pub fn iter_n8_mut(&mut self) -> impl Iterator<Item = WithN8Mut> {
        let cells = self.cells.as_mut_ptr();
        let size = self.size;
        let iter = self.iter_mut();
        unsafe { IterWithN8Mut::new(iter, cells, size) }
    }
    pub fn iter_land_n8(&self) -> impl Iterator<Item = WithN8> {
        let iter = self.iter_land();
        unsafe { IterWithN8::new(iter, self.cells.as_ptr(), self.size) }
    }
    pub fn iter_land_n8_mut(&mut self) -> impl Iterator<Item = WithN8Mut> {
        let cells = self.cells.as_mut_ptr();
        let size = self.size;
        let iter = self.iter_land_mut();
        unsafe { IterWithN8Mut::new(iter, cells, size) }
    }
    /// n-of
    pub fn rand_of_iter_land_n8<'a, R: Rng>(
        &'a self,
        rng: &'a mut R,
        count: usize,
    ) -> impl Iterator<Item = WithN8> {
        let cells = self.cells.as_ptr();
        let size = self.size;
        let iter = RandIter::new(self, rng, |c| c.is_land);
        unsafe { IterWithN8::new(iter, cells, size) }.take(count)
    }
    pub fn rand_of_iter_land_n8_mut<R: Rng>(
        &mut self,
        rng: R,
        count: usize,
    ) -> impl Iterator<Item = WithN8Mut> {
        let cells = self.cells.as_mut_ptr();
        let size = self.size;
        let iter = RandIterMut::new(self, rng, |c| c.is_land);
        unsafe { IterWithN8Mut::new(iter, cells, size) }.take(count)
    }
    pub fn rand_of_iter_land<'a, R: Rng>(
        &'a self,
        rng: &'a mut R,
        count: usize,
    ) -> impl Iterator<Item = &Cell> {
        RandIter::new(self, rng, |c| c.is_land).take(count)
    }
    pub fn rand_of_iter_land_mut<R: Rng>(
        &mut self,
        rng: R,
        count: usize,
    ) -> impl Iterator<Item = &mut Cell> {
        RandIterMut::new(self, rng, |c| c.is_land).take(count)
    }
}

pub struct RandIter<'a, R: Rng, F: FnMut(&Cell) -> bool> {
    cells: *const Cell,
    rng_iter: DistIter<Uniform<u32>, &'a mut R, u32>,
    filter_fn: F,
    marker: PhantomData<&'a CellGrid>,
}
impl<'a, R: Rng, F: FnMut(&Cell) -> bool> RandIter<'a, R, F> {
    pub fn new(grid: &'a CellGrid, rng: &'a mut R, filter_fn: F) -> Self {
        let dist = Uniform::new(0, grid.cells.len() as u32);
        Self {
            cells: grid.cells.as_ptr(),
            rng_iter: rng.sample_iter(dist),
            filter_fn,
            marker: PhantomData,
        }
    }
}
impl<'a, R: Rng, F: FnMut(&Cell) -> bool> Iterator for RandIter<'a, R, F> {
    type Item = &'a Cell;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let zidx = self.rng_iter.next().expect("rng iter never ends");
            // SAFETY: rng_iter returns values in range 0..cells.len()
            let cell = unsafe { &*self.cells.add(zidx as usize) };
            if (self.filter_fn)(cell) {
                return Some(cell);
            }
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.rng_iter.size_hint()
    }
}

pub struct RandIterMut<'a, R: Rng, F: FnMut(&mut Cell) -> bool> {
    cells: NonNull<Cell>,
    rng_iter: DistIter<Uniform<u32>, R, u32>,
    filter_fn: F,
    marker: PhantomData<&'a mut CellGrid>,
}
impl<'a, R: Rng, F: FnMut(&mut Cell) -> bool> RandIterMut<'a, R, F> {
    pub fn new(grid: &'a mut CellGrid, rng: R, filter_fn: F) -> Self {
        let dist = Uniform::new(0, grid.cells.len() as u32);
        Self {
            cells: NonNull::new(grid.cells.as_mut_ptr()).unwrap(),
            rng_iter: rng.sample_iter(dist),
            filter_fn,
            marker: PhantomData,
        }
    }
}
impl<'a, R: Rng, F: FnMut(&mut Cell) -> bool> Iterator for RandIterMut<'a, R, F> {
    type Item = &'a mut Cell;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let zidx = self.rng_iter.next().expect("rng iter never ends");
            // SAFETY: rng_iter returns values in range 0..cells.len()
            let cell = unsafe { &mut *self.cells.as_ptr().add(zidx as usize) };
            if (self.filter_fn)(cell) {
                return Some(cell);
            }
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.rng_iter.size_hint()
    }
}

#[derive(Clone)]
pub struct IterWithN8<'a, I: Iterator<Item = &'a Cell>> {
    iter: I,
    cells: *const Cell,
    grid_size: USize2,
}
impl<'a, I: Iterator<Item = &'a Cell>> IterWithN8<'a, I> {
    pub unsafe fn new(iter: I, cells: *const Cell, grid_size: USize2) -> Self {
        Self {
            iter,
            cells,
            grid_size,
        }
    }
}
impl<'a, I: Iterator<Item = &'a Cell>> Iterator for IterWithN8<'a, I> {
    type Item = (&'a Cell, [Option<&'a Cell>; 8]);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Some(cell) = self.iter.next() else {
            return None;
        };
        let n8s = unsafe {
            n8::slice_ptr_get_many_unchecked::<'a>(
                self.cells,
                n8::zorder_n8_indices(cell.coord.into(), self.grid_size),
            )
        };
        Some((cell, n8s))
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

#[derive(Clone)]
pub struct IterWithN8Mut<'a, I: Iterator<Item = &'a mut Cell>> {
    iter: I,
    cells: NonNull<Cell>,
    grid_size: USize2,
}
impl<'a, I: Iterator<Item = &'a mut Cell>> IterWithN8Mut<'a, I> {
    pub unsafe fn new(iter: I, cells: *mut Cell, grid_size: USize2) -> Self {
        Self {
            iter,
            cells: NonNull::new(cells).unwrap(),
            grid_size,
        }
    }
}
impl<'a, I: Iterator<Item = &'a mut Cell>> Iterator for IterWithN8Mut<'a, I> {
    type Item = (&'a mut Cell, [Option<&'a mut Cell>; 8]);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Some(cell) = self.iter.next() else {
            return None;
        };
        let n8s = unsafe {
            n8::slice_ptr_get_many_unchecked_mut::<'a>(
                self.cells,
                n8::zorder_n8_indices(cell.coord.into(), self.grid_size),
            )
        };
        Some((cell, n8s))
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

pub type WithN8<'a> = (&'a Cell, [Option<&'a Cell>; 8]);
pub type WithN8Mut<'a> = (&'a mut Cell, [Option<&'a mut Cell>; 8]);

impl<'a, I: Iterator<Item = &'a Cell> + FusedIterator> FusedIterator for IterWithN8<'a, I> {}
impl<'a, I: Iterator<Item = &'a mut Cell> + FusedIterator> FusedIterator for IterWithN8Mut<'a, I> {}

impl<'a, I: Iterator<Item = &'a Cell> + ExactSizeIterator> ExactSizeIterator for IterWithN8<'a, I> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}
impl<'a, I: Iterator<Item = &'a mut Cell> + ExactSizeIterator> ExactSizeIterator
    for IterWithN8Mut<'a, I>
{
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

fn zorder_data_len(grid_size: UVec2) -> usize {
    (grid_size.max_element().next_power_of_two() as usize).pow(2)
}
