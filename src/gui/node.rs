enum NodeType {
    Map,
    Reduce,
}

pub struct Node {
    node_type: NodeType,
    title: String,
    position: egui::Pos2,
}
