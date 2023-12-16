// TODO: This doesn't work because the idea is that each space has a single value (rect, size, etc), yet currently you can define multiple spaces with the same type, ideally each space would have a comptime known value, but some spaces (like screen) don't have a comptime known value
// So, a Space should be a struct then, and each space, (World, Grid, etc) will be defined by the user.
// Then it is up to the user to the user to not define the same space with different names (this conveys the semantics better anyway)
// Will need multiple space types for the different kinds of spaces though, if they have rect, size, origin, etc.

use std::marker::PhantomData;

use bevy::{
    math::{DVec2, Rect, UVec2, Vec2, Vec2Swizzles},
    reflect,
};

use super::{DRect, DSize2, Size2, USize2};

/// The space used by Bevy.
///
/// A "GameCoord2" would be [`Vec2`]
#[derive(Default, Debug, Clone, PartialEq, reflect::Reflect)]
pub struct GameSpace;

/// Space defined by the camera view and window dimensions.
#[derive(Debug, Clone, PartialEq, reflect::Reflect)]
pub struct ScreenSpace {
    view: Rect,
    window_size: USize2,
    pixel_size: Size2,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, reflect::Reflect)]
pub struct ScreenIndex2(pub UVec2);

/// expected to only have one instance.
#[derive(Default, Debug, Clone, PartialEq, reflect::Reflect)]
pub struct MapSpace {
    rect: DRect,
    grid_size: USize2,
    cell_size: DSize2,
}

impl MapSpace {
    pub fn from_rect(rect: DRect, grid_size: USize2) -> Self {
        Self {
            rect,
            grid_size,
            cell_size: DSize2(rect.size() / grid_size.0.as_dvec2()),
        }
    }
    pub fn rect(&self) -> &DRect {
        &self.rect
    }
    pub fn grid_size(&self) -> &USize2 {
        &self.grid_size
    }
    pub fn cell_size(&self) -> &DSize2 {
        &self.cell_size
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, reflect::Reflect)]
pub struct MapCoord2(pub DVec2);
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, reflect::Reflect)]
pub struct MapIndex2(pub UVec2);

/// expected to only have one instance.
#[derive(Debug, Clone, PartialEq, reflect::Reflect)]
pub struct GeoSpace {
    rect: DRect,
}

/// instance created for each imported AsciiGrid
#[derive(Debug, Clone, PartialEq, reflect::Reflect)]
pub struct GeoGridSpace {
    rect: DRect,
    grid_size: USize2,
    cell_size: DSize2,
}

impl GeoGridSpace {
    pub fn rect(&self) -> &DRect {
        &self.rect
    }
    pub fn grid_size(&self) -> &USize2 {
        &self.grid_size
    }
    pub fn cell_size(&self) -> &DSize2 {
        &self.cell_size
    }
    /// From lower left corner
    pub fn from_ll_corner(ll_corner: DVec2, grid_size: USize2, cell_size: DSize2) -> Self {
        let rect = DRect::from_corners(
            ll_corner,
            ll_corner + (grid_size.0.as_dvec2() * cell_size.0),
        );
        Self {
            rect,
            grid_size,
            cell_size,
        }
    }
    pub fn from_rect(rect: DRect, grid_size: USize2) -> Self {
        Self {
            rect,
            grid_size,
            cell_size: DSize2(rect.size() / grid_size.0.as_dvec2()),
        }
    }
    /// finds the maximum size to fit the entire grid within the rect, preserving the aspect ratio
    pub fn from_max_size(max_size: DVec2, grid_size: USize2, ll_corner: DVec2) -> Self {
        let grid_size_d = grid_size.0.as_dvec2();
        let size = max_size_with_aspect_d(grid_size_d, max_size);
        let rect = DRect::from_corners(ll_corner, ll_corner + size);
        Self {
            rect,
            grid_size,
            cell_size: DSize2(size / grid_size_d),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, reflect::Reflect)]
pub struct GeoCoord2(pub DVec2);
/// file is stored with each line being a row of data values, so it is read in row-major.
///
/// Stored in row-major, indices are (row, column), size is (n_columns, n_rows)
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, reflect::Reflect)]
pub struct GeoIndex2(pub UVec2);
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, reflect::Reflect)]
pub struct GeoIndex(pub usize);

