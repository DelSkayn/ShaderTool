use super::{App, State};
use egui::{self, menu, Color32, Label};

impl App {
    pub fn draw_gui(&mut self) {
        egui::TopBottomPanel::top("menu").show(self.egui.ctx(), |ui| {
            menu::bar(ui, |ui| {
                menu::menu(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        self.should_run = false;
                    }
                })
            });
        });

        match self.state {
            State::NotLoaded { ref error } | State::ReloadError { ref error, .. } => {
                egui::TopBottomPanel::bottom("error_panel").show(self.egui.ctx(), |ui| {
                    ui.add(
                        Label::new("！  Error")
                            .heading()
                            .strong()
                            .text_color(Color32::RED),
                    );
                    ui.separator();
                    ui.monospace(error);
                });
            }
            _ => {}
        }
    }
}
