use egui::emath::Numeric;

use crate::worker::Worker;

pub struct MapReduceApp {
    opened_file: Option<std::path::PathBuf>,
    open_file_dialog: Option<egui_file::FileDialog>,
    running: bool,
    num_workers: u8,
    reduce_jobs: u8,
    chunk_size: u32,
    nodes: Vec<Worker>,
}

impl MapReduceApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            opened_file: None,
            open_file_dialog: None,
            running: false,
            num_workers: 0,
            reduce_jobs: 2,
            chunk_size: 128,
            nodes: Vec::new(),
        }
    }

    pub fn render_controls(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("sidebar").show(ctx, |ui: &mut egui::Ui| {
            ui.heading("Controls");
            ui.separator();
            ui.horizontal(|ui| {
                self.render_file_button(ctx, ui);
            });
            ui.add_space(5.0);
            ui.horizontal(|ui: &mut egui::Ui| {
                self.render_sliders(ui);
            });
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);
            ui.horizontal(|ui: &mut egui::Ui| self.render_run_button(ui));
        });
    }

    pub fn render_workers(&mut self, ui: &mut eframe::egui::Ui) {
        for node in self.nodes.iter_mut() {
            node.render_worker(ui);
        }
    }

    fn render_file_button(self: &mut Self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let path_text: String = self.opened_file.as_ref().map_or_else(
            || String::default(),
            |path: &std::path::PathBuf| path.to_string_lossy().to_string(),
        );
        let path_parts: Vec<&str> = path_text.split('/').collect();
        let file_name: String = path_parts.last().unwrap().to_string();
        let button_label: &str = if self.opened_file.is_none() {
            "Select File"
        } else {
            file_name.as_str()
        };
        let button: egui::Button<'_> =
            egui::Button::new(egui::RichText::new(button_label)).min_size(egui::vec2(275.0, 30.0));
        if (ui.add(button))
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
            && !self.running
        {
            let mut dialog: egui_file::FileDialog =
                egui_file::FileDialog::open_file(self.opened_file.clone());
            dialog.open();
            self.open_file_dialog = Some(dialog);
        }
        if let Some(dialog) = &mut self.open_file_dialog {
            if dialog.show(ctx).selected() {
                if let Some(file) = dialog.path() {
                    self.opened_file = Some(file.to_path_buf());
                }
            }
        }
    }

    fn render_sliders(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui: &mut egui::Ui| {
            ui.add_space(2.5);
            ui.label("Number of workers: ");
            ui.add_space(13.5);
            ui.label("Reduce jobs: ");
            ui.add_space(13.5);
            ui.label("Chunk size: ");
        });
        ui.vertical(|ui: &mut egui::Ui| {
            if !self.running {
                if ui
                    .add(egui::Slider::new(&mut self.num_workers, 0..=8))
                    .changed()
                {
                    self.nodes.clear();
                    for i in 0..self.num_workers {
                        let node_x_pos: f64 = 50.0 + i.to_f64() * 135.0;
                        self.nodes.push(Worker {
                            id: i + 1,
                            title: format!("Worker {}", i + 1),
                            position: egui::Pos2::new(node_x_pos as f32, 50.0),
                            working: true,
                        });
                    }
                };
            } else {
                ui.add_space(2.5);
                ui.label(self.num_workers.to_string());
            }
            ui.add_space(10.0);
            if !self.running {
                if ui
                    .add(egui::Slider::new(&mut self.reduce_jobs, 2..=32))
                    .changed()
                {
                    // TODO: update reduce jobs ?
                };
            } else {
                ui.add_space(2.5);
                ui.label(self.reduce_jobs.to_string());
            }
            ui.add_space(10.0);
            if !self.running {
                if ui
                    .add(egui::Slider::new(&mut self.chunk_size, 32..=204800))
                    .changed()
                {
                    // TODO: update chunk size ?
                };
            } else {
                ui.add_space(2.5);
                ui.label(self.chunk_size.to_string());
            }
        });
    }

    fn render_run_button(self: &mut Self, ui: &mut egui::Ui) {
        let button_label: &str = if !self.running { "Run" } else { "Stop" };
        let disabled: bool =
            if !self.running && (self.num_workers == 0 || self.opened_file.is_none()) {
                true
            } else {
                false
            };
        let button_font_color: egui::Color32;
        let button_cursor_icon: egui::CursorIcon;
        let button_background_color: egui::Color32;
        if disabled {
            button_background_color = egui::Color32::from_rgb(128, 72, 72);
            button_cursor_icon = egui::CursorIcon::NotAllowed;
            button_font_color = egui::Color32::BLACK;
        } else {
            button_background_color = egui::Color32::DARK_GRAY;
            button_cursor_icon = egui::CursorIcon::PointingHand;
            button_font_color = egui::Color32::LIGHT_GRAY;
        }
        let button: egui::Button<'_> =
            egui::Button::new(egui::RichText::new(button_label).color(button_font_color))
                .min_size(egui::vec2(275.0, 30.0))
                .fill(button_background_color);
        if ui.add(button).on_hover_cursor(button_cursor_icon).clicked() && !disabled {
            self.running = !self.running;
            // TODO: Start map reduce with self.opened_file, self.num_workers, self.reduce_jobs, self.chunk_size
        };
    }
}
