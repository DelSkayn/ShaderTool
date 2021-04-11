use imgui::{im_str, Condition, ImString, MenuItem, Ui, Window, WindowFlags};
use winit::{event_loop::EventLoopProxy, window::Window as WinitWindow};

use super::UserEvent;

#[derive(Debug)]
pub enum Event{
    Quit,
}

pub struct State {
    event_loop: EventLoopProxy<UserEvent>,
    error: Option<ImString>,
}

impl State {
    pub fn new(event_loop: EventLoopProxy<UserEvent>) -> State {
        State { error: None, event_loop, }
    }

    pub fn set_error(&mut self, text: impl Into<String>) {
        self.error = Some(ImString::new(text))
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }

    pub fn event(&self,event: Event){
        self.event_loop.send_event(UserEvent::Gui(event)).unwrap();
    }
}

pub fn render(state: &mut State, ui: &Ui<'_>, window: &WinitWindow) {
    ui.main_menu_bar(|| {
        ui.menu(im_str!("File"), true, || {
            if MenuItem::new(im_str!("Quit"))
                .shortcut(im_str!("Ctrl-Q"))
                .build(ui){
                    state.event(Event::Quit)
            }
        })
    });

    Window::new(im_str!("Test"))
        .build(ui,||{
            ui.text(im_str!("AHHHH"))
        });

    let size = window.inner_size().to_logical(window.scale_factor());

    if let Some(x) = state.error.as_ref() {
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
            state.clear_error();
        }

        color.pop(ui);
    }
}
