use bevy::math::UVec2;
use bevy::reflect::Reflect;

#[derive(Reflect, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZCoord {
    pub x: u16,
    pub y: u16,
}
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZIndex(pub u32);

impl ZCoord {
    pub fn to_index(self) -> ZIndex {
        self.into()
    }
}
impl ZIndex {
    pub fn to_coord(self) -> ZCoord {
        self.into()
    }
}

impl From<(u16, u16)> for ZCoord {
    fn from(value: (u16, u16)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}
impl From<ZCoord> for ZIndex {
    fn from(value: ZCoord) -> Self {
        Self(index_of(value.x, value.y))
    }
}
impl From<ZIndex> for ZCoord {
    fn from(value: ZIndex) -> Self {
        coord_of(value.0).into()
    }
}
impl From<UVec2> for ZCoord {
    fn from(value: UVec2) -> Self {
        Self {
            x: value.x as u16,
            y: value.y as u16,
        }
    }
}
impl From<ZCoord> for UVec2 {
    fn from(value: ZCoord) -> Self {
        Self {
            x: value.x as u32,
            y: value.y as u32,
        }
    }
}
impl From<u32> for ZIndex {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<usize> for ZIndex {
    fn from(value: usize) -> Self {
        Self(value as u32)
    }
}

impl<T> std::ops::Index<ZCoord> for Vec<T> {
    type Output = T;
    fn index(&self, coord: ZCoord) -> &Self::Output {
        debug_assert!(self.len().is_power_of_two());
        let idx = index_of(coord.x, coord.y);
        &self[idx as usize]
    }
}
impl<T> std::ops::IndexMut<ZCoord> for Vec<T> {
    fn index_mut(&mut self, index: ZCoord) -> &mut Self::Output {
        debug_assert!(self.len().is_power_of_two());
        let idx = index_of(index.x, index.y);
        &mut self[idx as usize]
    }
}
impl<T> std::ops::Index<ZIndex> for Vec<T> {
    type Output = T;
    fn index(&self, index: ZIndex) -> &Self::Output {
        &self[index.0 as usize]
    }
}
impl<T> std::ops::IndexMut<ZIndex> for Vec<T> {
    fn index_mut(&mut self, index: ZIndex) -> &mut Self::Output {
        &mut self[index.0 as usize]
    }
}

#[cfg(not(all(target_feature = "avx2", feature = "bmi2")))]
#[inline]
fn index_of(x: u16, y: u16) -> u32 {
    ::zorder::index_of((x, y))
}

#[cfg(all(target_feature = "avx2", feature = "bmi2"))]
#[inline]
fn index_of(x: u16, y: u16) -> u32 {
    debug_assert!(is_x86_feature_detected!("bmi2"));
    unsafe { ::zorder::bmi2::index_of((x, y)) }
}

#[cfg(not(all(target_feature = "avx2", feature = "bmi2")))]
#[inline]
fn coord_of(idx: u32) -> (u16, u16) {
    ::zorder::coord_of(idx)
}

#[cfg(all(target_feature = "avx2", feature = "bmi2"))]
#[inline]
fn coord_of(idx: u32) -> (u16, u16) {
    debug_assert!(is_x86_feature_detected!("bmi2"));
    unsafe { ::zorder::bmi2::coord_of(idx) }
}
