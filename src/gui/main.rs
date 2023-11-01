mod app;
use crate::app::MapReduceApp;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        initial_window_size: Some([1440.0, 760.0].into()),
        min_window_size: Some([1440.0, 760.0].into()),
        ..Default::default()
    };
    eframe::run_native(
        "MapReduce GUI",
        native_options,
        Box::new(|cc| Box::new(MapReduceApp::new(cc))),
    )
}
