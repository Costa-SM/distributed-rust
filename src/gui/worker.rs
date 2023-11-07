pub struct Worker {
    pub id: u8,
    pub title: String,
    pub position: egui::Pos2,
    pub working: bool,
}

impl Worker {
    pub fn render_worker(&mut self, ui: &mut egui::Ui) {
        let worker_id = ui.make_persistent_id(self.id);

        let _worker_window: egui::InnerResponse<()> = egui::Area::new(worker_id)
            .default_pos(self.position.clone())
            .movable(false)
            .show(ui.ctx(), |ui: &mut egui::Ui| {
                egui::Frame::none()
                    .stroke(egui::Stroke::new(2.0, egui::Color32::DARK_GRAY))
                    .inner_margin(egui::Margin::same(10.0))
                    .rounding(5.0)
                    .show(ui, |ui: &mut egui::Ui| {
                        ui.vertical(|ui: &mut egui::Ui| {
                            ui.horizontal(|ui: &mut egui::Ui| {
                                let color = if self.working {
                                    egui::Color32::DARK_GREEN
                                } else {
                                    egui::Color32::DARK_RED
                                };
                                ui.label(self.title.clone());
                                ui.add_space(20.0);
                                egui::Frame::none()
                                    .fill(color)
                                    .rounding(9999.0)
                                    .inner_margin(egui::Margin::same(5.0))
                                    .show(ui, |_ui: &mut egui::Ui| {});
                            });
                            let button_label: &str = if self.working {
                                "Break worker"
                            } else {
                                "Fix worker"
                            };
                            let button: egui::Button<'_> =
                                egui::Button::new(button_label).min_size(egui::vec2(90.0, 30.0));
                            if ui.add(button).clicked() {
                                self.working = !(self.working.clone());
                                ui.ctx().request_repaint()
                            };
                        });
                    });
            });
    }
}
