pub struct TemplateApp {
    running: bool,
    num_workers: u8,
    reduce_jobs: u8,
    chunk_size: u32,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            running: false,
            num_workers: 1,
            reduce_jobs: 2,
            chunk_size: 128,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    // fn save(&mut self, storage: &mut dyn eframe::Storage) {
    //     eframe::set_value(storage, eframe::APP_KEY, self);
    // }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                egui::widgets::global_dark_light_mode_switch(ui);

                ui.separator();

                #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
                {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            _frame.close();
                        }
                    });
                }
            });
        });

        egui::SidePanel::right("sidebar").show(ctx, |ui| {
            ui.heading("Controls");

            ui.separator();

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.add_space(2.5);
                    ui.label("Number of workers: ");
                    ui.add_space(13.5);
                    ui.label("Reduce jobs: ");
                    ui.add_space(13.5);
                    ui.label("Chunk size: ");
                });

                ui.vertical(|ui| {
                    ui.add(egui::Slider::new(&mut self.num_workers, 1..=12).text("value"));
                    ui.add_space(10.0);
                    ui.add(egui::Slider::new(&mut self.reduce_jobs, 2..=32).text("value"));
                    ui.add_space(10.0);
                    ui.add(egui::Slider::new(&mut self.chunk_size, 32..=204800).text("value"))
                });
            });

            ui.add_space(10.0);

            ui.separator();

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                let mut button_label = "Run";
                if self.running {
                    button_label = "Stop";
                }
                let button = egui::Button::new(button_label).min_size(egui::vec2(300.0, 30.0));
                if ui.add(button).clicked() {
                    self.running = !self.running;
                };
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                ui.separator();
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
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
}
