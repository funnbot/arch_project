use bevy::math::{Affine2, DAffine2, DVec2, IVec2, UVec2};
use bevy::reflect::{Reflect, TypePath};

/// GIS Coordinate
pub struct Coord(pub DVec2);

#[derive(Reflect, Debug, Clone, Copy, Default)]
pub struct DRect {
    min: DVec2,
    max: DVec2,
}
impl DRect {
    pub fn new(a: DVec2, b: DVec2) -> Self {
        Self {
            min: a.min(b),
            max: a.max(b),
        }
    }
    pub fn union(&self, other: &Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }
    pub fn min(&self) -> &DVec2 {
        &self.min
    }
    pub fn max(&self) -> &DVec2 {
        &self.max
    }
    pub fn width(&self) -> f64 {
        self.max.x - self.min.x
    }
    pub fn height(&self) -> f64 {
        self.max.y - self.min.y
    }
    pub fn size(&self) -> DVec2 {
        self.max - self.min
    }
    pub fn center(&self) -> DVec2 {
        (self.min + self.max) / 2.0
    }
    /// expand outwards by delta in all directions
    pub fn expand(&self, delta: DVec2) -> DRect {
        DRect::new(self.min - delta, self.max + delta)
    }
    /// move entire rect by delta, size stays the same
    pub fn translate(&self, delta: DVec2) -> DRect {
        DRect {
            min: self.min + delta,
            max: self.max + delta,
        }
    }
}

pub struct CoordinateTransform {
    gis: DRect,
    sim: DRect,
    gis_center: DVec2,
    sim_center: DVec2,
    scale: DVec2,
}

pub fn coordinate_transform(from: DRect, to: DRect, equalize_scales: bool) -> DAffine2 {
    let mut scale = DVec2::new(to.width() / from.width(), to.height() / from.height());
    if equalize_scales {
        scale = DVec2::splat(scale.min_element());
    }
    let translate = to.center() - from.center();
    DAffine2::from_scale_angle_translation(scale, 0.0, translate)
}

impl CoordinateTransform {
    pub fn new(gis: DRect, mut sim: DRect, equalize_scales: bool) -> Self {
        // why do we need to expand the sim rect?
        // the sim rect is calculated with patch coordinates, which are the center of the patch.
        // so the actual edges of the sim rect are 0.5 patch units away from the center.
        // TODO: this will be set by `Map`.
        sim = sim.expand(DVec2::splat(0.5));
        let mut scale = DVec2::new(sim.width() / gis.width(), sim.height() / gis.height());
        if equalize_scales {
            scale = DVec2::splat(scale.min_element());
        }

        CoordinateTransform {
            gis_center: gis.center(),
            sim_center: sim.center(),
            gis,
            sim,
            scale,
        }
    }
    pub fn gis_to_sim(&self, coord: DVec2) -> DVec2 {
        (coord - self.gis_center) * self.scale + self.sim_center
    }
    pub fn sim_to_gis(&self, coord: DVec2) -> DVec2 {
        (coord - self.sim_center) / self.scale + self.gis_center
    }
}

/// describes a grid area in GIS coordinates.
/// useful for mapping a raster to a world grid.
#[derive(Reflect, Debug, Clone, Default)]
pub struct GridDimensions {
    /// grid size in cells
    size: UVec2,
    /// area the grid takes up
    rect: DRect,
    // might not need to keep this
    cell_size: DVec2,
}

impl GridDimensions {
    pub fn from_cell_size(grid_size: UVec2, cell_size: DVec2, ll_corner: DVec2) -> Self {
        assert!(grid_size != UVec2::ZERO);
        let rect = DRect::new(ll_corner, ll_corner + (grid_size.as_dvec2() * cell_size));
        Self {
            size: grid_size,
            rect,
            cell_size,
        }
    }
    pub fn from_rect(grid_size: UVec2, rect: DRect) -> Self {
        assert!(grid_size != UVec2::ZERO);
        let cell_size = DVec2::new(
            rect.width() / grid_size.x as f64,
            rect.height() / grid_size.y as f64,
        );
        Self {
            size: grid_size,
            rect,
            cell_size,
        }
    }
    pub fn cell_count(&self) -> u32 {
        self.size.x * self.size.y
    }
    pub fn gis_to_grid(&self, coord: DVec2) -> DVec2 {
        let grid = (coord - *self.rect.min()) / self.cell_size;
        grid
    }
    pub fn grid_to_gis(&self, coord: DVec2) -> DVec2 {
        *self.rect.min() + coord * self.cell_size
    }
    pub fn rect(&self) -> &DRect {
        &self.rect
    }
    pub fn grid_size(&self) -> &UVec2 {
        &self.size
    }
    pub fn grid_width(&self) -> u32 {
        self.size.x
    }
    pub fn grid_height(&self) -> u32 {
        self.size.y
    }
}

fn vec_inside_size(vec: &DVec2, size: &UVec2) -> bool {
    vec.x >= 0. && vec.y >= 0. && vec.x <= size.x as f64 && vec.y <= size.y as f64
}
