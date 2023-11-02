pub struct Node {
    id: u64,
    title: String,
    position: egui::Pos2,
    working: bool,
}

pub struct MapReduceApp {
    running: bool,
    num_workers: u8,
    reduce_jobs: u8,
    chunk_size: u32,
    nodes: Vec<Node>,
}

impl Default for MapReduceApp {
    fn default() -> Self {
        let iter = (0..1).map(|_a| Node {
            id: 0,
            title: "Node 0".to_string(),
            position: egui::Pos2::new(250.0, 250.0),
            working: true,
        });

        Self {
            running: false,
            num_workers: 1,
            reduce_jobs: 2,
            chunk_size: 128,
            nodes: Vec::from_iter(iter),
        }
    }
}

impl MapReduceApp {
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
                    ui.add(egui::Slider::new(&mut self.num_workers, 1..=12));
                    ui.add_space(10.0);
                    ui.add(egui::Slider::new(&mut self.reduce_jobs, 2..=32));
                    ui.add_space(10.0);
                    ui.add(egui::Slider::new(&mut self.chunk_size, 32..=204800))
                });
            });
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);
            ui.horizontal(|ui: &mut egui::Ui| {
                let mut button_label = "Run";
                if self.running {
                    button_label = "Stop";
                }
                let button: egui::Button<'_> =
                    egui::Button::new(button_label).min_size(egui::vec2(275.0, 30.0));
                if ui.add(button).clicked() {
                    self.running = !self.running;
                };
            });
        });
    }

    pub fn render_nodes(&mut self, ui: &mut eframe::egui::Ui) {
        for node in self.nodes.iter_mut() {
            Self::render_node(node, ui);
        }
    }

    fn render_node(node: &mut Node, ui: &mut egui::Ui) {
        let node_id = ui.make_persistent_id(node.id);

        let _node_window = egui::Area::new(node_id)
            .default_pos(node.position.clone())
            .movable(true)
            .show(ui.ctx(), |ui: &mut egui::Ui| {
                egui::Frame::none()
                    .stroke(egui::Stroke::new(2.0, egui::Color32::DARK_GRAY))
                    .inner_margin(egui::Margin::same(10.0))
                    .rounding(5.0)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                let mut color = egui::Color32::DARK_GREEN;
                                if node.working == false {
                                    color = egui::Color32::DARK_RED;
                                }
                                ui.label(node.title.clone());
                                egui::Frame::none()
                                    .fill(color)
                                    .rounding(9999.0)
                                    .inner_margin(egui::Margin::same(5.0))
                                    .show(ui, |_ui| {});
                            });
                            let button_label = if node.working {
                                "Force worker failure"
                            } else {
                                "Fix worker"
                            };
                            let button: egui::Button<'_> =
                                egui::Button::new(button_label).min_size(egui::vec2(50.0, 30.0));
                            if ui.add(button).clicked() {
                                node.working = !(node.working.clone());
                                ui.ctx().request_repaint()
                            };
                        });
                    });
            });
    }
}

// impl App for MapReduceApp {
//     // Called by the frame work to save state before shutdown.
//     // fn save(&mut self, storage: &mut dyn eframe::Storage) {
//     //     eframe::set_value(storage, eframe::APP_KEY, self);
//     // }
// }
