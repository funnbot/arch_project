use std::{mem::swap, rc::Rc};

use bevy::ecs::entity::Entity;
use bevy::{asset::Handle, math::IVec2, render::texture::Image, utils::EntityHashMap};
// grid of patches
#[derive(Clone)]
struct Patch {
    original_rainfall: f32,
    rainfall: f32,
    temp: f32,
    elevation: i32,
    soil_prod: f32, // soil productivity
    slope: f32,
    flow: f32,         // flow of water
    pop_gradient: f32, // population gradient
    env_degrade: f32,
    npp: f32,       // net primary productivity
    _yield: i32,    // crop yield
    ag_suit: f32,   // suitability for agriculture
    bca_ag: f32,    // benefit cost ratio of agriculture
    is_ag: bool,    // is agriculture
    ag_impact: f32, // impact of agriculture
    forest_state: i32,
    succession_counter: i32,
    travel_cost: f32,
    overland_to: f32,
    freshwater_to: f32,
    cropping_value: f32,
    water_value: f32,
    forest_food_value: f32,
    rain_value: f32,
    ecosystem_services_value: f32,
    is_vacant: bool,
    patch_migrant_utility: (),
    travel_cost_ut: (),
    es_ut: (), // ecosystem services utility
    my_settlement: Option<Settlement>,
    is_land_patch: bool,
    is_border: bool, // (land-patches with [ (sum [is-land-patch] of neighbors < 6) ])
}

trait Agent {
    fn id(&self) -> i32 {
        unimplemented!()
    }
    fn die(&self) {
        unimplemented!()
    }
}

#[derive(Clone)]
struct Settlement {
    birthrate: f32,
    trade_strength: f32,
    centrality: f32,
    cluster_number: i32,
    age: i32,
    population: i32,
    gdp_per_cap: f32, // gdp per capita
    trade_gdp: f32,
    yield_gdp: f32,   // crop yield gdp
    ecoserv_gdp: f32, // ecosystem services gdp
    death_rate: f32,
    out_migration: f32,
    out_migration_rate: f32,
    settlement_yield: f32,
    ecoserv_benefit: f32,
    my_ag_patches: Vec<Patch>,
    my_influence_patches: Vec<Patch>,
    rank: i32, // vertex rank
    trade_benefit: f32,
    explored: bool,
    city_travel_cost: f32,
}

impl Agent for Settlement {}

impl Settlement {
    fn links(&self) -> &mut Vec<Link> {
        unimplemented!()
    }
    fn searchers_here(&self) -> &Vec<Searcher> {
        unimplemented!()
    }
}

struct Slider<T> {
    min: T,
    max: T,
    step: T,
    dflt: T,
    value: T,
}

impl<T: Copy> Slider<T> {
    fn new(min: T, max: T, step: T, dflt: T) -> Self {
        Self {
            min,
            max,
            step,
            dflt,
            value: dflt,
        }
    }
}

struct Config {
    Nature: bool,
    Raining: bool,
    Humans: bool,
    Climate_Cycle: bool,
    Climate_Change: bool,
    Influence_view: bool,
    Agric_view: bool,
    // anthropogenic
    migration: bool,
    trade: bool,
    //   settlements
    num_cities: Slider<i32>,
    rank_1_pop: Slider<i32>,
    rank_2_pop: Slider<i32>,
    rank_3_pop: Slider<i32>,
    //   migration
    tc_pref: Slider<f32>,
    es_pref: Slider<f32>,
    //   travel cost
    flow_tc: Slider<f32>,
    slope_tc: Slider<f32>,
    //   real income
    trade_value: Slider<i32>,
    ecoserv_value: Slider<i32>,
    ag_value: Slider<f32>,
    population_control: bool,
    // biophysical
    //    climate
    rain_change: Slider<f32>,
    Veg_Rainfall: Slider<i32>,
    Climate_Var: Slider<i32>,
    //    water
    rain_steps: Slider<i32>,
    precip_percent: Slider<f32>,
    Infitration: Slider<f32>,
    //    soils
    soil_deg_rate: Slider<f32>,
    soil_regen_rate: Slider<f32>,
    //    agriculture
    estab_cost: Slider<i32>,
    ag_travel_cost: Slider<i32>,
    ag_suit_npp: Slider<f32>,
    ag_suit_slope: Slider<i32>,
    ag_suit_soils: Slider<i32>,
    ag_suit_flow: Slider<i32>,
    //    crop yield
    max_yield: Slider<i32>,
    origin_shift: Slider<f32>,
    slope_yield: Slider<f32>,
    //    forests
    disturb_rate: Slider<f32>,
    state_change_s2: Slider<i32>,
    state_change_s3: Slider<i32>,
    s3num_neigh: Slider<i32>,
    //    ecosystem services
    crop_value_param: Slider<f32>,
    forest_value_param: Slider<i32>,
    flow_value_param: Slider<i32>,
}

struct Global {
    // GIS data
    mask_dataset: Handle<Image>,
    elevation_dataset: Handle<Image>,
    soils_dataset: Handle<Image>,
    temp_dataset: Handle<Image>,
    precip_dataset: Handle<Image>,

    // Cached
    land_patches: Vec<Patch>,
    vacant_lands: Vec<Patch>,
    border: Vec<Patch>,

    traders: Vec<Settlement>,

