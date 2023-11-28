use bevy::app::App;
use bevy::ecs::reflect::ReflectResource;
use bevy::ecs::schedule::{IntoSystemConfigs, Schedule, ScheduleLabel};
use bevy::ecs::system::{ResMut, Resource};
use bevy::ecs::world::World;
use bevy::prelude::{Res, Update};
use bevy::reflect::Reflect;
use bevy::time::{Time, Virtual};
use bevy::utils::tracing::field::display;
use bevy::utils::{Duration, Instant};
use bevy_egui::{egui, EguiContexts};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::update_set::UpdateSet;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SimUpdate;

#[derive(Reflect, Resource, Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[reflect(Resource)]
pub struct SimulatedTime {
    // sim time
    elapsed_sec: u64,
    sec_this_update: u32,

    // ticks
    elapsed_ticks: u64,
    ticks_this_update: u32,

    // config
    sec_per_tick: u32,
    target_ticks_per_update: u32,
    target_min_fps: u32,
    target_min_frametime: Duration,

    // real time
    // the instant the simulation ended last update.
    real_last_end_sim: Instant,
    /// the delta between the last two app ticks,
    /// excluding the time spent running the simulation.
    /// This will be used with the target fps to calculate the allowed simulation time.
    real_last_app_delta: Duration,
    // the instant the simulation began this update.
    real_begin_sim: Instant,
    // the duration the simulation took last update.
    real_last_sim_time: Duration,
    // accumulate the time spent running the simulation this update.
    real_sim_time_accum: Duration,
    real_sim_allowed_time: Duration,

    temp_instant: Instant,
}

impl SimulatedTime {
    pub fn elapsed_sec(&self) -> u64 {
        self.elapsed_sec
    }
    pub fn elapsed_days(&self) -> f64 {
        days_from_sec(self.elapsed_sec)
    }

    fn sim_begin(&mut self) {
        self.temp_instant = Instant::now();
        self.real_begin_sim = self.temp_instant;
        self.real_last_app_delta = self.temp_instant - self.real_last_end_sim;
        self.real_sim_allowed_time = self
            .target_min_frametime
            .checked_sub(self.real_last_app_delta)
            .unwrap_or(Duration::ZERO);

        // resets
        self.ticks_this_update = 0;
        self.sec_this_update = 0;
        self.real_sim_time_accum = Duration::ZERO;
    }
    fn sim_end(&mut self) {
        self.real_last_end_sim = self.temp_instant;
        // self.real_last_sim_time = self.temp_instant - self.real_begin_sim;
    }
    /// returns true if next tick should run
    fn tick_end(&mut self) -> bool {
        let instant = Instant::now();
        let tick_time = instant - self.temp_instant;

        // sim time
        self.elapsed_ticks += 1;
        self.elapsed_sec += self.sec_per_tick as u64;
        self.sec_this_update += self.sec_per_tick;
        self.ticks_this_update += 1;

        // real time
        self.temp_instant = instant;
        self.real_sim_time_accum += tick_time;

        self.ticks_this_update < self.target_ticks_per_update
            && self.real_sim_time_accum < self.real_sim_allowed_time
    }
}

impl Default for SimulatedTime {
    fn default() -> Self {
        let instant = Instant::now();
        Self {
            elapsed_sec: 0,
            sec_this_update: 0,
            elapsed_ticks: 0,
            ticks_this_update: 0,
            sec_per_tick: sec_from_days(1) as u32,
            target_ticks_per_update: 1,
            target_min_fps: 30,
            target_min_frametime: Duration::from_secs_f64(1.0 / 30.0),
            real_last_end_sim: instant,
            real_last_app_delta: Duration::ZERO,
            real_begin_sim: instant,
            real_last_sim_time: Duration::ZERO,
            real_sim_time_accum: Duration::ZERO,
            real_sim_allowed_time: Duration::ZERO,
            temp_instant: instant,
        }
    }
}

pub struct SimTimePlugin;
impl bevy::prelude::Plugin for SimTimePlugin {
    fn build(&self, app: &mut App) {
        let sim_update = Schedule::new(SimUpdate);
        app.add_schedule(sim_update)
            .init_resource::<SimulatedTime>()
            .register_type::<SimulatedTime>()
            .add_systems(
                bevy::prelude::Update,
                run_sim_update_schedule.in_set(UpdateSet::Sim),
            )
            .add_systems(Update, display_sim_update.in_set(UpdateSet::Egui))
            .add_systems(SimUpdate, do_something_expensive);
    }
}

pub fn run_sim_update_schedule(world: &mut World) {
    world.resource_mut::<SimulatedTime>().sim_begin();
    world.schedule_scope(SimUpdate, |world, schedule| loop {
        // guarantee at-least one tick per update
        // TODO: when to flush command buffers? currently only at the end of Update.
        schedule.run(world);
        if !world.resource_mut::<SimulatedTime>().tick_end() {
            break;
        }
    });

    world.resource_mut::<SimulatedTime>().sim_end();
}

fn do_something_expensive(mut time: ResMut<SimulatedTime>) {
    let mut x = 0;
    for _ in 0..10000000 {
        x += 1;
        time.real_last_sim_time = Duration::from_secs(x);
    }
}

pub fn display_sim_update(
    mut ectx: EguiContexts,
    mut time: ResMut<SimulatedTime>,
    time_virt: bevy::prelude::Res<Time>,
) {
    let mut target_ticks_per_update = time.target_ticks_per_update;
    let mut sec_per_tick = time.sec_per_tick;
    let mut target_min_fps = time.target_min_fps;

    egui::Window::new("Sim Time").show(ectx.ctx_mut(), |ui| {
        ui.add(
            egui::Slider::new(&mut target_ticks_per_update, 1..=100)
                .integer()
                .text("Ticks per update"),
        );
        ui.add(
            egui::Slider::new(&mut sec_per_tick, 1..=(60 * 60 * 24))
                .integer()
                .text("Seconds per tick"),
        );
        ui.add(
            egui::Slider::new(&mut target_min_fps, 1..=60)
                .integer()
                .text("Target min FPS"),
        );
        ui.label(format!("Ticks ran this update: {}", time.ticks_this_update));
        ui.label(format!("Elapsed years: {}", time.elapsed_days() / 365.0));
        ui.label(format!("FPS: {}", 1.0 / time_virt.delta_seconds_f64()))
    });

    time.target_ticks_per_update = target_ticks_per_update;
    time.sec_per_tick = sec_per_tick;
    time.target_min_fps = target_min_fps;
    time.target_min_frametime = Duration::from_secs_f64(1.0 / target_min_fps as f64);
}

fn duration_rem(dividend: Duration, divisor: Duration) -> Duration {
    // `Duration` does not have a built-in modulo operation
    let quotient = (dividend.as_nanos() / divisor.as_nanos()) as u32;
    dividend - (quotient * divisor)
}

fn sec_from_days(days: u32) -> u64 {
    days as u64 * 24 * 60 * 60
}
fn days_from_sec(sec: u64) -> f64 {
    sec as f64 / (24.0 * 60.0 * 60.0)
}
