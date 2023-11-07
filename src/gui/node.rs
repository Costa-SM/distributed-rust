#[derive(Clone, Copy)]
pub enum NodeType {
    Input,
    Map,
    Reduce,
    ReduceResult,
    Result,
}
#[derive(Clone)]
pub struct Node {
    pub id: String,
    pub node_type: NodeType,
    pub position: egui::Pos2,
}

impl Node {
    pub fn render_node(&mut self, ui: &mut egui::Ui) {
        let node_id = ui.make_persistent_id(self.id.clone());

        let _node_window: egui::InnerResponse<()> = egui::Area::new(node_id)
            .default_pos(self.position.clone())
            .movable(false)
            .show(ui.ctx(), |ui: &mut egui::Ui| {
                egui::Frame::none()
                    .stroke(egui::Stroke::new(2.0, egui::Color32::DARK_GRAY))
                    .inner_margin(egui::Margin::same(10.0))
                    .rounding(5.0)
                    .show(ui, |ui: &mut egui::Ui| {
                        ui.label(self.id.clone());
                    });
            });
    }
}
