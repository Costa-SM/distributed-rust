mod app;

use app::MapReduceApp;

impl eframe::App for MapReduceApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        render_header(ctx, frame);
        self.render_controls(ctx);
        egui::CentralPanel::default().show(ctx, |ui: &mut egui::Ui| {
            self.render_nodes(ui);
            render_footer(ui);
        });
    }
}

fn render_header(ctx: &egui::Context, frame: &mut eframe::Frame) {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            egui::widgets::global_dark_light_mode_switch(ui);
            ui.add_space(4.0);
            ui.separator();
            {
                ui.menu_button("File", |ui: &mut egui::Ui| {
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
            }
        });
    });
}

fn render_footer(ui: &mut egui::Ui) {
    ui.with_layout(
        egui::Layout::bottom_up(egui::Align::LEFT),
        |ui: &mut egui::Ui| {
            ui.horizontal(|ui: &mut egui::Ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("See source code: ");
                ui.hyperlink_to(
                    "distributed-rust",
                    "https://github.com/Costa-SM/distributed-rust",
                );
                ui.label(". Powered by ");
                ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                ui.label(" and ");
                ui.hyperlink_to(
                    "eframe",
                    "https://github.com/emilk/egui/tree/master/crates/eframe",
                );
                ui.label(".");
            });
            ui.separator();
        },
    );
}

fn main() -> eframe::Result<()> {
    let mut window_options: eframe::NativeOptions = eframe::NativeOptions::default();
    window_options.initial_window_size = Some(egui::Vec2::new(1440.0, 760.0));
    eframe::run_native(
        "MapReduce GUI",
        window_options,
        Box::new(|cc: &eframe::CreationContext<'_>| Box::new(MapReduceApp::new(cc))),
    )
}