    failed_cities: i32,
    crop1_yield: i32,
    climate_cycle_counter: i32,
    abandoned_crops: i32,
    new_crops: i32,
    total_migrant_population: f32,
    giant_component_size: i32,
    component_size: i32,
    giant_start_node: i32,
    search_completed: bool,
    origin_city: Settlement,
    area: f32,
    max: i32,
    min: i32,
    total_migrant_utility: f32,
    rainfall_change: f32,

    // temp globals for expand-search
    visited_nodes: Vec<Settlement>, // searcher exploration, track already visited nodes
    network_start: Option<Settlement>, // one-of traders with [ not explored? ] | "nobody"
}

impl Global {
    fn create_searcher(&mut self) -> Rc<Searcher> {
        unimplemented!()
    }
}

struct Raindrop {
    rain_volume: f32,
}

#[derive(Clone)]
struct Searcher {
    location: Settlement,
    path_cost: f32,
}

impl Agent for Searcher {}

// used as link between settlements only
struct Link {
    from: Settlement,
    to: Settlement,
    thickness: f32,
    trade_flow: f32,
}

struct Migrant {
    migrant_population: i32,
    mig_tc_pref: f32,
    mig_es_pref: f32,
    my_migrant_location: Settlement,
    my_pioneer_patch: Patch,
    my_migrant_utility: f32,
    parent: Settlement,
}

struct EntityGraph {
    edges: EntityHashMap<Entity, Vec<Entity>>,
}

fn swap_remove_first_of<T: PartialEq>(vec: &mut Vec<T>, value: T) {
    if let Some(i) = vec.iter().position(|x| *x == value) {
        vec.swap_remove(i);
    }
}

const EMPTY_VEC: &Vec<Entity> = &vec![];

impl EntityGraph {
    fn new() -> Self {
        unimplemented!()
    }
    fn add_link(&mut self, a: Entity, b: Entity) {
        self.edges.entry(a).or_default().push(b);
        self.edges.entry(b).or_default().push(a);
        unimplemented!()
    }
    fn remove_link(&mut self, a: Entity, b: Entity) {
        self.edges.entry(a).and_modify(|edges| {
            swap_remove_first_of(edges, b);
        });
        self.edges.entry(b).and_modify(|edges| {
            swap_remove_first_of(edges, a);
        });
    }
    fn links(&self, a: Entity) -> &Vec<Entity> {
        self.edges.get(&a).unwrap_or(EMPTY_VEC)
    }
}

// Modelling

fn find_all_components(global: &mut Global) {
    for trader in global.traders.iter() {
        global.network_start = global.traders.iter().find(|t| !t.explored).cloned();
        if global.network_start.is_none() {
            return;
        }
        global.component_size = 0;
        // explore(&mut global.network_start.unwrap(), global, 2);
    }
}

fn explore(sett: &mut Settlement, global: &mut Global, value: i32) {
    if sett.explored {
        return;
    }
    sett.explored = true;
    global.component_size += 1;
    if let Some(s) = global.network_start.as_mut() {
        s.cluster_number += 1;
    }
    for link in sett.links().iter_mut() {
        explore(&mut link.to, global, value);
    }
}

fn expand_paths(mut searcher: Rc<Searcher>, global: &mut Global) {
    // list of links from searcher.location, with a non negative link count??, sorted by idk...
    let links: Vec<Settlement> = Vec::new();
    for target in links {
        expand_path(Rc::get_mut(&mut searcher).unwrap(), &target, global);
    }
}

fn expand_path<'a>(searcher: &'a mut Searcher, target: &'a Settlement, global: &mut Global) {
    if !global.search_completed && !global.visited_nodes.iter().any(|n| n.id() == target.id()) {
        global.visited_nodes.insert(0, target.clone());
        if searcher.location.searchers_here().len() > 1 {
            searcher.die();
        } else {
            // the previous search will just sit
            // and prevent other searchers from hatching at this point
            // it will be killed after all the outlinks of searcher.location are searched
            let mut new_searcher = searcher.clone(); // create new agent
            new_searcher.location = target.clone();
            new_searcher.path_cost += new_searcher.location.city_travel_cost;
        }
    }
}

fn recalc_gdp<'a, I: Iterator<Item = &'a mut Settlement>>(iter: I, config: &Config) {
    for sett in iter {
        sett.ecoserv_benefit = mean(
            sett.my_influence_patches
                .iter()
                .map(|p| p.ecosystem_services_value),
        );
        sett.ecoserv_gdp = sett.ecoserv_benefit * config.ecoserv_value.value as f32;
        if config.trade {
            if !sett.links().is_empty() {
                sett.trade_benefit = sett.trade_strength * config.trade_value.value as f32;
                for link in sett.links().iter_mut() {
                    link.trade_flow += sett.trade_benefit;
                    link.thickness = link.trade_flow / 10000.0;
                }
            } else {
                sett.trade_benefit = 0.0;
            }
        }
        sett.trade_gdp = sett.trade_benefit;
        sett.yield_gdp =
            (sett.settlement_yield + sett.trade_gdp + sett.ecoserv_gdp) / sett.population as f32;
    }
}

fn mean<I: Iterator<Item = f32>>(iter: I) -> f32 {
    let mut sum = 0.0;
    let mut count = 0;
    for v in iter {
        sum += v;
        count += 1;
    }
    sum / count as f32
}
