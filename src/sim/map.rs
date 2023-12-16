use std::ops::RangeInclusive;

use bevy::{
    ecs::schedule::OnEnter,
    math::{DVec3, IVec2},
    render::{
        color::Color,
        texture::{Image, ImageSampler},
    },
    sprite::{Sprite, SpriteBundle},
    transform::components::Transform,
};
use bevy_egui::egui::{lerp, remap};
use rand::distributions::Uniform;

use super::{cell::Cell, cell_grid::CellGrid, n8};
use crate::{
    ascii_grid::AsciiGrid,
    math::{
        coord::{
            flatten_index2, GeoGridSpace, GridTransform, IndexTransform, MapIndex2, MapSpace,
            Transformer,
        },
        DRect, ZCoord, ZIndex,
    },
    preludes::{bevy::*, core::*},
    MyAssets, MyStates, MyUpdate,
};

#[derive(Resource, reflect::Reflect, Debug, Default)]
#[reflect(Resource)]
pub struct Map {
    pub grid: CellGrid,
    space: MapSpace,
    modified: bool,
    land_cell_count: u32,
}

impl Map {
    pub fn space(&self) -> &MapSpace {
        &self.space
    }
    fn set_cell_layout(&mut self, size: UVec2, view_rect: DRect) {
        self.grid = CellGrid::new(size.into());
        self.space = MapSpace::from_rect(view_rect, size.into());
        assert!(size.x as usize * size.y as usize <= zorder_data_len(size));
    }
    /// iterate all cells in the map, and sample the ascii grid data at each cell
    fn load_ascii_grid<F: FnMut(&mut Cell, Option<i16>)>(&mut self, asset: &AsciiGrid, mut f: F) {
        let transform = Transformer::<MapSpace, GeoGridSpace>::new(&self.space, asset.space());

        for cell in self.grid.iter_mut() {
            let cell_coord = MapIndex2(cell.coord.into());
            let map_coord = self.space.ll_from_grid(&cell_coord);
            let geo_coord = transform.transform(&map_coord);
            let sample = asset.sample_nearest(&geo_coord);
            f(cell, sample);
        }
    }
    fn load_color(&mut self, image: &Image) {
        let image_space = GeoGridSpace::from_max_size(
            self.space.rect().size(),
            image.size().into(),
            self.space.rect().min,
        );
        let transform = Transformer::<MapSpace, GeoGridSpace>::new(&self.space, &image_space);
        for cell in self.grid.iter_all_mut() {
            let cell_coord = MapIndex2(cell.coord.into());
            if !self.space.grid_size().bounds_check(cell_coord.0) {
                cell.color = Color::RED;
                continue;
            }
            let map_coord = self.space.ll_from_grid(&cell_coord);
            let geo_coord = transform.transform(&map_coord);
            if !image_space.rect().contains(geo_coord.0) {
                cell.color = Color::RED;
                continue;
            }
            let index2 = image_space.to_grid_trunc(&geo_coord);
            let index = image_space.flatten_index2(&index2).0 * 4;
            let Some(pixel) = image.data.get(index..index + 4) else {
                cell.color = Color::RED;
                continue;
            };
            let color = Color::rgba_u8(pixel[0], pixel[1], pixel[2], pixel[3]);
            cell.color = color;
        }
    }
    fn visualize_cell_field<F: Fn(&Cell) -> Color>(&self, image: &mut Image, f: F) {
        let only_land: bool = true;
        let mut data = std::mem::take(&mut image.data);
        let size = self.space.grid_size();
        let size = (size.0.x as usize, size.0.y as usize);
        let new_len = size.0 * size.1 * 4;
        data.resize(new_len, 0);

        for cell in self.grid.iter() {
            let cell_coord: UVec2 = cell.coord.into();
            // 4 bytes per coord
            let pixel_idx = (cell_coord.x as usize + cell_coord.y as usize * size.0) * 4;
            let color = if !only_land || cell.is_land {
                f(cell)
            } else {
                cell.color
            };
            let color = color.as_rgba_u8();
            data[pixel_idx] = color[0];
            data[pixel_idx + 1] = color[1];
            data[pixel_idx + 2] = color[2];
            data[pixel_idx + 3] = color[3];
        }
        *image = Image::new(
            Extent3d {
                width: size.0 as u32,
                height: size.1 as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            TextureFormat::Rgba8UnormSrgb,
        );
    }
    /// storage is a temporary vector that will be cleared
    ///
    /// for each cell
    /// count the valid neighbor8 cells (not out of bounds), and passes that and the current cell to the CalcFn.
    /// CalcFn returns the Field value which will be subtracted from the current cell, and applied to the neighbors.
    /// the ApplyFn takes a cell, and applies the newly calculated field value from the temp storage vec.
    pub fn diffuse8<
        Field: Clone + Default + std::ops::AddAssign,
        CalcFn: Fn(&mut Cell, u8) -> Field,
        ApplyFn: Fn(&mut Cell, &Field),
    >(
        &mut self,
        storage: &mut Vec<Field>,
        calc_fn: CalcFn,
        apply_fn: ApplyFn,
    ) {
        let grid_size = *self.space.grid_size();
        let mut n8_indices: [isize; 8];

        storage.clear();
        storage.resize(self.grid.len(), default());
        for cell in self.grid.iter_mut() {
            let coord = MapIndex2(cell.coord.into());

            n8_indices = n8::zorder_n8_indices(coord.0, grid_size);
            let n_count = n8::count_non_negative(n8_indices) as u8;

            let val = calc_fn(cell, n_count);
            let storage_ns =
                unsafe { n8::slice_get_many_unchecked_mut(storage.as_mut_slice(), n8_indices) };
            #[allow(clippy::manual_flatten)]
            for store in storage_ns {
                if let Some(store) = store {
                    *store += val.clone();
                }
            }
        }
        for (zidx, cell) in self.grid.iter_mut().enumerate() {
            apply_fn(cell, &storage[zidx]);
        }
    }
    /// kernel - row major (indexed [y][x])
    pub fn convolve33<GetFn: Fn(&Cell) -> f64>(
        &self,
        output: &mut Vec<f64>,
        kernel: [[f64; 3]; 3],
        get_fn: GetFn,
    ) {
        let grid_size = *self.space.grid_size();
        let n8_offsets = n8::zorder_n8_offsets();

        output.clear();
        let out_len = grid_size.total_size();
        if output.capacity() < out_len {
            output.reserve_exact(out_len - output.capacity());
        }
        let out = output.spare_capacity_mut();
        for (cell, cell_ns) in self.grid.iter_n8() {
            let coord = MapIndex2(cell.coord.into());

            let mut sum: f64 = get_fn(cell) * kernel[1][1];
            for (i, c) in cell_ns.iter().enumerate() {
                if let Some(c) = *c {
                    let k_index2 = n8_offsets[i] + IVec2::ONE;
                    let kernel_v = kernel[k_index2.y as usize][k_index2.x as usize];
                    if kernel_v != 0.0 {
                        let mul_v = get_fn(c) * kernel_v;
                        sum += mul_v;
                    }
                }
            }
            let grid_index = coord.0.x as usize + grid_size.width() as usize * coord.0.y as usize;
            out[grid_index].write(sum);
        }
        unsafe {
            output.set_len(out_len);
        }
    }
    pub fn sum_cell_field<T: Default + std::ops::AddAssign, F: Fn(&Cell) -> T>(&self, f: F) -> T {
        let mut sum = T::default();
        for cell in self.grid.iter() {
            sum += f(cell);
        }
        sum
    }
    pub fn calculate_slope(&mut self) {
        let mut horiz_grad = Vec::new();
        let mut vert_grad = Vec::new();
        // Sobel operator
        // computes approximate derivative
        self.convolve33(
            &mut horiz_grad,
            [[1., 0., -1.], [2., 0., -2.], [1., 0., -1.]],
            |c| c.elevation as f64,
        );
        self.convolve33(
            &mut vert_grad,
            [[1., 2., 1.], [0., 0., 0.], [-1., -2., -1.]],
            |c| c.elevation as f64,
        );
        let grid_size = *self.space.grid_size();
        for cell in self.grid.iter_mut() {
            let index2: UVec2 = cell.coord.into();
            let idx = flatten_index2(index2, grid_size.width());
            let grad_mag = DVec2::new(horiz_grad[idx], vert_grad[idx]).length();
            let grad_mag = (grad_mag + 1.0).log2();
            cell.slope = grad_mag as f32;
        }
        for y in 0..grid_size.height() as usize {
            for x in 0..grid_size.width() as usize {
                let idx = x + grid_size.width() as usize * y;
                let gx = horiz_grad[idx];
                let gy = vert_grad[idx];

                horiz_grad[idx] = DVec2::new(gx, gy).length();
            }
        }
    }
    fn setup_land_cells(&mut self) {
        for cell in self.grid.iter_mut().filter(|c| c.is_land) {
            todo!();
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct MapView;

fn apply_geo_data_to_map(
    map: &mut Map,
    my_assets: &MyAssets,
    ascii_assets: &Assets<AsciiGrid>,
    image_assets: &Assets<Image>,
) {
    map.set_cell_layout(
        UVec2::new(401, 433),
        DRect::from_corners((-200., -216.).into(), (200., 216.).into()),
    );

    let background = image_assets.get(my_assets.background.id()).unwrap();
    map.load_color(background);

    let mut land_cell_count: u32 = 0;
    let elevation = ascii_assets.get(my_assets.elevation.id()).unwrap();
    map.load_ascii_grid(elevation, |cell, sample| {
        cell.elevation = sample.unwrap_or(0) as i32;
        if sample.is_some() {
            cell.is_land = true;
            land_cell_count += 1;
        }
    });
    map.land_cell_count = land_cell_count;

    let temperature = ascii_assets.get(my_assets.temperature.id()).unwrap();
    map.load_ascii_grid(temperature, |cell, sample| {
        cell.temperature = sample.map_or(21.0, |x| x as f32 / 12.0);
    });

    let precip = ascii_assets.get(my_assets.precipitation.id()).unwrap();
    map.load_ascii_grid(precip, |cell, sample| {
        cell.original_rainfall = sample.unwrap_or(1200) as f32;
        if cell.is_land {
            cell.rainfall = cell.original_rainfall;
        }
    });

    let soil_prod = ascii_assets.get(my_assets.soil_productivity.id()).unwrap();
    map.load_ascii_grid(soil_prod, |cell, sample| {
        cell.soil_prod = sample.map_or(1.5, f32::from);
        if cell.elevation <= 1 || cell.soil_prod >= 6.0 {
            cell.soil_prod = 1.5;
        }
    });

    let mut temp = Vec::new();
    let diffuse_iters = (map.space.rect().max.x / 20.0) as usize;
    for _ in 0..diffuse_iters {
        map.diffuse8(
            &mut temp,
            |cell, ncount| {
                let value = cell.soil_prod * 0.9;
                let share = value / 8.0;
                cell.soil_prod -= share * ncount as f32;
                share
            },
            |cell, value| cell.soil_prod += *value,
        );
    }

    map.calculate_slope();
}

fn on_running(
    mut commands: Commands,
    ascii_assets: Res<Assets<AsciiGrid>>,
    mut image_assets: ResMut<Assets<Image>>,
    my_assets: Res<MyAssets>,
    mut map: ResMut<Map>,
) {
    apply_geo_data_to_map(
        map.as_mut(),
        my_assets.as_ref(),
        ascii_assets.as_ref(),
        image_assets.as_ref(),
    );

    let mut image = Image::default();

    map.visualize_cell_field(&mut image, |cell| {
        color_lerp_lightness(
            Color::GRAY,
            0.2,
            remap(cell.slope as f32, 0.0..=15.0, 0.0..=1.0),
        )
    });

    image.sampler = ImageSampler::nearest();
    let image_handle = image_assets.add(image);
    //map.modified = true;
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(map.space.rect().min.as_vec2().extend(-1.0)),
            texture: image_handle,
            sprite: Sprite {
                custom_size: Some(map.space.rect().size().as_vec2()),
                ..default()
            },
            ..default()
        },
        MapView,
    ));
}

fn color_lerp(from: Color, to: Color, t: f32) -> Color {
    let from = from.as_linear_rgba_f32();
    let to = to.as_linear_rgba_f32();
    Color::rgba_linear(
        lerp(from[0]..=to[0], t),
        lerp(from[1]..=to[1], t),
        lerp(from[2]..=to[2], t),
        lerp(from[3]..=to[3], t),
    )
}
/*
 Reports a shade of color proportional to the value of number.

When range1 is less than or equal to range2, then the larger the number, the lighter the shade of color. However, if range2 is less than range1, the color scaling is inverted.

Let min-range be the minimum of range1 and range2. If number is less than or equal to min-range, then the result is the same as if number was equal to min-range. Let max-range be the maximum of range1 and range2. If number is greater than max-range, then the result is the same as if number was equal to max-range.

Note: for color shade is irrelevant, e.g. green and green + 2 are equivalent, and the same spectrum of colors will be used.
*/
fn color_lerp_lightness(around: Color, max: f32, t: f32) -> Color {
    let around = around.as_hsla();
    let lightness = around.l();
    let range = (lightness + max).min(1.0)..=(lightness - max).max(0.0);
    around.with_l(lerp(range, t))
}

fn zorder_data_len(grid_size: UVec2) -> usize {
    (grid_size.max_element().next_power_of_two() as usize).pow(2)
}

pub struct MapPlugin;
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Map>()
            .init_resource::<Map>()
            .add_systems(OnEnter(MyStates::Running), on_running);
    }
}
