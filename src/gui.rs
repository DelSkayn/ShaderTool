use egui::{self, menu, Color32, CtxRef, Label};

pub struct Model {
    error: Option<String>,
    run: bool,
}

impl Model {
    pub fn new() -> Self {
        Self {
            error: None,
            run: true,
        }
    }

    pub fn should_run(&self) -> bool {
        self.run
    }

    pub fn draw(&mut self, ui: &CtxRef) {
        egui::TopBottomPanel::top("menu").show(ui, |ui| {
            menu::bar(ui, |ui| {
                menu::menu(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        self.run = false;
                    }
                })
            });
        });

        if let Some(error) = self.error.clone() {
            egui::TopBottomPanel::bottom("error_panel").show(ui, |ui| {
                ui.add(
                    Label::new("！  Error")
                        .heading()
                        .strong()
                        .text_color(Color32::RED),
                );
                ui.separator();
                ui.monospace(error);
                if ui.small_button("Close").clicked() {
                    self.error = None;
                }
            });
        }
    }

    pub fn set_error(&mut self, error: Option<String>) {
        self.error = error;
    }
}
