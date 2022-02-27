use crate::config::{BuiltInUniform, CustomUniform, LoadedPass, UniformBinding, UniformData};

use super::{App, State};
use egui::{self, menu, Color32, ComboBox, RichText, Ui, Window};
use glium::program::Uniform;

pub struct Gui {
    show_uniforms: bool,
}

impl Gui {
    pub fn new() -> Self {
        Gui {
            show_uniforms: false,
        }
    }
}

impl App {
    pub fn draw_gui(&mut self) -> bool {
        self.egui.run(&self.display, |ctx| {
            egui::TopBottomPanel::top("menu").show(ctx, |ui| {
                menu::bar(ui, |ui| {
                    ui.menu_button("Shader Tool", |ui| {
                        if ui.button("Quit").clicked() {
                            self.should_run = false;
                        }
                    });
                    ui.menu_button("Scene", |ui| {
                        if ui.button("Toggle Uniforms").clicked() {
                            self.gui.show_uniforms = !self.gui.show_uniforms;
                        }
                    });
                });
            });

            match self.state {
                State::NotLoaded { ref error } | State::ReloadError { ref error, .. } => {
                    egui::TopBottomPanel::bottom("error_panel").show(ctx, |ui| {
                        ui.heading(RichText::new("！  Error").strong().color(Color32::RED));
                        ui.separator();
                        ui.monospace(error);
                    });
                }
                _ => {}
            }

            if let Some(config) = self.state.active_config_mut() {
                Window::new("Uniforms")
                    .open(&mut self.gui.show_uniforms)
                    .show(ctx, |ui| {
                        if config.passes.is_empty() {
                            ui.label("Config does not contain any render passes!");
                        } else {
                            for (pass_id, pass) in config.passes.iter_mut().enumerate() {
                                ui.collapsing(format!("pass: {}", pass_id), |ui| {
                                    Self::render_uniforms(ui, pass, pass_id);
                                });
                            }
                        }
                    });
            }
        })
    }

    pub fn render_uniforms(ui: &mut Ui, pass: &mut LoadedPass, pass_id: usize) {
        if pass.uniforms.is_empty() {
            ui.label("Pass does not contain any uniforms");
        } else {
            egui::Grid::new(("uniforms_grid", pass_id)).show(ui, |ui| {
                ui.heading("Name");
                ui.heading("Binding");
                ui.heading("Value");
                ui.end_row();
                pass.uniforms
                    .iter_mut()
                    .enumerate()
                    .for_each(|(idx, (name, value))| {
                        ui.monospace(name);
                        Self::render_uniform_data(ui, value, idx, pass_id);
                        ui.end_row();
                    });
            });
        }
    }

    pub fn render_uniform_data(ui: &mut Ui, data: &mut UniformData, idx: usize, pass_id: usize) {
        #[derive(Clone, Copy, Eq, PartialEq)]
        enum BindChoice {
            Unbound,
            Custom,
            BuiltIn,
        }

        impl BindChoice {
            fn label(&self) -> &'static str {
                match *self {
                    Self::Custom => "Custom",
                    Self::BuiltIn => "Built In",
                    Self::Unbound => "Unbound",
                }
            }

            fn from_binding(binding: &UniformBinding) -> Self {
                match *binding {
                    UniformBinding::Custom(_) => BindChoice::Custom,
                    UniformBinding::BuiltIn(_) => BindChoice::BuiltIn,
                    UniformBinding::Unbound => BindChoice::Unbound,
                }
            }

            fn into_binding(self, uniform: &Uniform) -> UniformBinding {
                match self {
                    Self::Custom => UniformBinding::Custom(
                        CustomUniform::from_uniform_type(uniform.ty).unwrap(),
                    ),
                    Self::BuiltIn => UniformBinding::BuiltIn(
                        BuiltInUniform::valid_for_uniform_type(uniform.ty)[0],
                    ),
                    Self::Unbound => UniformBinding::Unbound,
                }
            }
        }

        let before = BindChoice::from_binding(&data.binding);
        let mut choice = before;
        ComboBox::from_id_source(("uniforms", idx, pass_id))
            .selected_text(choice.label())
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut choice,
                    BindChoice::Unbound,
                    BindChoice::Unbound.label(),
                );
                if CustomUniform::from_uniform_type(data.kind.ty).is_some() {
                    ui.selectable_value(
                        &mut choice,
                        BindChoice::Custom,
                        BindChoice::Custom.label(),
                    );
                }
                if !BuiltInUniform::valid_for_uniform_type(data.kind.ty).is_empty() {
                    ui.selectable_value(
                        &mut choice,
                        BindChoice::BuiltIn,
                        BindChoice::BuiltIn.label(),
                    );
                }
            });
        match data.binding {
            UniformBinding::BuiltIn(ref mut x) => {
                let valid = BuiltInUniform::valid_for_uniform_type(data.kind.ty);
                ComboBox::from_id_source(("uniform_builtin", idx, pass_id))
                    .selected_text(x.label())
                    .show_ui(ui, |ui| {
                        for v in valid {
                            ui.selectable_value(x, *v, v.label());
                        }
                    });
            }
            _ => {}
        }

        if choice != before {
            data.binding = choice.into_binding(&data.kind);
        }
    }
}