pub trait Space: Sized {
    type Coord2;
    type Index2;
    type Index;
}
// pub trait RealSpace: Space {
//     type Coord2;
// }
// pub trait GridSpace: Space {
//     type Index2;
// }
// pub trait IndexSpace: Space {
//     type Index;
// }

impl Space for GameSpace {
    type Coord2 = Vec2;
    type Index2 = UVec2;
    type Index = usize;
}
impl Space for ScreenSpace {
    type Coord2 = !;
    type Index2 = ScreenIndex2;
    type Index = usize;
}
impl Space for MapSpace {
    type Coord2 = MapCoord2;
    type Index2 = MapIndex2;
    type Index = !;
}
impl Space for GeoSpace {
    type Coord2 = GeoCoord2;
    type Index2 = !;
    type Index = !;
}
impl Space for GeoGridSpace {
    type Coord2 = GeoCoord2;
    type Index2 = GeoIndex2;
    type Index = GeoIndex;
}

#[derive(Default, Debug, Clone, Copy, PartialEq, reflect::Reflect)]
pub struct Transformer<From: Space, To: Space> {
    scale: DVec2,
    translation: DVec2,
    #[reflect(ignore)]
    from_marker: PhantomData<From>,
    #[reflect(ignore)]
    to_marker: PhantomData<To>,
}

pub trait TransformCoord<From: Space, To: Space> {
    fn new(from: &From, to: &To) -> Self;
    fn transform(&self, coord: &From::Coord2) -> To::Coord2;
}

pub enum ScaleMode {
    /// scale to fit the entire rect within the target rect, preserving aspect ratio
    Fit,
    /// scale to fill the entire target rect, preserving aspect ratio
    Fill,
    /// scale to fill the entire target rect, not preserving aspect ratio
    Stretch,
}

pub enum Origin {
    Center,
    LowerLeft,
}

impl<From: Space, To: Space> Transformer<From, To> {
    fn from_origins_and_scale(
        from_origin: DVec2,
        to_origin: DVec2,
        scale: DVec2,
        scale_mode: ScaleMode,
    ) -> Self {
        let scale = match scale_mode {
            ScaleMode::Stretch => scale,
            ScaleMode::Fill => DVec2::splat(scale.min_element()),
            ScaleMode::Fit => DVec2::splat(scale.max_element()),
        };
        Self {
            scale,
            translation: to_origin - from_origin * scale,
            from_marker: PhantomData,
            to_marker: PhantomData,
        }
    }
    fn from_rects(
        from_rect: &DRect,
        to_rect: &DRect,
        scale_mode: ScaleMode,
        origin: Origin,
    ) -> Self {
        let scale = to_rect.size() / from_rect.size();
        let origins = match origin {
            Origin::Center => (from_rect.center(), to_rect.center()),
            Origin::LowerLeft => (from_rect.min, to_rect.min),
        };
        Self::from_origins_and_scale(origins.0, origins.1, scale, scale_mode)
    }
    fn transform_dvec2(&self, coord: &DVec2) -> DVec2 {
        *coord * self.scale + self.translation
    }
}

impl TransformCoord<GameSpace, MapSpace> for Transformer<GameSpace, MapSpace> {
    fn new(_from: &GameSpace, to: &MapSpace) -> Self {
        Self::from_origins_and_scale(
            DVec2::ZERO,
            to.rect.center(),
            DVec2::ONE,
            ScaleMode::Stretch,
        )
    }
    fn transform(&self, coord: &Vec2) -> MapCoord2 {
        MapCoord2(coord.as_dvec2())
    }
}
impl Transformer<MapSpace, GameSpace> {
    pub fn new(from: &MapSpace, _to: &GameSpace) -> Self {
        Self::from_origins_and_scale(
            from.rect.center(),
            DVec2::ZERO,
            DVec2::ONE,
            ScaleMode::Stretch,
        )
    }
    pub fn transform(&self, coord: &MapCoord2) -> Vec2 {
        coord.0.as_vec2()
    }
}

