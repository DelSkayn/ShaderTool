#![allow(dead_code)]

use anyhow::Result;
use std::path::PathBuf;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[derive(Debug)]
pub enum UserEvent {
    Reload(PathBuf),
    Gui(gui::Event),
}

mod gui;
mod renderer;
mod watcher;

fn main() -> Result<()> {
    env_logger::init();

    let event_loop = EventLoop::<UserEvent>::with_user_event();
    let proxy = event_loop.create_proxy();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let _watcher = watcher::build(proxy)?;

    let mut imgui = imgui::Context::create();

    let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
    platform.attach_window(
        imgui.io_mut(),
        &window,
        imgui_winit_support::HiDpiMode::Default,
    );

    let mut renderer = futures::executor::block_on(renderer::Renderer::new(&window, &mut imgui))?;
    let mut gui_state = gui::State::new(event_loop.create_proxy());

    event_loop.run(move |event, _, control_flow| {
        platform.handle_event(imgui.io_mut(), &window, &event);
        match event {
            Event::WindowEvent {
                ref event,
                ..
            } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    renderer.resize(*physical_size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    renderer.resize(**new_inner_size);
                }
                WindowEvent::KeyboardInput { input, .. } => match input {
                    winit::event::KeyboardInput {
                    state: winit::event::ElementState::Pressed,
                    virtual_keycode: Some(winit::event::VirtualKeyCode::Escape),
                    ..
                    } => {gui_state.set_error("BLA")}
                    _ => {},
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                let ui_frame = imgui.frame();

                gui::render(&mut gui_state, &ui_frame, &window);

                renderer.render(ui_frame.render()).unwrap();
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    })
}
