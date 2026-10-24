//! Notepad++ clean-room reimplementation in Rust.

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
    let cli = CliArgs::parse();

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1180.0, 760.0])
        .with_min_inner_size([600.0, 400.0])
        .with_title("Notepad++ (Rust)");

    if cli.always_on_top {
        viewport = viewport.with_always_on_top();
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Notepad++",
        options,
        Box::new(move |cc| Ok(Box::new(NotepadApp::new(cc, cli)))),
    )
}
