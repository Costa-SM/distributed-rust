use egui::emath::Numeric;

pub struct Worker {
    id: u8,
    title: String,
    position: egui::Pos2,
    working: bool,
}

// enum NodeType {
//     Map,
//     Reduce,
// }

// pub struct Node {
//     node_type: NodeType,
//     title: String,
//     position: egui::Pos2,
// }

pub struct MapReduceApp {
    running: bool,
    num_workers: u8,
    reduce_jobs: u8,
    chunk_size: u32,
    nodes: Vec<Worker>,
}

impl MapReduceApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
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
            ui.horizontal(|ui: &mut egui::Ui| {
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
            });
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);
            ui.horizontal(|ui: &mut egui::Ui| {
                let button_label: &str = if !self.running { "Run" } else { "Stop" };
                let disabled: bool = if !self.running && self.num_workers == 0 {
                    true
                } else {
                    false
                };
                let button_font_color: egui::Color32;
                let button_cursor_icon: egui::CursorIcon;
                let button_background_color: egui::Color32;
                if disabled {
                    button_background_color = egui::Color32::from_rgb(192, 128, 128);
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
                };
            });
        });
    }

    pub fn render_workers(&mut self, ui: &mut eframe::egui::Ui) {
        for node in self.nodes.iter_mut() {
            Self::render_worker(node, ui);
        }
    }

    fn render_worker(node: &mut Worker, ui: &mut egui::Ui) {
        let node_id = ui.make_persistent_id(node.id);

        let _node_window = egui::Area::new(node_id)
            .default_pos(node.position.clone())
            .movable(false)
            .show(ui.ctx(), |ui: &mut egui::Ui| {
                egui::Frame::none()
                    .stroke(egui::Stroke::new(2.0, egui::Color32::DARK_GRAY))
                    .inner_margin(egui::Margin::same(10.0))
                    .rounding(5.0)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                let color = if node.working {
                                    egui::Color32::DARK_GREEN
                                } else {
                                    egui::Color32::DARK_RED
                                };
                                ui.label(node.title.clone());
                                ui.add_space(20.0);
                                egui::Frame::none()
                                    .fill(color)
                                    .rounding(9999.0)
                                    .inner_margin(egui::Margin::same(5.0))
                                    .show(ui, |_ui| {});
                            });
                            let button_label = if node.working {
                                "Break worker"
                            } else {
                                "Fix worker"
                            };
                            let button: egui::Button<'_> =
                                egui::Button::new(button_label).min_size(egui::vec2(90.0, 30.0));
                            if ui.add(button).clicked() {
                                node.working = !(node.working.clone());
                                ui.ctx().request_repaint()
                            };
                        });
                    });
            });
    }
}
