use crate::Application;
use glium::Display;
use imgui::{im_str, Condition, ImString, MenuItem, Ui, Window, WindowFlags};

pub struct State {
    error: Option<ImString>,
}

impl State {
    pub fn new() -> State {
        State { error: None }
    }

    pub fn set_error(&mut self, text: impl Into<String>) {
        self.error = Some(ImString::new(text))
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }
}

pub fn build(app: &mut Application, ui: &Ui<'_>, display: &Display) {
    ui.main_menu_bar(|| {
        ui.menu(im_str!("File"), true, || {
            app.should_run = !MenuItem::new(im_str!("Quit"))
                .shortcut(im_str!("Ctrl-Q"))
                .build(ui);
        })
    });
    let window = display.gl_window();
    let window = window.window();

    let size = window.inner_size().to_logical(window.scale_factor());

    if let Some(x) = app.gui_state.error.as_ref() {
        let mut error_open = true;
        let color = ui.push_style_color(
            imgui::StyleColor::TitleBgActive,
            [191.0 / 255.0, 46.0 / 255.0, 51.0 / 255.0, 1.0],
        );

        Window::new(im_str!("Error"))
            .opened(&mut error_open)
            .position_pivot([0.0, 1.0])
            .size_constraints([size.width, -1.0], [size.width, -1.0])
            .position([0.0, size.height], Condition::Always)
            .flags(WindowFlags::NO_MOVE | WindowFlags::NO_RESIZE)
            .build(ui, || {
                ui.text_wrapped(x);
            });
        if !error_open {
            app.gui_state.clear_error();
        }

        color.pop(ui);
    }
}
