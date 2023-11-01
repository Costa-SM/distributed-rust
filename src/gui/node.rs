pub struct Node {
    id: u64,
    title: String,
    position: egui::Pos2,
}

impl Node {
    pub fn new(id: u64, title: String, position: egui::Pos2) -> Self {
        Node {
            id,
            position,
            title: title.to_owned(),
        }
    }

    pub fn ui(&self, ui: &mut egui::Ui) {
        let node_id = ui.make_persistent_id(self.id);

        let _node_window = egui::Area::new(node_id)
            .default_pos(self.position.clone())
            .movable(true)
            .show(ui.ctx(), |ui: &mut egui::Ui| {
                egui::Frame::none()
                    .stroke(egui::Stroke::new(2.0, egui::Color32::RED))
                    .show(ui, |ui| {
                        ui.label(self.title.clone());
                    });
            });
    }
}
