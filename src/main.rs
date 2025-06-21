mod app;
mod game;
mod handler;
mod input;
mod launch;
mod paths;
mod util;

use crate::app::*;
use crate::paths::*;
use crate::util::*;
use eframe::egui;

fn apply_steamdeck_style(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    let accent = egui::Color32::from_rgb(0, 174, 239);
    visuals.widgets.active.bg_fill = accent;
    visuals.widgets.hovered.bg_fill = accent.gamma_multiply(0.8);
    visuals.selection.bg_fill = accent;
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    ctx.set_style(style);
}

fn main() -> eframe::Result {
    std::fs::create_dir_all(PATH_PARTY.join("gamesyms"))
        .expect("Failed to create gamesyms directory");
    std::fs::create_dir_all(PATH_PARTY.join("handlers"))
        .expect("Failed to create handlers directory");
    std::fs::create_dir_all(PATH_PARTY.join("profiles"))
        .expect("Failed to create profiles directory");

    remove_guest_profiles().unwrap();

    if PATH_PARTY.join("tmp").exists() {
        std::fs::remove_dir_all(PATH_PARTY.join("tmp")).unwrap();
    }

    println!("\n[PARTYDECK] started\n");

    let fullscreen = std::env::args().any(|arg| arg == "--fullscreen");

    let (_, scrheight) = get_screen_resolution();

    let scale = match fullscreen {
        true => scrheight as f32 / 560.0,
        false => 1.3,
    };

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1080.0, 540.0])
            .with_min_inner_size([640.0, 360.0])
            .with_fullscreen(fullscreen)
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../res/icon.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "PartyDeck",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);
            cc.egui_ctx.set_zoom_factor(scale);
            apply_steamdeck_style(&cc.egui_ctx);
            Ok(Box::<PartyApp>::default())
        }),
    )
}
