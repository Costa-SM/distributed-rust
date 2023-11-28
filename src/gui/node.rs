#[derive(Clone, Copy, PartialEq)]
pub enum NodeStatus {
    Failed,
    Default,
    Finished,
}
#[derive(Clone, Copy)]
pub enum NodeType {
    Input,
    Map,
    Reduce,
    Result,
}
#[derive(Clone)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub status: NodeStatus,
    pub node_type: NodeType,
    pub position: egui::Pos2,
}

impl Node {
    pub fn render_node(&mut self, ui: &mut egui::Ui) {
        let _node_window: egui::InnerResponse<()> = egui::Area::new(self.id.clone())
            .default_pos(self.position.clone())
            .movable(false)
            .show(ui.ctx(), |ui: &mut egui::Ui| {
                let color: egui::Color32 = if self.status == NodeStatus::Failed {
                    egui::Color32::DARK_RED
                } else if self.status == NodeStatus::Finished {
                    egui::Color32::DARK_GREEN
                } else {
                    egui::Color32::DARK_GRAY
                };
                egui::Frame::none()
                    .stroke(egui::Stroke::new(2.0, color))
                    .inner_margin(egui::Margin::same(10.0))
                    .rounding(5.0)
                    .show(ui, |ui: &mut egui::Ui| {
                        ui.label(self.label.clone());
                    });
            });
    }
}
