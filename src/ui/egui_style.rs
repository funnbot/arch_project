use bevy::ecs::query::With;
use bevy::ecs::system::Query;
use bevy::window::PrimaryWindow;
use bevy_egui::egui::epaint::Shadow;
use bevy_egui::egui::{self, Style};
use bevy_egui::{EguiContext, EguiContexts};

fn set_style(style: &mut Style) {
    style.visuals.window_shadow = Shadow::NONE;
    style.visuals.popup_shadow = Shadow::NONE;
}

pub fn set_egui_style_system(mut ctx_q: Query<&mut EguiContext>) {
    for mut ctx in &mut ctx_q {
        ctx.get_mut().style_mut(set_style);
    }
}
