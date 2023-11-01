#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
use crate::app::TemplateApp;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        initial_window_size: Some([1440.0, 760.0].into()),
        min_window_size: Some([1440.0, 760.0].into()),
        ..Default::default()
    };
    eframe::run_native(
        "MapReduce GUI",
        native_options,
        Box::new(|cc| Box::new(TemplateApp::new(cc))),
    )
}
