//! rust-note — Modern, high-performance clean-room code and text editor in Rust.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::collapsible_if)]

pub mod app;
pub mod buffer;
pub mod cli;
pub mod editor;
pub mod plugins;
pub mod search;
pub mod settings;
pub mod syntax;
pub mod ui;

use app::RustNoteApp;
use cli::CliArgs;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let cli = CliArgs::parse();

    let window_title = if let Some(ref add) = cli.title_add {
        format!("rust-note - {add}")
    } else {
        "rust-note".to_string()
    };

    let app_icon = ui::icon::generate_app_icon();

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1180.0, 760.0])
        .with_min_inner_size([600.0, 400.0])
        .with_title(window_title)
        .with_icon(app_icon)
        .with_visible(true)
        .with_active(true);

    if cli.always_on_top {
        viewport = viewport.with_always_on_top();
    }

    let options = eframe::NativeOptions {
        viewport,
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    eframe::run_native(
        "rust-note",
        options,
        Box::new(move |cc| {
            let app = RustNoteApp::new(cc, cli);
            Ok(Box::new(app))
        }),
    )
}
