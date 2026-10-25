//! Notepad++ clean-room reimplementation in Rust.
#![allow(clippy::collapsible_if)]

pub mod app;
pub mod buffer;
pub mod cli;
pub mod editor;
pub mod plugins;
pub mod search;
pub mod syntax;
pub mod ui;

use app::NotepadApp;
use cli::CliArgs;
use eframe::egui;

fn main() -> eframe::Result<()> {
    eprintln!("[1] Parsing CLI args...");
    let cli = CliArgs::parse();

    eprintln!("[2] Setting up viewport...");
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1180.0, 760.0])
        .with_min_inner_size([600.0, 400.0])
        .with_title("Notepad++ (Rust)")
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

    eprintln!("[3] Calling eframe::run_native...");
    eframe::run_native(
        "Notepad++",
        options,
        Box::new(move |cc| {
            eprintln!("[4] Inside app creation closure!");
            let app = NotepadApp::new(cc, cli);
            eprintln!("[5] App created successfully!");
            Ok(Box::new(app))
        }),
    )
}
