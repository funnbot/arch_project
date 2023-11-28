use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::ascii_asset::AsciiAsset;
use crate::drect::{coordinate_transform, CoordinateTransform, DRect, GridDimensions};
use crate::update_set::UpdateSet;
use crate::zcoord::{ZCoord, ZIndex};

#[derive(Resource, Reflect, Debug)]
#[reflect(Resource)]
pub struct Map {
    cells: Vec<Cell>,
    dim: GridDimensions,
    modified: bool,
    temp: Handle<AsciiAsset>,
}

impl FromWorld for Map {
    fn from_world(world: &mut World) -> Self {
        let mut map = Map {
            cells: Vec::new(),
            dim: GridDimensions::default(),
            modified: false,
            temp: Handle::default(),
        };
        map.set_cell_layout(
            UVec2::new(400, 400),
            DRect::new((-200.0, -200.0).into(), (200.0, 200.0).into()),
        );
        map
    }
}

#[derive(Component, Debug, Clone)]
pub struct MapView {
    ascii: Handle<AsciiAsset>,
    image: Handle<Image>,
}

fn startup_load_ascii(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut map: ResMut<Map>,
) {
    let asset: Handle<AsciiAsset> = asset_server.load("temp.asc");
    let image: Handle<Image> = asset_server.add(Image::default());
    map.temp = asset.clone();
    commands.spawn(MapView {
        image: image.clone(),
        ascii: asset,
    });
}

fn on_ascii_loaded(
    mut commands: Commands,
    mut events: EventReader<AssetEvent<AsciiAsset>>,
    mut map_view_q: Query<(Entity, &mut MapView)>,
    mut ascii_assets: ResMut<Assets<AsciiAsset>>,
    mut image_assets: ResMut<Assets<Image>>,
    mut map: ResMut<Map>,
) {
    for event in events.read() {
        let (entity, mut map_view) = map_view_q.single_mut();
        if event.is_loaded_with_dependencies(map.temp.id()) {
            if let Some(ascii) = ascii_assets.get_mut(map_view.ascii.id()) {
                // image_assets.insert(
                //     map_view.image.id(),
                //     ascii.create_image(TextureFormat::Rgba16Unorm),
                // );
                map.load_ascii_grid(&ascii, |cell, sample| {
                    cell.elevation = sample.unwrap_or(0) as f64;
                });
                let mut image = Image::default();
                map.visualize_temp(&mut image);
                image_assets.insert(map_view.image.id(), image);
                //map.modified = true;
                commands.entity(entity).insert(SpriteBundle {
                    transform: Transform::from_xyz(0.0, 0.0, -1.0),
                    texture: map_view.image.clone(),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(500., 500.)),
                        ..default()
                    },
                    ..default()
                });
            }
        }
    }
}

pub struct SimPlugin;
impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup_load_ascii)
            .add_systems(
                Update,
                on_ascii_loaded.run_if(on_event::<AssetEvent<AsciiAsset>>()),
            )
            .register_type::<Map>()
            .init_resource::<Map>();
    }
}

impl Map {
    fn get_cell(&self, pos: UVec2) -> &Cell {
        let zc: ZCoord = pos.into();
        &self.cells[zc]
    }
    fn get_cell_mut(&mut self, pos: UVec2) -> &mut Cell {
        let zc: ZCoord = pos.into();
        &mut self.cells[zc]
    }
    fn set_cell_layout(&mut self, size: UVec2, view_rect: DRect) {
        let len = cell_data_side_len(size).pow(2) as usize;
        self.cells = Vec::with_capacity(len);
        for i in 0..len {
            self.cells.push(Cell {
                coord: ZIndex::from(i).into(),
                ..default()
            })
        }
        self.dim = GridDimensions::from_rect(size, view_rect);
    }
    /// iterate all cells in the map, and sample the ascii grid data at each cell
    fn load_ascii_grid<F: Fn(&mut Cell, Option<i16>)>(&mut self, asset: &AsciiAsset, f: F) {
        let transform = coordinate_transform(*self.dim.rect(), asset.rect(), true);

        for cell in self.cells.iter_mut() {
            let cell_coord: UVec2 = cell.coord.into();
            let gis_coord = self.dim.grid_to_gis(cell_coord.as_dvec2());
            let sample_coord = transform.transform_point2(gis_coord);
            let sample = asset.sample_nearest(sample_coord);
            f(cell, sample);
        }
    }
    fn visualize_temp(&self, image: &mut Image) {
        let mut data = std::mem::take(&mut image.data);
        let side_len = cell_data_side_len(*self.dim.grid_size());
        let new_len = side_len * side_len * 4;
        data.resize(new_len as usize, 0);

        for cell in self.cells.iter() {
            let cell_coord: UVec2 = cell.coord.into();
            // 4 bytes per coord
            let pixel_idx = (cell_coord.x + cell_coord.y * side_len) as usize * 4;
            data[pixel_idx] = cell.elevation as u8;
            data[pixel_idx + 1] = 0;
            data[pixel_idx + 2] = 0;
            data[pixel_idx + 3] = if (cell.elevation as u8) == 0 { 0 } else { 255 };
        }
        *image = Image::new(
            Extent3d {
                width: side_len,
                height: side_len,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            TextureFormat::Rgba8Unorm,
        );
    }
    fn cell_data_side_len(&self) -> usize {
        self.dim.grid_size().max_element().next_power_of_two() as usize
    }
}

#[derive(Debug, Reflect, Default, Clone)]
pub enum CellChild {
    Settlement(Entity),
    Agriculture(Entity),
    #[default]
    Empty,
}

/// Mostly static precomputed data about a grid location
/// aka a patch
#[derive(Debug, Reflect, Default, Clone)]
struct Cell {
    coord: ZCoord,
    elevation: f64,
    child: CellChild,
}

fn vec_ensure_capacity<T>(vec: &mut Vec<T>, capacity: usize) {
    if vec.capacity() < capacity {
        vec.clear();
        vec.reserve_exact(capacity - vec.capacity());
    }
}

fn cell_data_side_len(grid_size: UVec2) -> u32 {
    grid_size.max_element().next_power_of_two()
}

fn test_sprite(size: UVec2) -> Image {
    todo!()
}