impl Transformer<GeoGridSpace, MapSpace> {
    pub fn new(from: &GeoGridSpace, to: &MapSpace) -> Self {
        Self::from_rects(&from.rect, &to.rect, ScaleMode::Fit, Origin::Center)
    }
    pub fn transform(&self, coord: &GeoCoord2) -> MapCoord2 {
        MapCoord2(self.transform_dvec2(&coord.0))
    }
}
impl Transformer<MapSpace, GeoGridSpace> {
    pub fn new(from: &MapSpace, to: &GeoGridSpace) -> Self {
        Self::from_rects(&from.rect, &to.rect, ScaleMode::Fit, Origin::Center)
    }
    pub fn transform(&self, coord: &MapCoord2) -> GeoCoord2 {
        GeoCoord2(self.transform_dvec2(&coord.0))
    }
}

pub trait GridTransform: Space {
    /// grid index2 from coord
    fn to_grid_trunc(&self, coord: &Self::Coord2) -> Self::Index2;
    /// coord from grid index2, lower left of the cell
    fn ll_from_grid(&self, index: &Self::Index2) -> Self::Coord2;
    /// coord from grid index2, center of the cell
    fn center_from_grid(&self, index: &Self::Index2) -> Self::Coord2;
    /// convert a coord anywhere in a cell, to the bottom left corner of that cell
    #[inline]
    fn ll_from_coord(&self, coord: &Self::Coord2) -> Self::Coord2 {
        self.ll_from_grid(&self.to_grid_trunc(coord))
    }
}

impl GridTransform for MapSpace {
    #[inline]
    fn to_grid_trunc(&self, coord: &Self::Coord2) -> Self::Index2 {
        MapIndex2(((coord.0 - self.rect.min) / self.cell_size.0).as_uvec2())
    }
    #[inline]
    fn ll_from_grid(&self, index: &Self::Index2) -> Self::Coord2 {
        MapCoord2((index.0.as_dvec2() * self.cell_size.0) + self.rect.min)
    }
    #[inline]
    fn center_from_grid(&self, index: &Self::Index2) -> Self::Coord2 {
        MapCoord2(((index.0.as_dvec2() + 0.5) * self.cell_size.0) + self.rect.min)
    }
}

impl GridTransform for GeoGridSpace {
    #[inline]
    fn to_grid_trunc(&self, coord: &Self::Coord2) -> Self::Index2 {
        GeoIndex2(((coord.0 - self.rect.min) / self.cell_size.0).as_uvec2())
    }
    #[inline]
    fn ll_from_grid(&self, index: &Self::Index2) -> Self::Coord2 {
        GeoCoord2((index.0.as_dvec2() * self.cell_size.0) + self.rect.min)
    }
    #[inline]
    fn center_from_grid(&self, index: &Self::Index2) -> Self::Coord2 {
        GeoCoord2(((index.0.as_dvec2() + 0.5) * self.cell_size.0) + self.rect.min)
    }
}

pub trait IndexTransform: Space {
    fn flatten_index2(&self, index2: &Self::Index2) -> Self::Index;
    fn fold_index(&self, index: &Self::Index) -> Self::Index2;
}

impl IndexTransform for GeoGridSpace {
    #[inline]
    fn flatten_index2(&self, index2: &Self::Index2) -> Self::Index {
        let stride = self.grid_size.width() as usize;
        GeoIndex(index2.0.x as usize + index2.0.y as usize * stride)
    }
    #[inline]
    fn fold_index(&self, index: &Self::Index) -> Self::Index2 {
        let stride = self.grid_size.width() as usize;
        let x = index.0 % stride;
        let y = index.0 / stride;
        GeoIndex2(UVec2::new(x as u32, y as u32))
    }
}

/// finds the maximum size within bounds which maintains the aspect ratio.
#[inline]
fn max_size_with_aspect(original: UVec2, target: UVec2) -> UVec2 {
    let v = original * target.yx();
    if v.x > v.y {
        UVec2::new(target.x, v.y / original.x)
    } else {
        UVec2::new(v.x / original.y, target.y)
    }
}
#[inline]
fn max_size_with_aspect_d(original: DVec2, target: DVec2) -> DVec2 {
    let v = original * target.yx();
    if v.x > v.y {
        DVec2::new(target.x, v.y / original.x)
    } else {
        DVec2::new(v.x / original.y, target.y)
    }
}

#[inline]
pub fn flatten_index2(index2: UVec2, width: u32) -> usize {
    index2.x as usize + width as usize * index2.y as usize
}
#[inline]
pub fn flatten_index2_xy(x: u32, y: u32, width: u32) -> usize {
    x as usize + width as usize * y as usize
}
