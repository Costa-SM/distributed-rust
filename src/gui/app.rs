use egui::emath::Numeric;
use std::io::Read;

use crate::node::{Node, NodeType};
use crate::utils::extract_file_name_from_path;
use crate::worker::Worker;

pub struct MapReduceApp {
    pub running: bool,
    opened_file: Option<std::path::PathBuf>,
    open_file_dialog: Option<egui_file::FileDialog>,
    num_workers: u8,
    reduce_jobs: u8,
    chunk_size: u32,
    workers: Vec<Worker>,
    nodes: Vec<Node>,
    connections: Vec<(egui::Pos2, egui::Pos2)>,
}

impl MapReduceApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            opened_file: None,
            open_file_dialog: None,
            running: false,
            num_workers: 0,
            reduce_jobs: 2,
            chunk_size: 16,
            workers: Vec::new(),
            nodes: Vec::new(),
            connections: Vec::new(),
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

    fn render_file_button(self: &mut Self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let file_name: String = extract_file_name_from_path(&self.opened_file);
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
                    self.workers.clear();
                    for i in 0..self.num_workers {
                        let worker_x_pos: f64 = 50.0 + i.to_f64() * 135.0;
                        self.workers.push(Worker {
                            id: i + 1,
                            title: format!("Worker {}", i + 1),
                            position: egui::Pos2::new(worker_x_pos as f32, 50.0),
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
                    .add(egui::Slider::new(&mut self.chunk_size, 16..=204800))
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
            let mut text_length = 0;
            if let Some(file_path) = &self.opened_file {
                if let Ok(mut file) = std::fs::File::open(&file_path) {
                    let mut contents = String::new();
                    if file.read_to_string(&mut contents).is_ok() {
                        text_length = contents.len();
                    }
                }
            }
            self.running = !self.running;
            self.build_graph(text_length);
        };
    }

    fn build_graph(&mut self, text_length: usize) {
        const NUM_STEPS: usize = 5;
        const NODES_TOP_Y: f32 = 150.0;
        const NODES_BOTTOM_Y: f32 = 675.0;
        const NODES_LEFT_X: f32 = 50.0;
        const NODES_RIGHT_X: f32 = 1045.0;

        const NODES_X_GAP: f32 = (NODES_RIGHT_X - NODES_LEFT_X) / (NUM_STEPS - 1) as f32;

        self.nodes.clear();
        let input_node_pos = egui::Pos2::new(NODES_LEFT_X, (NODES_BOTTOM_Y + NODES_TOP_Y) / 2.0);
        self.nodes.push(Node {
            id: extract_file_name_from_path(&self.opened_file),
            position: input_node_pos,
            node_type: NodeType::Input,
        });
        let result_node_pos = egui::Pos2::new(NODES_RIGHT_X, (NODES_BOTTOM_Y + NODES_TOP_Y) / 2.0);
        let result_node = Node {
            id: "result".to_string(),
            position: result_node_pos,
            node_type: NodeType::Result,
        };
        self.nodes.push(result_node);
        let num_map_nodes: usize =
            ((text_length as f64 / (self.chunk_size as f64)) as f64).ceil() as usize;
        let num_reduce_nodes = num_map_nodes * self.reduce_jobs as usize;
        let reduce_nodes_spacing: f32 =
            (NODES_BOTTOM_Y - NODES_TOP_Y) / (num_reduce_nodes - 1) as f32;
        let node_position_x = NODES_LEFT_X + 2.0 * NODES_X_GAP;
        let mut node_position_y = NODES_TOP_Y;
        let mut last_node_position_y = NODES_TOP_Y;
        for i in 0..(((NODES_BOTTOM_Y - NODES_TOP_Y) / reduce_nodes_spacing) + 1 as f32) as i32 {
            self.nodes.push(Node {
                id: node_position_y.to_string(),
                position: egui::Pos2::new(node_position_x, node_position_y as f32),
                node_type: NodeType::Reduce,
            });
            if i % 2 == 1 {
                let map_node_pos = egui::Pos2::new(
                    NODES_LEFT_X + NODES_X_GAP,
                    ((last_node_position_y + node_position_y) / 2 as f32) as f32,
                );
                self.nodes.push(Node {
                    id: node_position_y.to_string() + "asd",
                    position: map_node_pos,
                    node_type: NodeType::Map,
                });
                self.connections.push((
                    map_node_pos,
                    egui::Pos2::new(node_position_x, node_position_y as f32),
                ));
                self.connections.push((
                    map_node_pos,
                    egui::Pos2::new(node_position_x, last_node_position_y as f32),
                ));
                self.connections.push((input_node_pos, map_node_pos));
                let reduce_result_node_pos = egui::Pos2::new(
                    NODES_LEFT_X + 3.0 * NODES_X_GAP,
                    ((last_node_position_y + node_position_y) / 2 as f32) as f32,
                );
                self.nodes.push(Node {
                    id: node_position_y.to_string() + "fsdg",
                    position: reduce_result_node_pos,
                    node_type: NodeType::ReduceResult,
                });
                self.connections
                    .push((reduce_result_node_pos, result_node_pos));
                self.connections.push((
                    egui::Pos2::new(node_position_x, node_position_y as f32),
                    reduce_result_node_pos,
                ));
                self.connections.push((
                    egui::Pos2::new(node_position_x, last_node_position_y as f32),
                    reduce_result_node_pos,
                ));
            }
            last_node_position_y = node_position_y;
            node_position_y += reduce_nodes_spacing;
        }
    }

    pub fn render_workers(&mut self, ui: &mut eframe::egui::Ui) {
        for worker in self.workers.iter_mut() {
            worker.render_worker(ui);
        }
    }

    pub fn render_nodes(&mut self, ui: &mut eframe::egui::Ui) {
        for node in self.nodes.iter_mut() {
            node.render_node(ui);
        }
        for connection in self.connections.iter_mut() {
            draw_connection(
                ui.painter(),
                connection.0,
                connection.1,
                egui::Color32::DARK_GRAY,
            );
        }
    }
}

fn draw_connection(
    painter: &egui::Painter,
    src_pos: egui::Pos2,
    dst_pos: egui::Pos2,
    color: egui::Color32,
) {
    let connection_stroke = egui::Stroke { width: 2.0, color };

    let control_scale = ((dst_pos.x - src_pos.x) / 2.0).max(30.0);
    let src_control = src_pos + egui::Vec2::X * control_scale;
    let dst_control = dst_pos - egui::Vec2::X * control_scale;

    let bezier = egui::epaint::CubicBezierShape::from_points_stroke(
        [src_pos, src_control, dst_control, dst_pos],
        false,
        egui::Color32::TRANSPARENT,
        connection_stroke,
    );

    painter.circle(src_pos, 1.5, color, connection_stroke);
    painter.circle(dst_pos, 1.5, color, connection_stroke);

    painter.add(bezier);
}
